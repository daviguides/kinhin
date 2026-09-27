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

pub fn tag_colored(tag: &LifecycleTag) -> String {
    match tag {
        LifecycleTag::Scaffold => format!("{} scaffold", "◇".dimmed()),
        LifecycleTag::Characterization => format!("{} characterization", "◈".dimmed()),
        LifecycleTag::Decision => format!("{} decision", "◆".green()),
        LifecycleTag::Contract => format!("{} contract", "◆".blue()),
        LifecycleTag::Incident => format!("{} incident", "⚡".red()),
        LifecycleTag::Untagged => format!("{} untagged", "?".yellow()),
    }
}

pub fn status_icon(valid: Option<bool>) -> &'static str {
    match valid {
        Some(true) => "✓",
        Some(false) => "✗",
        None => "○",
    }
}

pub fn status_colored(valid: Option<bool>) -> String {
    match valid {
        Some(true) => format!("{}", "✓".green()),
        Some(false) => format!("{}", "✗".red()),
        None => format!("{}", "○".dimmed()),
    }
}

pub fn print_success(message: &str) {
    println!("{} {message}", "✓".green());
}

pub fn print_warning(message: &str) {
    println!("{} {message}", "⚠".yellow());
}

pub fn print_error(message: &str) {
    eprintln!("{} {message}", "✗".red());
}

pub fn print_titled(title: &str, table: &Table) {
    let rendered = table.to_string();
    let width = rendered.lines().next().map_or(0, |l| l.chars().count());
    let visible_len = title.chars().count();
    let pad = width.saturating_sub(visible_len) / 2;
    println!("{:pad$}{title}", "", pad = pad);
    println!("{rendered}");
}
