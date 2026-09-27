//! `kinhin tag`: classify untagged tests in one agent session and write
//! the lifecycle markers into the test files.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use comfy_table::{Cell, CellAlignment, Color};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::agent::{self, TagSuggestion};
use crate::census::Census;
use crate::detect::{self, Language};
use crate::display;
use crate::env::{self, PyEnv};
use crate::pycheck;
use crate::pyproject;
use crate::pytests::{self, Marker};
use crate::tags::{self, LifecycleTag};
use crate::OutputFormat;

const CACHE_SCHEMA: u32 = 2;
const CACHE_FILE: &str = "tag-suggestions.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileSuggestions {
    /// Path relative to the project root.
    file: String,
    suggestions: Vec<TagSuggestion>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Cache {
    schema_version: u32,
    model: String,
    files: Vec<FileSuggestions>,
}

#[derive(Debug, Serialize)]
struct TagReport {
    untagged_found: usize,
    files: Vec<FileSuggestions>,
    applied: bool,
    markers_written: usize,
}

fn relative(path: &Path, root: &Path) -> String {
    let absolute = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    absolute.strip_prefix(root).unwrap_or(&absolute).display().to_string()
}

fn load_cache(path: &Path) -> Option<Cache> {
    let cache: Cache = serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
    (cache.schema_version == CACHE_SCHEMA).then_some(cache)
}

/// The marker to write for one suggestion.
fn marker_for(suggestion: &TagSuggestion) -> Marker {
    let reason = Some(suggestion.reason.trim()).filter(|r| !r.is_empty());
    let reference = suggestion.ref_value.as_deref().map(str::trim).filter(|r| !r.is_empty() && *r != "null");
    match suggestion.tag.as_str() {
        "decision" => Marker {
            tag: LifecycleTag::Decision,
            text: reason.or(reference).map(String::from),
        },
        "contract" => Marker {
            tag: LifecycleTag::Contract,
            text: reference.or(reason).map(String::from),
        },
        // An incident without a ticket has no authority to point at: keep it as a decision.
        "incident" => match reference {
            Some(r) => Marker {
                tag: LifecycleTag::Incident,
                text: Some(r.to_string()),
            },
            None => Marker {
                tag: LifecycleTag::Decision,
                text: reason.map(String::from),
            },
        },
        _ => Marker {
            tag: LifecycleTag::Scaffold,
            text: None,
        },
    }
}

