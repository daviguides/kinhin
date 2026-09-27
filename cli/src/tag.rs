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

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct FileSuggestions {
    file: String,
    suggestions: Vec<TagSuggestion>,
}

const SUGGESTIONS_PATH: &str = ".kinhin/tag-suggestions.json";

pub async fn run(
    path: &str,
    lang: Option<Language>,
    apply: bool,
    force: bool,
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

    let root = Path::new(path)
        .canonicalize()
        .unwrap_or_else(|_| Path::new(path).to_path_buf());
    let primary_lang = languages[0];
    let cache_path = Path::new(SUGGESTIONS_PATH);

    // Try cache first (unless --force or --dry-run which always runs fresh)
    let all_suggestions: Vec<FileSuggestions> = if !force && apply && cache_path.exists() {
        println!("{} loading cached suggestions from {SUGGESTIONS_PATH}", "→".cyan());
        match std::fs::read_to_string(cache_path) {
            Ok(json) => match serde_json::from_str(&json) {
                Ok(s) => s,
                Err(e) => {
                    display::print_warning(&format!("cache corrupt ({e}), running fresh session"));
                    run_session(path, &languages, model, &root).await
                }
            },
            Err(_) => run_session(path, &languages, model, &root).await,
        }
    } else {
        run_session(path, &languages, model, &root).await
    };

    if all_suggestions.is_empty() {
        display::print_success("no suggestions generated");
        return;
    }

    // Always save to cache
    if let Ok(json) = serde_json::to_string_pretty(&all_suggestions) {
        let _ = std::fs::create_dir_all(".kinhin");
        let _ = std::fs::write(cache_path, &json);
    }

    // Display
    match format {
        OutputFormat::Json => display_json(&all_suggestions),
        OutputFormat::Rich => display_rich(&all_suggestions),
    }

    // Apply unless --dry-run
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
                display::print_warning(&format!("cannot read {}, skipping", file_sugg.file));
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

        // Clear cache after successful apply
        let _ = std::fs::remove_file(cache_path);
    } else {
        println!("\n  {} dry run — suggestions saved to {SUGGESTIONS_PATH}", "→".dimmed());
        println!("  {} run `kinhin tag` to apply", "→".dimmed());
    }
}

async fn run_session(
    path: &str,
    languages: &[Language],
    model: Option<&str>,
    root: &Path,
) -> Vec<FileSuggestions> {
    let tests = tags::scan_test_files(path, languages);
    let untagged: Vec<&TaggedTest> = tests
        .iter()
        .filter(|t| t.tag == LifecycleTag::Untagged)
        .collect();

    if untagged.is_empty() {
        display::print_success("all tests already tagged — nothing to do");
        return Vec::new();
    }

    println!(
        "{} {} untagged test(s) across {} file(s)",
        "→".cyan(),
        untagged.len(),
        untagged.iter().map(|t| &t.file).collect::<std::collections::HashSet<_>>().len(),
    );

    let mut files_map: std::collections::BTreeMap<String, Vec<&TaggedTest>> =
        std::collections::BTreeMap::new();
    for test in &untagged {
        files_map.entry(test.file.display().to_string()).or_default().push(test);
    }

    let primary_lang = languages[0];
    let mut all_suggestions: Vec<FileSuggestions> = Vec::new();

    let mut session = match agent::TaggerSession::connect(primary_lang, model).await {
        Ok(s) => s,
        Err(e) => {
            display::print_error(&format!("failed to start agent session: {e}"));
            std::process::exit(1);
        }
    };

    for (file_path, _) in &files_map {
        let Ok(content) = std::fs::read_to_string(file_path) else {
            display::print_warning(&format!("cannot read {file_path}, skipping"));
            continue;
        };

        let short = shorten(file_path, root);
        println!("  {} {short}", "⠋".cyan());

        match session.tag_file(&content).await {
            Ok(suggestions) => {
                all_suggestions.push(FileSuggestions { file: short, suggestions });
            }
            Err(e) => {
                display::print_error(&format!("agent failed on {short}: {e}"));
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

    all_suggestions
}

fn display_json(suggestions: &[FileSuggestions]) {
    let total: usize = suggestions.iter().map(|f| f.suggestions.len()).sum();
    let report = TagReport {
        files_scanned: suggestions.len(),
        untagged_found: total,
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
    let needs_pytest_markers = lang == Language::Python && !suggestions.is_empty();

    if needs_pytest_markers && !content.contains("import pytest") {
        let lines: Vec<&str> = content.lines().collect();
        let mut result: Vec<String> = Vec::with_capacity(lines.len() + 2);
        let mut insert_idx = 0;
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
                insert_idx = i + 1;
            }
        }
        for (i, line) in lines.iter().enumerate() {
            if i == insert_idx {
                result.push("import pytest".to_string());
                result.push(String::new());
            }
            result.push(line.to_string());
        }
        let new_content = result.join("\n");
        return insert_tags_inner(&new_content, suggestions, lang);
    }

    insert_tags_inner(content, suggestions, lang)
}

fn insert_tags_inner(content: &str, suggestions: &[TagSuggestion], lang: Language) -> String {
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
