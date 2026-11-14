//! `kinhin prune`: delete construction-time tests (scaffold,
//! characterization). The suite must still collect afterwards; every
//! file is restored when it does not.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use comfy_table::{Cell, CellAlignment, Color};
use owo_colors::OwoColorize;
use serde::Serialize;

use crate::census::Census;
use crate::detect::{self, Language};
use crate::display;
use crate::env::{self, PyEnv};
use crate::pycheck;
use crate::pytests;
use crate::tags::{self, LifecycleTag, TaggedTest};
use crate::OutputFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PruneAction {
    Keep,
    Delete,
    /// Untagged: prune refuses to guess.
    TagFirst,
}

#[derive(Debug, Clone, Serialize)]
pub struct PruneEntry {
    pub file: String,
    pub test: String,
    pub tag: String,
    pub action: PruneAction,
}

#[derive(Debug, Default, Serialize)]
struct PruneResult {
    applied: bool,
    deleted_tests: usize,
    deleted_files: Vec<String>,
    census_line: String,
}

#[derive(Debug, Serialize)]
struct PruneReport {
    before: Census,
    plan: Vec<PruneEntry>,
    result: PruneResult,
}

fn relative(path: &Path, root: &Path) -> String {
    let absolute = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    absolute.strip_prefix(root).unwrap_or(&absolute).display().to_string()
}

pub fn plan(tests: &[TaggedTest], root: &Path) -> Vec<PruneEntry> {
    tests
        .iter()
        .map(|test| PruneEntry {
            file: relative(&test.file, root),
            test: test.name.clone().unwrap_or_else(|| "(anonymous)".into()),
            tag: test.tag.to_string(),
            action: match test.tag {
                LifecycleTag::Scaffold | LifecycleTag::Characterization => PruneAction::Delete,
                LifecycleTag::Untagged => PruneAction::TagFirst,
                _ => PruneAction::Keep,
            },
        })
        .collect()
}

type Deletions = BTreeMap<String, BTreeSet<String>>;

struct Workspace<'a> {
    root: &'a Path,
    /// Original content of every file prune may touch.
    originals: BTreeMap<String, String>,
}

impl Workspace<'_> {
    fn restore(&self) {
        for (file, content) in &self.originals {
            let _ = std::fs::write(self.root.join(file), content);
        }
    }

    /// Rewrite every file from its original content minus `deletions`.
    /// Returns (tests deleted, files removed).
    fn write(&self, deletions: &Deletions) -> Result<(usize, Vec<String>), String> {
        let mut deleted = 0;
        let mut removed_files = Vec::new();
        for (file, original) in &self.originals {
            let path = self.root.join(file);
            let wanted: HashSet<String> = deletions.get(file).map(|set| set.iter().cloned().collect()).unwrap_or_default();
            if wanted.is_empty() {
                std::fs::write(&path, original).map_err(|e| format!("cannot write {file}: {e}"))?;
                continue;
            }
            let outcome = pytests::delete_tests(original, &wanted);
            deleted += outcome.removed.len();
            if outcome.empty {
                if path.exists() {
                    std::fs::remove_file(&path).map_err(|e| format!("cannot delete {file}: {e}"))?;
                }
                removed_files.push(file.clone());
            } else {
                std::fs::write(&path, outcome.content).map_err(|e| format!("cannot write {file}: {e}"))?;
            }
        }
        Ok((deleted, removed_files))
    }
}

/// Exit code: 0 done, 1 refused or rolled back, 2 tool error.
pub fn run(path: &str, apply: bool, format: OutputFormat) -> i32 {
    let root = env::canonical_root(path);
    let languages = detect::detect_languages(path);
    if languages.is_empty() {
        display::print_error("no language detected.");
        return 2;
    }

    let tests = tags::scan_test_files(&root.display().to_string(), &languages);
    if tests.is_empty() {
        display::print_warning("no tests found");
        return 0;
    }
    let before = Census::from_tests(&tests);
    let entries = plan(&tests, &root);
    let mut report = PruneReport {
        before: before.clone(),
        plan: entries,
        result: PruneResult::default(),
    };

    if format == OutputFormat::Rich {
        display_plan(&report);
    }

    let code = if !apply {
        if before.untagged > 0 {
            display::print_warning(&format!(
                "{} untagged test(s): run `kinhin tag` before `kinhin prune --apply`",
                before.untagged
            ));
        }
        0
    } else if before.untagged > 0 {
        display::print_error(&format!(
            "{} untagged test(s): prune will not guess what they are. Run `kinhin tag` first.",
            before.untagged
        ));
        1
    } else if !languages.contains(&Language::Python) || languages.len() > 1 {
        display::print_error("deleting tests is implemented for Python-only projects. The plan above lists what to delete by hand.");
        2
    } else {
        match apply_plan(&root, &report.plan, &before) {
            Ok(result) => {
                report.result = result;
                0
            }
            Err((code, message)) => {
                display::print_error(&message);
                code
            }
        }
    };

    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report).expect("report serializes")),
        OutputFormat::Rich if report.result.applied => display_result(&report.result),
        OutputFormat::Rich => {}
    }
    code
}