/// Exit code: 0 done, 1 could not complete (nothing left half-written), 2 usage/tool error.
pub async fn run(path: &str, dry_run: bool, force: bool, model: &str, format: OutputFormat) -> i32 {
    let root = env::canonical_root(path);
    let languages = detect::detect_languages(path);
    let Some(&language) = languages.first() else {
        display::print_error("no language detected.");
        return 2;
    };
    if !dry_run && language != Language::Python {
        display::print_error(&format!(
            "writing tags is implemented for Python only ({language} detected). Use --dry-run to see the suggestions."
        ));
        return 2;
    }

    let tests = tags::scan_test_files(&root.display().to_string(), &languages);
    let mut untagged: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for test in tests.iter().filter(|t| t.tag == LifecycleTag::Untagged) {
        untagged
            .entry(relative(&test.file, &root))
            .or_default()
            .push(test.name.clone().unwrap_or_default());
    }
    let untagged_count: usize = untagged.values().map(Vec::len).sum();
    if untagged_count == 0 {
        if format == OutputFormat::Json {
            print_json(&TagReport {
                untagged_found: 0,
                files: Vec::new(),
                applied: false,
                markers_written: 0,
            });
        } else {
            display::print_success(&format!("all {} test(s) already tagged: nothing to do", tests.len()));
        }
        return 0;
    }

    let kinhin_dir = root.join(".kinhin");
    let cache_path = kinhin_dir.join(CACHE_FILE);
    let cached: HashMap<String, Vec<TagSuggestion>> = if dry_run || force {
        HashMap::new()
    } else {
        load_cache(&cache_path)
            .map(|cache| cache.files.into_iter().map(|f| (f.file, f.suggestions)).collect())
            .unwrap_or_default()
    };

    let mut results: Vec<FileSuggestions> = Vec::new();
    let mut pending: Vec<(&String, &Vec<String>)> = Vec::new();
    for (file, ids) in &untagged {
        match cached.get(file) {
            Some(suggestions) if ids.iter().all(|id| suggestions.iter().any(|s| s.test == *id)) => {
                results.push(FileSuggestions {
                    file: file.clone(),
                    suggestions: suggestions.iter().filter(|s| ids.contains(&s.test)).cloned().collect(),
                });
            }
            _ => pending.push((file, ids)),
        }
    }
    if !results.is_empty() {
        eprintln!(
            "{} {} file(s) from cached suggestions ({})",
            "→".cyan(),
            results.len(),
            cache_path.strip_prefix(&root).unwrap_or(&cache_path).display()
        );
    }

    let mut failed_files = 0;
    if !pending.is_empty() {
        let pending_tests: usize = pending.iter().map(|(_, ids)| ids.len()).sum();
        eprintln!(
            "{} classifying {pending_tests} untagged test(s) in {} file(s), one session, model {model}",
            "→".cyan(),
            pending.len()
        );
        let mut session = match agent::TaggerSession::connect(language, model, &kinhin_dir).await {
            Ok(session) => session,
            Err(e) => {
                display::print_error(&format!("could not start the agent session: {e}"));
                return 2;
            }
        };
        let mut closed: Vec<agent::SessionMetrics> = Vec::new();
        for (file, ids) in pending {
            let Ok(content) = std::fs::read_to_string(root.join(file)) else {
                display::print_error(&format!("cannot read {file}"));
                failed_files += 1;
                continue;
            };
            eprintln!("  {} {file} ({} tests)", "·".cyan(), ids.len());
            let mut outcome = session.tag_file(file, &content, ids).await;
            if let Err(first_error) = &outcome {
                // The CLI process behind a session can die mid-run: reconnect once and retry this file.
                display::print_warning(&format!("{file}: {first_error}; reconnecting and retrying once"));
                match agent::TaggerSession::connect(language, model, &kinhin_dir).await {
                    Ok(fresh) => {
                        let dead = std::mem::replace(&mut session, fresh);
                        if let Ok(metrics) = dead.disconnect().await {
                            closed.push(metrics);
                        }
                        outcome = session.tag_file(file, &content, ids).await;
                    }
                    Err(e) => display::print_error(&format!("could not reconnect: {e}")),
                }
            }
            match outcome {
                Ok(suggestions) => results.push(FileSuggestions {
                    file: file.clone(),
                    suggestions,
                }),
                Err(e) => {
                    display::print_error(&format!("{file}: {e}"));
                    failed_files += 1;
                }
            }
        }
        match session.disconnect().await {
            Ok(metrics) => closed.push(metrics),
            Err(e) => display::print_warning(&format!("session did not close cleanly: {e}")),
        }
        eprintln!(
            "  {} {} session(s): {} file(s), {} turn(s), ${:.4}",
            "⧗".dimmed(),
            closed.len(),
            closed.iter().map(|m| m.files_processed).sum::<u32>(),
            closed.iter().map(|m| m.total_turns).sum::<u32>(),
            closed.iter().map(|m| m.total_cost_usd).sum::<f64>(),
        );
        for metrics in &closed {
            if let Some(log) = &metrics.log_path {
                eprintln!("    log {}", log.strip_prefix(&root).unwrap_or(log).display());
            }
        }
    }
    results.sort_by(|a, b| a.file.cmp(&b.file));

    // Save what we have: a failed file does not cost the others their work.
    let cache = Cache {
        schema_version: CACHE_SCHEMA,
        model: model.to_string(),
        files: results.clone(),
    };
    let saved = std::fs::create_dir_all(&kinhin_dir).is_ok()
        && std::fs::write(&cache_path, serde_json::to_string_pretty(&cache).expect("cache serializes")).is_ok();
    if !saved {
        display::print_warning(&format!("could not save suggestions to {}", cache_path.display()));
    }

    if format == OutputFormat::Rich {
        display_rich(&results);
    }

    if failed_files > 0 {
        display::print_error(&format!(
            "{failed_files} file(s) could not be classified; nothing was written. Run `kinhin tag` again: classified files are cached."
        ));
        return 1;
    }

    if dry_run {
        if format == OutputFormat::Json {
            print_json(&TagReport {
                untagged_found: untagged_count,
                files: results,
                applied: false,
                markers_written: 0,
            });
        } else {
            println!(
                "\n  {} dry run: nothing written. Run `kinhin tag` to apply these suggestions (no new session).",
                "→".dimmed()
            );
        }
        return 0;
    }

    match apply(&root, &results) {
        Ok(written) => {
            let _ = std::fs::remove_file(&cache_path);
            if format == OutputFormat::Json {
                print_json(&TagReport {
                    untagged_found: untagged_count,
                    files: results,
                    applied: true,
                    markers_written: written,
                });
            } else {
                let after = Census::from_tests(&tags::scan_test_files(&root.display().to_string(), &languages));
                display::print_success(&format!(
                    "{written} marker(s) written in {} file(s). Suite collects. Census: {}",
                    results.len(),
                    after.census_line()
                ));
            }
            0
        }
        Err(message) => {
            display::print_error(&message);
            1
        }
    }
}

