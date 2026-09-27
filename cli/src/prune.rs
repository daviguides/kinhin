use std::collections::BTreeMap;
use std::path::Path;

use comfy_table::{Cell, CellAlignment, Color};
use owo_colors::OwoColorize;

use crate::agent;
use crate::census::Census;
use crate::detect::{self, Language};
use crate::display;
use crate::tags::{self, LifecycleTag, TaggedTest};
use crate::OutputFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum PruneAction {
    Keep,
    Collapse,
    Delete,
    Classify,
}

impl PruneAction {
    fn icon(&self) -> &'static str {
        match self {
            PruneAction::Keep => "✓",
            PruneAction::Collapse => "⊕",
            PruneAction::Delete => "✗",
            PruneAction::Classify => "?",
        }
    }

    fn color(&self) -> Color {
        match self {
            PruneAction::Keep => Color::Green,
            PruneAction::Collapse => Color::Yellow,
            PruneAction::Delete => Color::Red,
            PruneAction::Classify => Color::Magenta,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PrunePlan {
    pub entries: Vec<PruneEntry>,
    pub before: Census,
    pub after_estimate: AfterEstimate,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PruneEntry {
    pub file: String,
    pub test: String,
    pub tag: String,
    pub action: PruneAction,
    pub reason: String,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct AfterEstimate {
    pub kept: usize,
    pub collapsed: usize,
    pub deleted: usize,
    pub classify: usize,
}

pub async fn run(
    path: &str,
    lang: Option<Language>,
    apply: bool,
    verify: bool,
    format: OutputFormat,
) {
    let languages = lang
        .map(|l| vec![l])
        .unwrap_or_else(|| detect::detect_languages(path));

    if languages.is_empty() {
        display::print_error("no language detected — use --lang to specify");
        std::process::exit(1);
    }

    let root = Path::new(path)
        .canonicalize()
        .unwrap_or_else(|_| Path::new(path).to_path_buf());

    // Step 1: Census
    println!("{} scanning tests...", "→".cyan());
    let tests = tags::scan_test_files(path, &languages);

    if tests.is_empty() {
        display::print_warning("no tests found");
        return;
    }

    let before = Census::from_tests(&tests);
    println!(
        "  {} total, {} scaffold, {} permanent, {} untagged",
        before.total,
        before.temporary_count().to_string().dimmed(),
        before.permanent_count().to_string().green(),
        if before.untagged > 0 {
            before.untagged.to_string().yellow().to_string()
        } else {
            "0".dimmed().to_string()
        },
    );

    // Step 2: Classify untagged via agent session
    let mut classified_tags: BTreeMap<String, String> = BTreeMap::new();

    let untagged: Vec<&TaggedTest> = tests
        .iter()
        .filter(|t| t.tag == LifecycleTag::Untagged)
        .collect();

    if !untagged.is_empty() {
        println!("\n{} classifying {} untagged test(s) via agent session...", "→".cyan(), untagged.len());

        let primary_lang = languages[0];

        let session = match agent::TaggerSession::connect(primary_lang).await {
            Ok(s) => Some(s),
            Err(e) => {
                display::print_warning(&format!("agent session failed: {e}, treating untagged as scaffold"));
                None
            }
        };
        // Shadow with mutable binding for tag_file calls
        let mut session = session;

        // Group by file
        let mut file_groups: BTreeMap<String, Vec<&TaggedTest>> = BTreeMap::new();
        for test in &untagged {
            let key = test.file.display().to_string();
            file_groups.entry(key).or_default().push(test);
        }

        if let Some(ref mut session) = session {
            for (file_path, _) in &file_groups {
                let Ok(content) = std::fs::read_to_string(file_path) else {
                    continue;
                };

                println!("  {} {}", "⠋".cyan(), shorten(file_path, &root));

                match session.tag_file(&content).await {
                    Ok(suggestions) => {
                        for s in suggestions {
                            let key = format!("{}::{}", file_path, s.test);
                            classified_tags.insert(key, s.tag);
                        }
                    }
                    Err(e) => {
                        display::print_warning(&format!("agent failed: {e}"));
                    }
                }
            }
        }

        if let Some(session) = session {
            match session.disconnect().await {
                Ok(metrics) => {
                    println!(
                        "\n  {} files: {}  turns: {}  cost: ${:.4}",
                        "⧗".dimmed(),
                        metrics.files_processed,
                        metrics.total_turns,
                        metrics.total_cost_usd,
                    );
                }
                Err(e) => display::print_warning(&format!("session disconnect: {e}")),
            }
        }
    }

    // Step 3-4: Build prune plan
    println!("\n{} building prune plan...", "→".cyan());

    let mut entries: Vec<PruneEntry> = Vec::new();
    let mut scaffold_by_file: BTreeMap<String, Vec<&TaggedTest>> = BTreeMap::new();

    for test in &tests {
        let file_str = shorten(&test.file.display().to_string(), &root);
        let test_name = test.name.clone().unwrap_or_else(|| "(anonymous)".into());
        let tag_key = format!("{}::{}", test.file.display(), test_name);

        let effective_tag = if test.tag == LifecycleTag::Untagged {
            classified_tags
                .get(&tag_key)
                .map(|s| s.as_str())
                .unwrap_or("scaffold")
        } else {
            &test.tag.to_string()
        };

        let (action, reason) = match effective_tag {
            "decision" | "contract" | "incident" => {
                (PruneAction::Keep, format!("permanent: {effective_tag}"))
            }
            "scaffold" | "characterization" => {
                scaffold_by_file
                    .entry(file_str.clone())
                    .or_default()
                    .push(test);

                let file_scaffolds = scaffold_by_file.get(&file_str).map_or(0, |v| v.len());
                if file_scaffolds > 3 {
                    (PruneAction::Collapse, "multiple scaffolds — collapse into table-driven".into())
                } else {
                    (PruneAction::Delete, "scaffold with no authority".into())
                }
            }
            _ => (PruneAction::Classify, "could not classify — manual review needed".into()),
        };

        entries.push(PruneEntry {
            file: file_str,
            test: test_name,
            tag: effective_tag.to_string(),
            action,
            reason,
        });
    }

    let after_estimate = AfterEstimate {
        kept: entries.iter().filter(|e| e.action == PruneAction::Keep).count(),
        collapsed: entries.iter().filter(|e| e.action == PruneAction::Collapse).count(),
        deleted: entries.iter().filter(|e| e.action == PruneAction::Delete).count(),
        classify: entries.iter().filter(|e| e.action == PruneAction::Classify).count(),
    };

    let plan = PrunePlan {
        entries,
        before,
        after_estimate,
    };

    // Step 5: Display plan
    match format {
        OutputFormat::Json => display_json(&plan),
        OutputFormat::Rich => display_rich(&plan),
    }

    // Step 6: Apply
    if apply {
        apply_deletions(&plan, &root);
    }

    // Step 7: Verify via mutation gate
    if verify {
        println!("\n{} running mutation parity gate...", "→".cyan());
        display::print_warning("mutation gate: run `kinhin gate` separately in this version");
    }

    // Step 8: Census line
    println!();
    let _survivors = plan.after_estimate.kept + plan.after_estimate.collapsed;
    println!(
        "  {}",
        format!(
            "written {} / pruned {} / survivors: {} kept, {} to collapse",
            plan.before.total,
            plan.after_estimate.deleted,
            plan.after_estimate.kept,
            plan.after_estimate.collapsed,
        )
        .bold(),
    );
}

fn display_json(plan: &PrunePlan) {
    println!("{}", serde_json::to_string_pretty(plan).unwrap());
}

fn display_rich(plan: &PrunePlan) {
    let mut table = display::styled_table();
    table.set_header(vec![
        Cell::new("File").set_alignment(CellAlignment::Left),
        Cell::new("Test").set_alignment(CellAlignment::Left),
        Cell::new("Tag").set_alignment(CellAlignment::Left),
        Cell::new("Action").set_alignment(CellAlignment::Center),
        Cell::new("Reason").set_alignment(CellAlignment::Left),
    ]);

    for entry in &plan.entries {
        let action_color = entry.action.color();
        let action_icon = entry.action.icon();
        let action_label = match entry.action {
            PruneAction::Keep => "KEEP",
            PruneAction::Collapse => "COLLAPSE",
            PruneAction::Delete => "DELETE",
            PruneAction::Classify => "CLASSIFY",
        };

        let tag_color = match entry.tag.as_str() {
            "scaffold" | "characterization" => Color::DarkGrey,
            "decision" => Color::Green,
            "contract" => Color::Blue,
            "incident" => Color::Red,
            _ => Color::Yellow,
        };

        table.add_row(vec![
            Cell::new(&entry.file),
            Cell::new(&entry.test),
            Cell::new(&entry.tag).fg(tag_color),
            Cell::new(format!("{action_icon} {action_label}")).fg(action_color),
            Cell::new(&entry.reason),
        ]);
    }

    println!();
    display::print_titled(&format!("{}", "Prune Plan".bold()), &table);

    println!();
    let est = &plan.after_estimate;
    println!(
        "  {} keep  {} collapse  {} delete  {} classify",
        est.kept.to_string().green(),
        est.collapsed.to_string().yellow(),
        est.deleted.to_string().red(),
        if est.classify > 0 {
            est.classify.to_string().magenta().to_string()
        } else {
            "0".dimmed().to_string()
        },
    );
}

fn apply_deletions(plan: &PrunePlan, root: &Path) {
    let deletions: Vec<&PruneEntry> = plan
        .entries
        .iter()
        .filter(|e| e.action == PruneAction::Delete)
        .collect();

    if deletions.is_empty() {
        display::print_success("nothing to delete");
        return;
    }

    // Group by file — if ALL tests in a scaffold file are deleted, remove the file
    let mut file_deletion_counts: BTreeMap<&str, usize> = BTreeMap::new();
    let mut file_total_counts: BTreeMap<&str, usize> = BTreeMap::new();

    for entry in &plan.entries {
        *file_total_counts.entry(&entry.file).or_default() += 1;
    }
    for entry in &deletions {
        *file_deletion_counts.entry(&entry.file).or_default() += 1;
    }

    let mut files_removed = 0;
    let mut tests_noted = 0;

    for (file, del_count) in &file_deletion_counts {
        let total = file_total_counts.get(file).copied().unwrap_or(0);
        if *del_count == total {
            // All tests in file are scaffold — delete the whole file
            let full_path = root.join(file);
            if full_path.exists() {
                if let Err(e) = std::fs::remove_file(&full_path) {
                    display::print_error(&format!("failed to delete {file}: {e}"));
                } else {
                    println!("  {} deleted {file}", "✗".red());
                    files_removed += 1;
                }
            }
        } else {
            println!(
                "  {} {file}: {del_count}/{total} tests to remove manually",
                "⚠".yellow()
            );
            tests_noted += *del_count;
        }
    }

    if files_removed > 0 {
        println!();
        display::print_success(&format!("{files_removed} scaffold file(s) deleted"));
    }
    if tests_noted > 0 {
        display::print_warning(&format!(
            "{tests_noted} test(s) in mixed files — remove manually (function-level deletion not yet automated)"
        ));
    }
}

fn shorten(path: &str, root: &Path) -> String {
    let p = Path::new(path);
    p.strip_prefix(root)
        .unwrap_or(p)
        .display()
        .to_string()
}