fn apply_plan(root: &Path, entries: &[PruneEntry], before: &Census) -> Result<PruneResult, (i32, String)> {
    let mut deletions: Deletions = BTreeMap::new();
    for entry in entries.iter().filter(|e| e.action == PruneAction::Delete) {
        deletions.entry(entry.file.clone()).or_default().insert(entry.test.clone());
    }
    if deletions.is_empty() {
        return Ok(PruneResult {
            applied: true,
            census_line: census_line(before, before),
            ..PruneResult::default()
        });
    }

    let py = PyEnv::detect(root);
    let mut originals = BTreeMap::new();
    for file in deletions.keys() {
        let content = std::fs::read_to_string(root.join(file)).map_err(|e| (2, format!("cannot read {file}: {e}")))?;
        originals.insert(file.clone(), content);
    }
    let workspace = Workspace { root, originals };

    let rollback = |code: i32, message: String| {
        workspace.restore();
        (code, format!("{message}\n  every test file was restored; nothing was deleted."))
    };

    eprintln!("{} deleting {} scaffold test(s)...", "→".cyan(), deletions.values().map(BTreeSet::len).sum::<usize>());
    let (deleted, removed_files) = workspace.write(&deletions).map_err(|m| rollback(2, m))?;
    pycheck::collect(&py).map_err(|m| rollback(1, m))?;

    // Deleting tests can orphan imports; let the project's own ruff tidy them.
    let touched: Vec<PathBuf> = workspace
        .originals
        .keys()
        .filter(|file| deletions.get(*file).is_some_and(|set| !set.is_empty()))
        .map(|file| root.join(file))
        .filter(|path| path.exists())
        .collect();
    if pycheck::ruff_fix(&py, &touched, "F401,I") {
        pycheck::collect(&py).map_err(|m| rollback(1, m))?;
    }

    let after = Census::from_tests(&tags::scan_test_files(&root.display().to_string(), &[Language::Python]));
    Ok(PruneResult {
        applied: true,
        deleted_tests: deleted,
        deleted_files: removed_files,
        census_line: census_line(before, &after),
    })
}

fn census_line(before: &Census, after: &Census) -> String {
    format!(
        "written {} / pruned {} / survivors: {} decision, {} contract, {} incident, {} scaffold",
        before.total,
        before.total.saturating_sub(after.total),
        after.decision,
        after.contract,
        after.incident,
        after.temporary_count(),
    )
}

fn display_plan(report: &PruneReport) {
    let mut table = display::styled_table();
    table.set_header(vec![
        Cell::new("File").set_alignment(CellAlignment::Left),
        Cell::new("Test").set_alignment(CellAlignment::Left),
        Cell::new("Tag").set_alignment(CellAlignment::Left),
        Cell::new("Action").set_alignment(CellAlignment::Left),
    ]);
    for entry in &report.plan {
        let (label, color) = match entry.action {
            PruneAction::Keep => ("✓ KEEP", Color::Green),
            PruneAction::Delete => ("✗ DELETE", Color::Red),
            PruneAction::TagFirst => ("? TAG FIRST", Color::Yellow),
        };
        table.add_row(vec![
            Cell::new(&entry.file),
            Cell::new(&entry.test),
            Cell::new(&entry.tag),
            Cell::new(label).fg(color),
        ]);
    }
    println!();
    display::print_titled(&format!("{}", "Prune Plan".bold()), &table);

    let count = |action: PruneAction| report.plan.iter().filter(|e| e.action == action).count();
    println!(
        "\n  {} keep  {} delete  {} untagged",
        count(PruneAction::Keep).to_string().green(),
        count(PruneAction::Delete).to_string().red(),
        count(PruneAction::TagFirst).to_string().yellow(),
    );
}

fn display_result(result: &PruneResult) {
    println!();
    display::print_success(&format!(
        "{} test(s) deleted{}",
        result.deleted_tests,
        if result.deleted_files.is_empty() {
            String::new()
        } else {
            format!(", {} file(s) removed: {}", result.deleted_files.len(), result.deleted_files.join(", "))
        }
    ));
    if result.deleted_tests > 0 {
        display::print_warning("run the suite (`make test`): a red test means a deleted one was load-bearing — restore it as a decision");
    }
    println!("\n  {}", result.census_line.bold());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tagged(name: &str, tag: LifecycleTag) -> TaggedTest {
        TaggedTest {
            file: PathBuf::from("/p/tests/test_a.py"),
            name: Some(name.to_string()),
            tag,
            ref_value: None,
        }
    }

    #[test]
    fn only_temporary_tags_are_planned_for_deletion() {
        let tests = [
            tagged("a", LifecycleTag::Scaffold),
            tagged("b", LifecycleTag::Characterization),
            tagged("c", LifecycleTag::Decision),
            tagged("d", LifecycleTag::Contract),
            tagged("e", LifecycleTag::Incident),
            tagged("f", LifecycleTag::Untagged),
        ];
        let actions: Vec<PruneAction> = plan(&tests, Path::new("/p")).into_iter().map(|e| e.action).collect();
        assert_eq!(
            actions,
            [
                PruneAction::Delete,
                PruneAction::Delete,
                PruneAction::Keep,
                PruneAction::Keep,
                PruneAction::Keep,
                PruneAction::TagFirst
            ]
        );
    }
}
