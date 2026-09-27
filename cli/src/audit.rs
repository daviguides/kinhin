//! `kinhin audit`: every test with its lifecycle tag, and whether each
//! permanent tag names an authority.

use std::path::Path;

use comfy_table::{Cell, CellAlignment, Color};
use owo_colors::OwoColorize;
use regex::Regex;

use crate::census::Census;
use crate::detect::{self, Language};
use crate::display;
use crate::env;
use crate::tags::{self, LifecycleTag, TaggedTest};
use crate::OutputFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RefStatus {
    /// Ticket, URL, or a path that exists.
    Linked,
    /// Free-text reason: valid, but nothing to check it against.
    Inline,
    /// Looks like a path, and the path does not exist.
    Broken,
    /// Permanent tag with no reason or ref at all.
    Missing,
    /// Temporary or untagged: no ref expected.
    NotNeeded,
}

impl RefStatus {
    fn icon(self) -> &'static str {
        match self {
            RefStatus::Linked => "✓",
            RefStatus::Inline => "·",
            RefStatus::Broken => "⚠",
            RefStatus::Missing => "✗",
            RefStatus::NotNeeded => "",
        }
    }

    fn color(self) -> Color {
        match self {
            RefStatus::Linked => Color::Green,
            RefStatus::Inline | RefStatus::NotNeeded => Color::DarkGrey,
            RefStatus::Broken => Color::Yellow,
            RefStatus::Missing => Color::Red,
        }
    }
}

pub fn classify_ref(tag: &LifecycleTag, reference: Option<&str>, root: &Path) -> RefStatus {
    if !tag.is_permanent() {
        return RefStatus::NotNeeded;
    }
    let Some(reference) = reference.map(str::trim).filter(|r| !r.is_empty()) else {
        return RefStatus::Missing;
    };
    if reference.contains("://") || Regex::new(r"^[A-Z][A-Z0-9]*-\d+\b").unwrap().is_match(reference) {
        return RefStatus::Linked;
    }
    let target = reference.split('#').next().unwrap_or(reference);
    let path_like = !target.contains(char::is_whitespace)
        && (target.contains('/') || Regex::new(r"\.(md|rst|txt|py|toml|ya?ml|json)$").unwrap().is_match(target));
    if !path_like {
        return RefStatus::Inline;
    }
    if root.join(target).exists() {
        RefStatus::Linked
    } else {
        RefStatus::Broken
    }
}

#[derive(serde::Serialize)]
struct AuditEntry {
    file: String,
    test: String,
    tag: String,
    #[serde(skip)]
    parsed_tag: LifecycleTag,
    #[serde(rename = "ref")]
    ref_value: Option<String>,
    ref_status: RefStatus,
}

#[derive(serde::Serialize)]
struct AuditReport<'a> {
    census: &'a Census,
    tests: &'a [AuditEntry],
}

fn entry_for(test: &TaggedTest, root: &Path) -> AuditEntry {
    AuditEntry {
        file: test.file.strip_prefix(root).unwrap_or(&test.file).display().to_string(),
        test: test.name.clone().unwrap_or_else(|| "(anonymous)".into()),
        tag: test.tag.to_string(),
        parsed_tag: test.tag.clone(),
        ref_value: test.ref_value.clone(),
        ref_status: classify_ref(&test.tag, test.ref_value.as_deref(), root),
    }
}