/// Write markers for every suggestion, then prove the suite still collects
/// and that no test was left untagged. Any failure restores every file.
fn apply(root: &Path, results: &[FileSuggestions]) -> Result<usize, String> {
    let py = PyEnv::detect(root);
    let mut originals: Vec<(PathBuf, String)> = Vec::new();
    let mut with_new_import: Vec<PathBuf> = Vec::new();
    let mut written = 0;

    let restore = |originals: &[(PathBuf, String)]| {
        for (path, content) in originals {
            let _ = std::fs::write(path, content);
        }
    };

    let pyproject_path = root.join("pyproject.toml");
    let mut width = pytests::DEFAULT_LINE_WIDTH;
    if let Ok(content) = std::fs::read_to_string(&pyproject_path) {
        width = pyproject::line_length(&content).unwrap_or(width);
        originals.push((pyproject_path, content));
    }
    if let Err(reason) = pyproject::ensure_markers(root) {
        return Err(format!("cannot register the lifecycle markers with pytest: {reason}"));
    }

    for file in results {
        let path = root.join(&file.file);
        let content = std::fs::read_to_string(&path).map_err(|e| {
            restore(&originals);
            format!("cannot read {}: {e}", file.file)
        })?;
        let markers: HashMap<String, Marker> =
            file.suggestions.iter().map(|s| (s.test.clone(), marker_for(s))).collect();
        let outcome = pytests::insert_markers(&content, &markers, width);
        if outcome.inserted == 0 {
            continue;
        }
        originals.push((path.clone(), content));
        if let Err(e) = std::fs::write(&path, &outcome.content) {
            restore(&originals);
            return Err(format!("cannot write {}: {e}", file.file));
        }
        written += outcome.inserted;
        if outcome.added_import {
            with_new_import.push(path);
        }
    }

    // `import pytest` was appended to the import block: let the project's
    // own ruff put it where its isort rules want it.
    pycheck::ruff_fix(&py, &with_new_import, "I");

    if let Err(reason) = pycheck::collect(&py) {
        restore(&originals);
        return Err(format!("{reason}\n  every file was restored; no tag was written."));
    }

    let still_untagged = tags::scan_test_files(&root.display().to_string(), &[Language::Python])
        .iter()
        .filter(|t| t.tag == LifecycleTag::Untagged)
        .count();
    if still_untagged > 0 {
        return Err(format!(
            "{written} marker(s) written, but {still_untagged} test(s) are still untagged. Run `kinhin tag --force`."
        ));
    }
    Ok(written)
}

fn print_json(report: &TagReport) {
    println!("{}", serde_json::to_string_pretty(report).expect("report serializes"));
}

fn display_rich(results: &[FileSuggestions]) {
    if results.is_empty() {
        return;
    }
    let mut table = display::styled_table();
    table.set_header(vec![
        Cell::new("File").set_alignment(CellAlignment::Left),
        Cell::new("Test").set_alignment(CellAlignment::Left),
        Cell::new("Tag").set_alignment(CellAlignment::Left),
        Cell::new("Reason").set_alignment(CellAlignment::Left),
        Cell::new("Ref").set_alignment(CellAlignment::Left),
    ]);
    for file in results {
        for suggestion in &file.suggestions {
            let marker = marker_for(suggestion);
            let color = match marker.tag {
                LifecycleTag::Decision => Color::Green,
                LifecycleTag::Contract => Color::Blue,
                LifecycleTag::Incident => Color::Red,
                _ => Color::DarkGrey,
            };
            table.add_row(vec![
                Cell::new(&file.file),
                Cell::new(&suggestion.test),
                Cell::new(format!("{} {}", display::tag_icon(&marker.tag), marker.tag)).fg(color),
                Cell::new(&suggestion.reason),
                Cell::new(suggestion.ref_value.as_deref().unwrap_or("—")),
            ]);
        }
    }
    println!();
    display::print_titled(&format!("{}", "Tag Suggestions".bold()), &table);

    let total: usize = results.iter().map(|f| f.suggestions.len()).sum();
    let scaffolds = results
        .iter()
        .flat_map(|f| &f.suggestions)
        .filter(|s| marker_for(s).tag == LifecycleTag::Scaffold)
        .count();
    println!(
        "\n  {total} suggestion(s): {} scaffold, {} permanent",
        scaffolds.to_string().dimmed(),
        (total - scaffolds).to_string().green(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn suggestion(tag: &str, reason: &str, reference: Option<&str>) -> TagSuggestion {
        TagSuggestion {
            test: "t".into(),
            tag: tag.into(),
            reason: reason.into(),
            ref_value: reference.map(String::from),
        }
    }

    #[test]
    fn incident_without_a_ticket_is_written_as_a_decision() {
        let marker = marker_for(&suggestion("incident", "crashed on empty file", None));
        assert_eq!(marker.tag, LifecycleTag::Decision);
        assert_eq!(marker.text.as_deref(), Some("crashed on empty file"));
    }

    #[test]
    fn contract_prefers_the_named_consumer_over_the_reason() {
        let marker = marker_for(&suggestion("contract", "pins output", Some("CLI users")));
        assert_eq!(marker.text.as_deref(), Some("CLI users"));
        let fallback = marker_for(&suggestion("contract", "pins output", Some("null")));
        assert_eq!(fallback.text.as_deref(), Some("pins output"));
    }

    #[test]
    fn unknown_tag_becomes_scaffold() {
        assert_eq!(marker_for(&suggestion("keep", "", None)).tag, LifecycleTag::Scaffold);
    }
}
