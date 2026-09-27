use owo_colors::OwoColorize;

use crate::display;
use crate::tags::{LifecycleTag, TaggedTest};
use crate::OutputFormat;

#[derive(Debug, Default, serde::Serialize)]
pub struct Census {
    pub total: usize,
    pub scaffold: usize,
    pub characterization: usize,
    pub decision: usize,
    pub contract: usize,
    pub incident: usize,
    pub untagged: usize,
}

impl Census {
    pub fn from_tests(tests: &[TaggedTest]) -> Self {
        let mut c = Census::default();
        c.total = tests.len();
        for t in tests {
            match &t.tag {
                LifecycleTag::Scaffold => c.scaffold += 1,
                LifecycleTag::Characterization => c.characterization += 1,
                LifecycleTag::Decision => c.decision += 1,
                LifecycleTag::Contract => c.contract += 1,
                LifecycleTag::Incident => c.incident += 1,
                LifecycleTag::Untagged => c.untagged += 1,
            }
        }
        c
    }

    pub fn permanent_count(&self) -> usize {
        self.decision + self.contract + self.incident
    }

    pub fn temporary_count(&self) -> usize {
        self.scaffold + self.characterization
    }

    pub fn census_line(&self) -> String {
        format!(
            "total {}: {} scaffold, {} decision, {} contract, {} incident, {} untagged",
            self.total,
            self.scaffold + self.characterization,
            self.decision,
            self.contract,
            self.incident,
            self.untagged,
        )
    }

    pub fn display(&self, format: OutputFormat) {
        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(self).unwrap());
            }
            OutputFormat::Rich => {
                self.display_rich();
            }
        }
    }

    fn display_rich(&self) {
        use comfy_table::{Cell, CellAlignment, Color};

        let mut table = display::styled_table();
        table.set_header(vec![
            Cell::new("Tag").set_alignment(CellAlignment::Left),
            Cell::new("Count").set_alignment(CellAlignment::Right),
            Cell::new("").set_alignment(CellAlignment::Left),
        ]);

        let rows: Vec<(&str, &str, usize, Color)> = vec![
            ("◇", "scaffold", self.scaffold + self.characterization, Color::DarkGrey),
            ("◆", "decision", self.decision, Color::Green),
            ("◆", "contract", self.contract, Color::Blue),
            ("⚡", "incident", self.incident, Color::Red),
            ("?", "untagged", self.untagged, Color::Yellow),
        ];

        for (icon, label, count, color) in rows {
            if count > 0 {
                table.add_row(vec![
                    Cell::new(format!("{icon} {label}")).fg(color),
                    Cell::new(count.to_string())
                        .set_alignment(CellAlignment::Right)
                        .fg(color),
                    Cell::new(bar(count, self.total)).fg(color),
                ]);
            }
        }

        table.add_row(vec![
            Cell::new("total").add_attribute(comfy_table::Attribute::Bold),
            Cell::new(self.total.to_string())
                .set_alignment(CellAlignment::Right)
                .add_attribute(comfy_table::Attribute::Bold),
            Cell::new(""),
        ]);

        display::print_titled(&format!("{}", "Test Census".bold()), "Test Census".len(), &table);

        if self.untagged > 0 {
            display::print_warning(&format!(
                "{} untagged test(s) — classify before prune",
                self.untagged
            ));
        }
    }
}

fn bar(count: usize, total: usize) -> String {
    if total == 0 {
        return String::new();
    }
    let width = 20;
    let filled = (count * width) / total;
    let empty = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}
