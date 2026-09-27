//! Terminal output helpers. Tables and results go to stdout; progress,
//! warnings and errors go to stderr so `--output json` stays parseable.

use comfy_table::presets::UTF8_FULL_CONDENSED;
use comfy_table::{ContentArrangement, Table, TableComponent};
use owo_colors::OwoColorize;

use crate::tags::LifecycleTag;

pub fn styled_table() -> Table {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_style(TableComponent::VerticalLines, '│');
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table
}

pub fn tag_icon(tag: &LifecycleTag) -> &'static str {
    match tag {
        LifecycleTag::Scaffold => "◇",
        LifecycleTag::Characterization => "◈",
        LifecycleTag::Decision => "◆",
        LifecycleTag::Contract => "◆",
        LifecycleTag::Incident => "⚡",
        LifecycleTag::Untagged => "?",
    }
}

pub fn print_success(message: &str) {
    eprintln!("{} {message}", "✓".green());
}

pub fn print_warning(message: &str) {
    eprintln!("{} {message}", "⚠".yellow());
}

pub fn print_error(message: &str) {
    eprintln!("{} {message}", "✗".red());
}

pub fn print_titled(title: &str, table: &Table) {
    let rendered = table.to_string();
    let width = rendered.lines().next().map_or(0, |l| l.chars().count());
    // `title` may carry ANSI styling: measure what is visible.
    let visible: usize = {
        let mut count = 0;
        let mut in_escape = false;
        for c in title.chars() {
            match (in_escape, c) {
                (false, '\u{1b}') => in_escape = true,
                (true, 'm') => in_escape = false,
                (false, _) => count += 1,
                _ => {}
            }
        }
        count
    };
    let pad = width.saturating_sub(visible) / 2;
    println!("{:pad$}{title}", "", pad = pad);
    println!("{rendered}");
}
