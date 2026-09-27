//! `kinhin prune`: delete construction-time tests (scaffold,
//! characterization) and, with `--verify`, prove by mutation parity that
//! nothing load-bearing went with them.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use comfy_table::{Cell, CellAlignment, Color};
use owo_colors::OwoColorize;
use serde::Serialize;

use crate::census::Census;
use crate::detect::{self, Language};
use crate::display;
use crate::env::{self, PyEnv};
use crate::gate::{self, MutationRun};
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
    verified: bool,
    deleted_tests: usize,
    deleted_files: Vec<String>,
    /// Scaffolds restored because mutants escaped without them.
    load_bearing: Vec<String>,
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

/// `tests/x.py::TestA::test_b[param]` → (`tests/x.py`, `TestA::test_b`).
fn split_node_id(node_id: &str) -> Option<(String, String)> {
    let (file, rest) = node_id.split_once("::")?;
    let qualified = rest.split('[').next().unwrap_or(rest);
    Some((file.to_string(), qualified.to_string()))
}

/// Tests (file, qualified name) that execute each mangled function,
/// from the stats mutmut wrote during its last run.
fn tests_by_function(root: &Path) -> BTreeMap<String, Vec<(String, String)>> {
    let Ok(content) = std::fs::read_to_string(root.join("mutants/mutmut-stats.json")) else {
        return BTreeMap::new();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return BTreeMap::new();
    };
    value
        .get("tests_by_mangled_function_name")
        .and_then(|v| v.as_object())
        .map(|map| {
            map.iter()
                .map(|(function, tests)| {
                    let ids = tests
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|t| t.as_str())
                        .filter_map(split_node_id)
                        .collect();
                    (function.clone(), ids)
                })
                .collect()
        })
        .unwrap_or_default()
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
pub fn run(path: &str, apply: bool, verify: bool, format: OutputFormat) -> i32 {
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
        match apply_plan(&root, &report.plan, verify, &before) {
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

fn apply_plan(root: &Path, entries: &[PruneEntry], verify: bool, before: &Census) -> Result<PruneResult, (i32, String)> {
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

    // K₀: what the full suite kills, and which tests execute which function.
    let baseline: Option<MutationRun> = if verify {
        eprintln!("{} mutation baseline (K₀) with the full suite...", "→".cyan());
        Some(gate::run_mutmut(&py, &[]).map_err(|m| (2, format!("{m}\n  nothing was deleted.")))?)
    } else {
        None
    };
    let coverage = if verify { tests_by_function(root) } else { BTreeMap::new() };

    let rollback = |code: i32, message: String| {
        workspace.restore();
        (code, format!("{message}\n  every test file was restored; nothing was deleted."))
    };

    eprintln!("{} deleting {} scaffold test(s)...", "→".cyan(), deletions.values().map(BTreeSet::len).sum::<usize>());
    let (mut deleted, mut removed_files) = workspace.write(&deletions).map_err(|m| rollback(2, m))?;
    pycheck::collect(&py).map_err(|m| rollback(1, m))?;

    let mut load_bearing: Vec<String> = Vec::new();
    if let Some(baseline) = &baseline {
        eprintln!("{} mutation run (K₁) without the scaffolds...", "→".cyan());
        let current = gate::run_mutmut(&py, &[]).map_err(|m| rollback(1, m))?;
        let lost = gate::escaped(baseline, &current);

        if !lost.is_empty() {
            // Scaffolds that execute a function whose mutants escaped are the
            // only candidates for having killed them: put those back.
            let at_risk: BTreeSet<&str> = lost.iter().map(|e| gate::function_of(&e.mutant)).collect();
            for function in &at_risk {
                for (file, qualified) in coverage.get(*function).into_iter().flatten() {
                    if deletions.get_mut(file).is_some_and(|set| set.remove(qualified)) {
                        load_bearing.push(format!("{file}::{qualified}"));
                    }
                }
            }
            if load_bearing.is_empty() {
                return Err(rollback(
                    1,
                    format!(
                        "{} mutant(s) escaped and no deleted test could be tied to them (first: {})",
                        lost.len(),
                        lost[0].mutant
                    ),
                ));
            }
            eprintln!(
                "{} {} mutant(s) escaped: restoring {} load-bearing scaffold(s), verifying again...",
                "→".cyan(),
                lost.len(),
                load_bearing.len()
            );
            (deleted, removed_files) = workspace.write(&deletions).map_err(|m| rollback(2, m))?;
            pycheck::collect(&py).map_err(|m| rollback(1, m))?;
            let again = gate::run_mutmut(&py, &[]).map_err(|m| rollback(1, m))?;
            let still_lost = gate::escaped(baseline, &again);
            if !still_lost.is_empty() {
                return Err(rollback(
                    1,
                    format!(
                        "mutation parity not reached: {} baseline-killed mutant(s) still escape (first: {})",
                        still_lost.len(),
                        still_lost[0].mutant
                    ),
                ));
            }
        }
    }

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

    load_bearing.sort();
    let after = Census::from_tests(&tags::scan_test_files(&root.display().to_string(), &[Language::Python]));
    Ok(PruneResult {
        applied: true,
        verified: verify,
        deleted_tests: deleted,
        deleted_files: removed_files,
        load_bearing,
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
    if result.verified {
        display::print_success("mutation parity holds: K₁ ⊇ K₀");
    } else if result.deleted_tests > 0 {
        display::print_warning("not verified: run with --verify (or `kinhin gate`) to prove nothing load-bearing was deleted");
    }
    if !result.load_bearing.is_empty() {
        display::print_warning(&format!(
            "{} scaffold(s) kept: they execute code whose mutants escaped without them. Promote each to a decision with a reason, or collapse them:",
            result.load_bearing.len()
        ));
        for test in &result.load_bearing {
            println!("    {} {test}", "·".yellow());
        }
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

    #[test]
    fn node_ids_map_back_to_file_and_qualified_name() {
        assert_eq!(
            split_node_id("tests/marks/test_x.py::TestA::test_b[case-1]"),
            Some(("tests/marks/test_x.py".to_string(), "TestA::test_b".to_string()))
        );
        assert_eq!(
            split_node_id("tests/test_y.py::test_z"),
            Some(("tests/test_y.py".to_string(), "test_z".to_string()))
        );
    }
}
