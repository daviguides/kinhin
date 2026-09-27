use std::path::{Path, PathBuf};

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
    model: Option<&str>,
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

    let mut session = match agent::TaggerSession::connect(primary_lang, model).await {
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

    // Save suggestions for reuse
    let suggestions_path = Path::new(".kinhin").join("tag-suggestions.json");
    if let Ok(json) = serde_json::to_string_pretty(&all_suggestions) {
        let _ = std::fs::create_dir_all(".kinhin");
        let _ = std::fs::write(&suggestions_path, &json);
    }

    if apply {
        let mut applied = 0;
        let mut failed = 0;

        for file_sugg in &all_suggestions {
            let file_path = if Path::new(&file_sugg.file).is_relative() {
                root.join(&file_sugg.file)
            } else {
                PathBuf::from(&file_sugg.file)
            };

            let Ok(content) = std::fs::read_to_string(&file_path) else {
                display::print_warning(&format!("cannot read {}, skipping apply", file_sugg.file));
                failed += 1;
                continue;
            };

            let new_content = insert_tags(&content, &file_sugg.suggestions, primary_lang);

            if new_content != content {
                if let Err(e) = std::fs::write(&file_path, &new_content) {
                    display::print_error(&format!("write failed {}: {e}", file_sugg.file));
                    failed += 1;
                } else {
                    applied += 1;
                }
            }
        }

        let total_tags: usize = all_suggestions.iter().map(|f| f.suggestions.len()).sum();
        display::print_success(&format!("{applied} file(s) updated, {total_tags} tag(s) inserted"));
        if failed > 0 {
            display::print_warning(&format!("{failed} file(s) failed"));
        }
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

fn insert_tags(content: &str, suggestions: &[TagSuggestion], lang: Language) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result: Vec<String> = Vec::with_capacity(lines.len() + suggestions.len());
    let mut tagged_tests: std::collections::HashMap<&str, &TagSuggestion> =
        suggestions.iter().map(|s| (s.test.as_str(), s)).collect();

    for line in &lines {
        // Check if this line defines a test function that we have a suggestion for
        let test_name = extract_test_name(line, lang);
        if let Some(name) = test_name {
            if let Some(suggestion) = tagged_tests.remove(name) {
                let marker = format_marker(suggestion, lang);
                if !marker.is_empty() {
                    // Detect indentation
                    let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
                    result.push(format!("{indent}{marker}"));
                }
            }
        }
        result.push(line.to_string());
    }

    result.join("\n") + if content.ends_with('\n') { "\n" } else { "" }
}

fn extract_test_name<'a>(line: &'a str, lang: Language) -> Option<&'a str> {
    let trimmed = line.trim();
    match lang {
        Language::Python => {
            if trimmed.starts_with("def test_") || trimmed.starts_with("async def test_") {
                let start = trimmed.find("test_")?;
                let rest = &trimmed[start..];
                let end = rest.find('(')?;
                Some(&rest[..end])
            } else {
                None
            }
        }
        Language::Rust => {
            if trimmed.starts_with("fn test_") || trimmed.starts_with("fn ") && trimmed.contains("test") {
                let start = trimmed.find("fn ")? + 3;
                let rest = &trimmed[start..];
                let end = rest.find('(').unwrap_or(rest.len());
                Some(&rest[..end])
            } else {
                None
            }
        }
        Language::Java => {
            if trimmed.starts_with("void test") || trimmed.starts_with("public void test") {
                let start = trimmed.find("test")?;
                let rest = &trimmed[start..];
                let end = rest.find('(').unwrap_or(rest.len());
                Some(&rest[..end])
            } else {
                None
            }
        }
        Language::TypeScript => {
            None // TS uses file-suffix convention, not inline markers
        }
    }
}

fn format_marker(suggestion: &TagSuggestion, lang: Language) -> String {
    let ref_str = suggestion.ref_value.as_deref().unwrap_or("");

    match lang {
        Language::Python => match suggestion.tag.as_str() {
            "scaffold" => "@pytest.mark.scaffold".to_string(),
            "decision" => {
                let reason = &suggestion.reason;
                format!("@pytest.mark.decision(reason=\"{reason}\")")
            }
            "contract" => format!("@pytest.mark.contract(party=\"{ref_str}\")"),
            "incident" => format!("@pytest.mark.incident(ref=\"{ref_str}\")"),
            _ => String::new(),
        },
        Language::Rust => match suggestion.tag.as_str() {
            "scaffold" => String::new(), // Rust uses mod scaffold, handled differently
            "decision" => format!("// kinhin: decision(ref=\"{ref_str}\")"),
            "contract" => format!("// kinhin: contract(ref=\"{ref_str}\")"),
            "incident" => format!("// kinhin: incident(ref=\"{ref_str}\")"),
            _ => String::new(),
        },
        Language::Java => match suggestion.tag.as_str() {
            "scaffold" => "@Tag(\"scaffold\")".to_string(),
            "decision" => format!("@Tag(\"decision\") // {}", suggestion.reason),
            "contract" => format!("@Tag(\"contract\") // {ref_str}"),
            "incident" => format!("@Tag(\"incident\") // {ref_str}"),
            _ => String::new(),
        },
        Language::TypeScript => String::new(), // TS uses file-suffix convention
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