/// Exit code: 0 clean, 1 when a permanent tag has no authority or a broken ref.
pub fn run(path: &str, format: OutputFormat) -> i32 {
    let root = env::canonical_root(path);
    let mut languages = detect::detect_languages(path);
    if languages.is_empty() {
        display::print_warning("no language markers found: scanning for test files of every supported language");
        languages = vec![Language::Python, Language::Rust, Language::Java, Language::TypeScript];
    }

    let tests = tags::scan_test_files(&root.display().to_string(), &languages);
    let census = Census::from_tests(&tests);
    let entries: Vec<AuditEntry> = tests.iter().map(|t| entry_for(t, &root)).collect();
    let count = |status: RefStatus| entries.iter().filter(|e| e.ref_status == status).count();
    let (missing, broken, inline) = (count(RefStatus::Missing), count(RefStatus::Broken), count(RefStatus::Inline));

    match format {
        OutputFormat::Json => {
            let report = AuditReport {
                census: &census,
                tests: &entries,
            };
            println!("{}", serde_json::to_string_pretty(&report).expect("report serializes"));
        }
        OutputFormat::Rich => {
            if entries.is_empty() {
                display::print_warning("no tests found");
                return 0;
            }
            display_rich(&entries);
            println!();
            census.display(OutputFormat::Rich);
            println!();
            if inline > 0 {
                println!(
                    "  {} {inline} permanent test(s) carry an inline reason with no linkable ref",
                    "·".dimmed()
                );
            }
            if broken > 0 {
                display::print_warning(&format!("{broken} ref(s) point to a path that does not exist"));
            }
            if missing > 0 {
                display::print_error(&format!("{missing} permanent test(s) have no reason or ref"));
            }
            if missing + broken == 0 && census.untagged == 0 {
                display::print_success("every test is tagged and every permanent tag names its authority");
            }
        }
    }
    if missing + broken > 0 { 1 } else { 0 }
}

fn display_rich(entries: &[AuditEntry]) {
    let mut table = display::styled_table();
    table.set_header(vec![
        Cell::new("File").set_alignment(CellAlignment::Left),
        Cell::new("Test").set_alignment(CellAlignment::Left),
        Cell::new("Tag").set_alignment(CellAlignment::Left),
        Cell::new("Reason / ref").set_alignment(CellAlignment::Left),
        Cell::new("").set_alignment(CellAlignment::Center),
    ]);
    for entry in entries {
        let tag_color = match entry.parsed_tag {
            LifecycleTag::Scaffold | LifecycleTag::Characterization => Color::DarkGrey,
            LifecycleTag::Decision => Color::Green,
            LifecycleTag::Contract => Color::Blue,
            LifecycleTag::Incident => Color::Red,
            LifecycleTag::Untagged => Color::Yellow,
        };
        table.add_row(vec![
            Cell::new(&entry.file),
            Cell::new(&entry.test),
            Cell::new(format!("{} {}", display::tag_icon(&entry.parsed_tag), entry.tag)).fg(tag_color),
            Cell::new(entry.ref_value.as_deref().unwrap_or("—")),
            Cell::new(entry.ref_status.icon()).fg(entry.ref_status.color()),
        ]);
    }
    display::print_titled(&format!("{}", "Kinhin Audit".bold()), &table);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(tag: LifecycleTag, reference: Option<&str>) -> RefStatus {
        classify_ref(&tag, reference, Path::new(env!("CARGO_MANIFEST_DIR")))
    }

    #[test]
    fn temporary_and_untagged_tests_need_no_ref() {
        assert_eq!(status(LifecycleTag::Scaffold, None), RefStatus::NotNeeded);
        assert_eq!(status(LifecycleTag::Untagged, None), RefStatus::NotNeeded);
    }

    #[test]
    fn permanent_tag_without_text_is_missing() {
        assert_eq!(status(LifecycleTag::Decision, None), RefStatus::Missing);
        assert_eq!(status(LifecycleTag::Incident, Some("  ")), RefStatus::Missing);
    }

    #[test]
    fn free_text_reason_is_inline_not_a_broken_path() {
        assert_eq!(
            status(LifecycleTag::Decision, Some("discount is never negative, by billing contract")),
            RefStatus::Inline
        );
    }

    #[test]
    fn tickets_and_urls_are_linked() {
        assert_eq!(status(LifecycleTag::Incident, Some("ISSUE-186")), RefStatus::Linked);
        assert_eq!(status(LifecycleTag::Contract, Some("https://example.com/spec")), RefStatus::Linked);
    }

    #[test]
    fn path_refs_are_checked_on_disk() {
        assert_eq!(status(LifecycleTag::Decision, Some("Cargo.toml#package")), RefStatus::Linked);
        assert_eq!(status(LifecycleTag::Decision, Some("docs/nope.md#x")), RefStatus::Broken);
    }
}
