use std::path::Path;

use comfy_table::{Cell, CellAlignment, Color};
use owo_colors::OwoColorize;

use crate::census::Census;
use crate::detect;
use crate::display;
use crate::tags::{self, LifecycleTag, TaggedTest};
use crate::OutputFormat;

#[derive(serde::Serialize)]
struct AuditEntry {
    file: String,
    test: String,
    tag: String,
    ref_value: Option<String>,
    ref_status: RefStatus,
}

#[derive(Clone, Copy, serde::Serialize)]
enum RefStatus {
    Valid,
    Missing,
    NotNeeded,
    Required,
}

impl RefStatus {
    fn icon(&self) -> &'static str {
        match self {
            RefStatus::Valid => "✓",
            RefStatus::Missing => "⚠",
            RefStatus::NotNeeded => "○",
            RefStatus::Required => "✗",
        }
    }

    fn color(&self) -> Color {
        match self {
            RefStatus::Valid => Color::Green,
            RefStatus::Missing => Color::Yellow,
            RefStatus::NotNeeded => Color::DarkGrey,
            RefStatus::Required => Color::Red,
        }
    }
}

fn check_ref(test: &TaggedTest, root: &Path) -> RefStatus {
    if test.tag.is_temporary() || test.tag == LifecycleTag::Untagged {
        return RefStatus::NotNeeded;
    }

    match &test.ref_value {
        Some(ref_val) => {
            let ref_path = root.join(ref_val);
            if ref_path.exists() {
                RefStatus::Valid
            } else if ref_val.starts_with("ISSUE-")
                || ref_val.starts_with("TRYPLAT-")
                || ref_val.starts_with("TICKET-")
                || ref_val.contains("://")
            {
                RefStatus::Valid
            } else {
                RefStatus::Missing
            }
        }
        None => RefStatus::Required,
    }
}

fn shorten_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

pub fn run(path: &str, format: OutputFormat) {
    let root = Path::new(path).canonicalize().unwrap_or_else(|_| Path::new(path).to_path_buf());
    let languages = detect::detect_languages(path);

    if languages.is_empty() {
        display::print_warning("no language markers found — scanning all test-like files");
    } else {
        let lang_list: Vec<String> = languages.iter().map(|l| l.to_string()).collect();
        tracing::info!("detected: {}", lang_list.join(", "));
    }

    let effective_languages = if languages.is_empty() {
        vec![
            detect::Language::Python,
            detect::Language::Rust,
            detect::Language::Java,
            detect::Language::TypeScript,
        ]
    } else {
        languages
    };

    let tests = tags::scan_test_files(path, &effective_languages);

    if tests.is_empty() {
        display::print_warning("no tagged tests found");
        return;
    }

    let census = Census::from_tests(&tests);

    let entries: Vec<AuditEntry> = tests
        .iter()
        .map(|t| {
            let ref_status = check_ref(t, &root);
            AuditEntry {
                file: shorten_path(&t.file, &root),
                test: t.name.clone().unwrap_or_else(|| "(anonymous)".into()),
                tag: t.tag.to_string(),
                ref_value: t.ref_value.clone(),
                ref_status,
            }
        })
        .collect();

    match format {
        OutputFormat::Json => display_json(&entries, &census),
        OutputFormat::Rich => display_rich(&entries, &census, &root),
    }
}

fn display_json(entries: &[AuditEntry], census: &Census) {
    #[derive(serde::Serialize)]
    struct AuditReport<'a> {
        census: &'a Census,
        tests: &'a [AuditEntry],
    }

    let report = AuditReport {
        census,
        tests: entries,
    };
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}

fn display_rich(entries: &[AuditEntry], census: &Census, _root: &Path) {
    let mut table = display::styled_table();
    table.set_header(vec![
        Cell::new("File").set_alignment(CellAlignment::Left),
        Cell::new("Test").set_alignment(CellAlignment::Left),
        Cell::new("Tag").set_alignment(CellAlignment::Left),
        Cell::new("Ref").set_alignment(CellAlignment::Left),
        Cell::new("").set_alignment(CellAlignment::Center),
    ]);

    for entry in entries {
        let tag_color = match entry.tag.as_str() {
            "scaffold" | "characterization" => Color::DarkGrey,
            "decision" => Color::Green,
            "contract" => Color::Blue,
            "incident" => Color::Red,
            _ => Color::Yellow,
        };

        let tag_icon = match entry.tag.as_str() {
            "scaffold" => "◇",
            "characterization" => "◈",
            "decision" => "◆",
            "contract" => "◆",
            "incident" => "⚡",
            _ => "?",
        };

        let ref_display = entry
            .ref_value
            .as_deref()
            .unwrap_or("—");

        table.add_row(vec![
            Cell::new(&entry.file),
            Cell::new(&entry.test),
            Cell::new(format!("{tag_icon} {}", entry.tag)).fg(tag_color),
            Cell::new(ref_display),
            Cell::new(entry.ref_status.icon()).fg(entry.ref_status.color()),
        ]);
    }

    display::print_titled(
        &format!("{}", "Kinhin Audit".bold()),
        &table,
    );

    println!();
    census.display(OutputFormat::Rich);

    let issues: Vec<&AuditEntry> = entries
        .iter()
        .filter(|e| matches!(e.ref_status, RefStatus::Missing | RefStatus::Required))
        .collect();

    if !issues.is_empty() {
        println!();
        let missing_refs = entries
            .iter()
            .filter(|e| matches!(e.ref_status, RefStatus::Missing))
            .count();
        let no_refs = entries
            .iter()
            .filter(|e| matches!(e.ref_status, RefStatus::Required))
            .count();

        if missing_refs > 0 {
            display::print_warning(&format!(
                "{missing_refs} ref(s) point to missing files"
            ));
        }
        if no_refs > 0 {
            display::print_error(&format!(
                "{no_refs} permanent test(s) have no ref"
            ));
        }
    } else if census.untagged == 0 {
        println!();
        display::print_success("all tests tagged, all refs valid");
    }
}
