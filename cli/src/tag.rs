use std::path::Path;

use comfy_table::{Cell, CellAlignment, Color};
use owo_colors::OwoColorize;

use crate::agent::{self, TagSuggestion};
use crate::detect::{self, Language};
use crate::display;
use crate::tags::{self, LifecycleTag, TaggedTest};
use crate::OutputFormat;

#[derive(serde::Serialize)]
struct TagReport {
    files_scanned: usize,
    untagged_found: usize,
    suggestions: Vec<FileSuggestions>,
}

#[derive(serde::Serialize)]
struct FileSuggestions {
    file: String,
    suggestions: Vec<TagSuggestion>,
}

pub async fn run(
    path: &str,
    lang: Option<Language>,
    apply: bool,
    format: OutputFormat,
) {
    let languages = lang
        .map(|l| vec![l])
        .unwrap_or_else(|| detect::detect_languages(path));

    if languages.is_empty() {
        display::print_error("no language detected — use --lang to specify");
        std::process::exit(1);
    }

    let tests = tags::scan_test_files(path, &languages);
    let untagged: Vec<&TaggedTest> = tests
        .iter()
        .filter(|t| t.tag == LifecycleTag::Untagged)
        .collect();

    if untagged.is_empty() {
        display::print_success("all tests already tagged — nothing to do");
        return;
    }

    println!(
        "{} {} untagged test(s) across {} file(s)",
        "→".cyan(),
        untagged.len(),
        untagged
            .iter()
            .map(|t| &t.file)
            .collect::<std::collections::HashSet<_>>()
            .len(),
    );

    // Group untagged tests by file
    let mut files_map: std::collections::BTreeMap<String, Vec<&TaggedTest>> =
        std::collections::BTreeMap::new();

    for test in &untagged {
        let key = test.file.display().to_string();
        files_map.entry(key).or_default().push(test);
    }

    let root = Path::new(path)
        .canonicalize()
        .unwrap_or_else(|_| Path::new(path).to_path_buf());

    let primary_lang = languages[0];
    let mut all_suggestions: Vec<FileSuggestions> = Vec::new();

    let mut session = match agent::TaggerSession::connect(primary_lang).await {
        Ok(s) => s,
        Err(e) => {
            display::print_error(&format!("failed to start agent session: {e}"));
            std::process::exit(1);
        }
    };

    for (file_path, _file_tests) in &files_map {
        let Ok(content) = std::fs::read_to_string(file_path) else {
            display::print_warning(&format!("cannot read {file_path}, skipping"));
            continue;
        };

        println!("  {} {}", "⠋".cyan(), shorten(file_path, &root));

        match session.tag_file(&content).await {
            Ok(suggestions) => {
                all_suggestions.push(FileSuggestions {
                    file: shorten(file_path, &root),
                    suggestions,
                });
            }
            Err(e) => {
                display::print_error(&format!("agent failed on {}: {e}", shorten(file_path, &root)));
            }
        }
    }

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

    match format {
        OutputFormat::Json => display_json(&all_suggestions, untagged.len()),
        OutputFormat::Rich => display_rich(&all_suggestions),
    }

    if apply {
        display::print_warning("--apply: tag insertion not yet implemented in v0.1 — apply manually from the suggestions above");
    }
}

fn display_json(suggestions: &[FileSuggestions], untagged_count: usize) {
    let report = TagReport {
        files_scanned: suggestions.len(),
        untagged_found: untagged_count,
        suggestions: suggestions.to_vec(),
    };
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}

fn display_rich(suggestions: &[FileSuggestions]) {
    if suggestions.is_empty() {
        return;
    }

    let mut table = display::styled_table();
    table.set_header(vec![
        Cell::new("File").set_alignment(CellAlignment::Left),
        Cell::new("Test").set_alignment(CellAlignment::Left),
        Cell::new("Suggested").set_alignment(CellAlignment::Left),
        Cell::new("Reason").set_alignment(CellAlignment::Left),
        Cell::new("Ref").set_alignment(CellAlignment::Left),
    ]);

    for file_sugg in suggestions {
        for s in &file_sugg.suggestions {
            let (color, icon) = tag_style(&s.tag);
            table.add_row(vec![
                Cell::new(&file_sugg.file),
                Cell::new(&s.test),
                Cell::new(format!("{icon} {}", s.tag)).fg(color),
                Cell::new(&s.reason),
                Cell::new(
                    s.ref_value
                        .as_deref()
                        .unwrap_or("—"),
                ),
            ]);
        }
    }

    println!();
    display::print_titled(&format!("{}", "Tag Suggestions".bold()), &table);

    let total: usize = suggestions.iter().map(|f| f.suggestions.len()).sum();
    let scaffolds: usize = suggestions
        .iter()
        .flat_map(|f| &f.suggestions)
        .filter(|s| s.tag == "scaffold")
        .count();
    let permanent = total - scaffolds;

    println!();
    println!(
        "  {} suggestions: {} scaffold, {} permanent",
        total,
        scaffolds.to_string().dimmed(),
        permanent.to_string().green(),
    );
}

fn tag_style(tag: &str) -> (Color, &'static str) {
    match tag {
        "scaffold" => (Color::DarkGrey, "◇"),
        "decision" => (Color::Green, "◆"),
        "contract" => (Color::Blue, "◆"),
        "incident" => (Color::Red, "⚡"),
        _ => (Color::Yellow, "?"),
    }
}

fn shorten(path: &str, root: &Path) -> String {
    let p = Path::new(path);
    p.strip_prefix(root)
        .unwrap_or(p)
        .display()
        .to_string()
}

// Make FileSuggestions cloneable for the JSON report
impl Clone for FileSuggestions {
    fn clone(&self) -> Self {
        Self {
            file: self.file.clone(),
            suggestions: self.suggestions.clone(),
        }
    }
}
