//! Mutation parity gate (K₁ ⊇ K₀) on top of mutmut ≥ 3.
//!
//! K is the SET of killed mutant ids. A baseline run is saved to a file;
//! a later run passes only if every mutant killed in the baseline is still
//! killed. Tool failure is an error, never a pass.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Stdio;

use comfy_table::{Cell, CellAlignment, Color};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::detect::{self, Language};
use crate::display;
use crate::env::{self, PyEnv};
use crate::pyproject;
use crate::OutputFormat;

pub const BASELINE_SCHEMA: u32 = 1;

pub const EXIT_PARITY_LOST: i32 = 1;
pub const EXIT_TOOL_ERROR: i32 = 2;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MutationRun {
    pub schema_version: u32,
    pub tool: String,
    /// Mutant-name globs the run was restricted to (empty = everything).
    pub scope: Vec<String>,
    /// Mutant id → mutmut status (`killed`, `survived`, `segfault`, ...).
    pub statuses: BTreeMap<String, String>,
}

impl MutationRun {
    pub fn killed(&self) -> BTreeSet<&str> {
        self.statuses
            .iter()
            .filter(|(_, status)| status.as_str() == "killed")
            .map(|(id, _)| id.as_str())
            .collect()
    }

    pub fn counts(&self) -> BTreeMap<String, usize> {
        let mut counts = BTreeMap::new();
        for status in self.statuses.values() {
            *counts.entry(status.clone()).or_default() += 1;
        }
        counts
    }

    pub fn checked(&self) -> usize {
        self.statuses.values().filter(|s| s.as_str() != "not checked").count()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Escaped {
    pub mutant: String,
    /// Status in the current run, or `absent` when the mutant no longer exists.
    pub now: String,
}

/// Parse `mutmut results --all true`: one `    <mutant id>: <status>` per line.
pub fn parse_results(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .filter_map(|line| line.trim().split_once(": "))
        .filter(|(id, _)| id.contains("__mutmut_") && !id.contains(' '))
        .map(|(id, status)| (id.to_string(), status.trim().to_string()))
        .collect()
}

/// Mutants killed in the baseline that are not killed now.
pub fn escaped(baseline: &MutationRun, current: &MutationRun) -> Vec<Escaped> {
    baseline
        .killed()
        .into_iter()
        .filter_map(|id| match current.statuses.get(id).map(String::as_str) {
            Some("killed") => None,
            Some(status) => Some(Escaped {
                mutant: id.to_string(),
                now: status.to_string(),
            }),
            None => Some(Escaped {
                mutant: id.to_string(),
                now: "absent".to_string(),
            }),
        })
        .collect()
}

/// `pkg.mod.x_func__mutmut_3` / `pkg.mod.xǁClassǁmethod__mutmut_3` → `pkg.mod`.
pub fn module_of(mutant: &str) -> &str {
    let function = mutant.rsplit_once("__mutmut_").map_or(mutant, |(f, _)| f);
    [".x_", ".xǁ"]
        .iter()
        .filter_map(|sep| function.find(sep))
        .min()
        .map_or(function, |at| &function[..at])
}

/// The mangled function a mutant belongs to (key into mutmut's stats).
pub fn function_of(mutant: &str) -> &str {
    mutant.rsplit_once("__mutmut_").map_or(mutant, |(f, _)| f)
}

fn clean_tool_output(raw: &str) -> Vec<String> {
    raw.replace('\r', "\n")
        .lines()
        .map(str::trim_end)
        .filter(|l| !l.trim().is_empty())
        .filter(|l| !l.contains("Generating mutants") && !l.contains("Running stats") && !l.contains("🎉"))
        .map(String::from)
        .collect()
}

/// Run mutmut from a clean state and collect every mutant's status.
pub fn run_mutmut(py: &PyEnv, scope: &[String]) -> Result<MutationRun, String> {
    if !py.modules_available(&["mutmut"])["mutmut"] {
        return Err("mutmut is not installed in the project environment. Run: kinhin setup --install".into());
    }
    if !pyproject::mutmut_configured(&py.root) && !py.root.join("src").is_dir() {
        return Err("mutmut has no `source_paths` configured. Run: kinhin setup --install".into());
    }

    // Results from a previous run describe a different test suite: start clean.
    let mutants_dir = py.root.join("mutants");
    if mutants_dir.exists() {
        std::fs::remove_dir_all(&mutants_dir).map_err(|e| format!("cannot clear {}: {e}", mutants_dir.display()))?;
    }

    let mut command = py.tool("mutmut");
    command.arg("run").args(scope);
    if cfg!(target_os = "macos") {
        // mutmut forks its workers; urllib's macOS proxy lookup (SystemConfiguration)
        // is not fork-safe and segfaults the child. Skipping the lookup avoids it.
        command.env("NO_PROXY", "*").env("no_proxy", "*");
    }
    let output = command
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("cannot launch mutmut: {e}"))?;

    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if !output.status.success() {
        let lines = clean_tool_output(&combined);
        let tail = lines[lines.len().saturating_sub(12)..].join("\n    ");
        let headline = if combined.contains("failed to collect stats") {
            "mutmut could not run: the test suite is not green. Mutation testing needs a passing suite."
        } else {
            "mutmut failed."
        };
        return Err(format!(
            "{headline}\n  exit code {}; last output:\n    {tail}",
            output.status.code().map_or("signal".to_string(), |c| c.to_string())
        ));
    }

    let results = py
        .tool("mutmut")
        .args(["results", "--all", "true"])
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("cannot launch mutmut results: {e}"))?;
    if !results.status.success() {
        return Err(format!(
            "`mutmut results` failed:\n    {}",
            clean_tool_output(&String::from_utf8_lossy(&results.stderr)).join("\n    ")
        ));
    }

    let run = MutationRun {
        schema_version: BASELINE_SCHEMA,
        tool: "mutmut".to_string(),
        scope: scope.to_vec(),
        statuses: parse_results(&String::from_utf8_lossy(&results.stdout)),
    };
    if run.statuses.is_empty() {
        return Err("mutmut generated no mutants. Check `[tool.mutmut] source_paths`.".into());
    }
    if run.checked() == 0 {
        return Err("mutmut checked no mutant (does --scope match any mutant name?).".into());
    }
    Ok(run)
}

pub fn load_baseline(path: &Path) -> Result<MutationRun, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("cannot read baseline {}: {e}", path.display()))?;
    let run: MutationRun = serde_json::from_str(&content).map_err(|e| {
        format!(
            "{} is not a kinhin baseline ({e}). Record a new one with --save-baseline.",
            path.display()
        )
    })?;
    if run.schema_version != BASELINE_SCHEMA {
        return Err(format!(
            "baseline schema {} is not supported (expected {BASELINE_SCHEMA})",
            run.schema_version
        ));
    }
    if run.killed().is_empty() {
        return Err(format!("baseline {} has no killed mutant: nothing to compare against", path.display()));
    }
    Ok(run)
}

#[derive(Debug, Serialize)]
struct ModuleCounts {
    killed: usize,
    survived: usize,
    other: usize,
}

#[derive(Debug, Serialize)]
struct GateReport {
    verdict: &'static str,
    total: usize,
    counts: BTreeMap<String, usize>,
    modules: BTreeMap<String, ModuleCounts>,
    scope: Vec<String>,
    baseline_killed: Option<usize>,
    escaped: Vec<Escaped>,
}

fn build_report(run: &MutationRun, baseline: Option<&MutationRun>) -> GateReport {
    let mut modules: BTreeMap<String, ModuleCounts> = BTreeMap::new();
    for (id, status) in &run.statuses {
        let entry = modules.entry(module_of(id).to_string()).or_insert(ModuleCounts {
            killed: 0,
            survived: 0,
            other: 0,
        });
        match status.as_str() {
            "killed" => entry.killed += 1,
            "survived" => entry.survived += 1,
            _ => entry.other += 1,
        }
    }
    let escaped = baseline.map(|b| escaped(b, run)).unwrap_or_default();
    GateReport {
        verdict: match baseline {
            None => "no_baseline",
            Some(_) if escaped.is_empty() => "pass",
            Some(_) => "fail",
        },
        total: run.statuses.len(),
        counts: run.counts(),
        modules,
        scope: run.scope.clone(),
        baseline_killed: baseline.map(|b| b.killed().len()),
        escaped,
    }
}

fn status_color(status: &str) -> Color {
    match status {
        "killed" => Color::Green,
        "survived" => Color::Red,
        "not checked" => Color::DarkGrey,
        _ => Color::Yellow,
    }
}

fn display_rich(report: &GateReport, saved_baseline: bool) {
    let mut table = display::styled_table();
    table.set_header(vec![
        Cell::new("Status"),
        Cell::new("Mutants").set_alignment(CellAlignment::Right),
    ]);
    for (status, count) in &report.counts {
        table.add_row(vec![
            Cell::new(status).fg(status_color(status)),
            Cell::new(count).set_alignment(CellAlignment::Right),
        ]);
    }
    table.add_row(vec![
        Cell::new("total"),
        Cell::new(report.total).set_alignment(CellAlignment::Right),
    ]);
    println!();
    display::print_titled(&format!("{}", "Mutation Results".bold()), &table);

    let mut by_module = display::styled_table();
    by_module.set_header(vec![
        Cell::new("Module"),
        Cell::new("Killed").set_alignment(CellAlignment::Right),
        Cell::new("Survived").set_alignment(CellAlignment::Right),
        Cell::new("Other").set_alignment(CellAlignment::Right),
    ]);
    for (module, counts) in &report.modules {
        if counts.killed + counts.survived + counts.other == 0 {
            continue;
        }
        by_module.add_row(vec![
            Cell::new(module),
            Cell::new(counts.killed).fg(Color::Green).set_alignment(CellAlignment::Right),
            Cell::new(counts.survived).fg(Color::Red).set_alignment(CellAlignment::Right),
            Cell::new(counts.other).fg(Color::Yellow).set_alignment(CellAlignment::Right),
        ]);
    }
    println!();
    display::print_titled(&format!("{}", "By Module".bold()), &by_module);
    println!();

    let unstable: usize = report
        .counts
        .iter()
        .filter(|(status, _)| matches!(status.as_str(), "segfault" | "timeout" | "suspicious"))
        .map(|(_, count)| count)
        .sum();
    if unstable > 0 {
        display::print_warning(&format!(
            "{unstable} mutant(s) ended as segfault/timeout/suspicious: reported on their own, never counted as killed"
        ));
    }

    match report.verdict {
        "pass" => println!(
            "  {} {}",
            "✓".green().bold(),
            format!(
                "K₁ ⊇ K₀: all {} baseline-killed mutants are still killed",
                report.baseline_killed.unwrap_or(0)
            )
            .green()
            .bold()
        ),
        "fail" => {
            println!(
                "  {} {}",
                "✗".red().bold(),
                format!(
                    "K₁ ⊉ K₀: {} of {} baseline-killed mutants are no longer killed",
                    report.escaped.len(),
                    report.baseline_killed.unwrap_or(0)
                )
                .red()
                .bold()
            );
            const SHOWN: usize = 40;
            for item in report.escaped.iter().take(SHOWN) {
                println!("    {} {}  {}", "→".red(), item.mutant, format!("(now {})", item.now).dimmed());
            }
            if report.escaped.len() > SHOWN {
                println!("    … {} more (use --output json for the full list)", report.escaped.len() - SHOWN);
            }
        }
        _ if saved_baseline => println!("  {} {}", "○".dimmed(), "baseline recorded: compare a later run with --baseline".dimmed()),
        _ => println!("  {} {}", "○".dimmed(), "no baseline given: results reported, nothing compared".dimmed()),
    }
    println!();
}

pub fn run(path: &str, baseline_path: Option<&str>, save_baseline_path: Option<&str>, scope: &[String], output: OutputFormat) {
    let root = env::canonical_root(path);
    if !detect::detect_languages(path).contains(&Language::Python) {
        display::print_error("kinhin gate supports Python projects (mutmut ≥ 3) only; no Python project detected here.");
        std::process::exit(EXIT_TOOL_ERROR);
    }
    let py = PyEnv::detect(&root);

    let baseline = match baseline_path.map(|p| load_baseline(Path::new(p))) {
        Some(Ok(run)) => Some(run),
        Some(Err(message)) => {
            display::print_error(&message);
            std::process::exit(EXIT_TOOL_ERROR);
        }
        None => None,
    };
    if let Some(b) = &baseline {
        if b.scope != scope {
            display::print_error(&format!(
                "baseline was recorded with scope {:?}, this run uses {:?}: the two are not comparable",
                b.scope, scope
            ));
            std::process::exit(EXIT_TOOL_ERROR);
        }
    }

    eprintln!(
        "  {} running mutmut{} (clean run)...",
        "⧗".dimmed(),
        if scope.is_empty() { String::new() } else { format!(" on {}", scope.join(" ")) }
    );
    let current = match run_mutmut(&py, scope) {
        Ok(run) => run,
        Err(message) => {
            display::print_error(&message);
            std::process::exit(EXIT_TOOL_ERROR);
        }
    };

    if let Some(save_path) = save_baseline_path {
        if current.killed().is_empty() {
            display::print_error("no mutant was killed: refusing to save an empty baseline");
            std::process::exit(EXIT_TOOL_ERROR);
        }
        let json = serde_json::to_string_pretty(&current).expect("baseline serializes");
        if let Err(e) = std::fs::write(save_path, json) {
            display::print_error(&format!("cannot write baseline {save_path}: {e}"));
            std::process::exit(EXIT_TOOL_ERROR);
        }
        eprintln!("  {} baseline saved to {save_path} ({} killed mutants)", "✓".green(), current.killed().len());
    }

    let report = build_report(&current, baseline.as_ref());
    match output {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report).expect("report serializes")),
        OutputFormat::Rich => display_rich(&report, save_baseline_path.is_some()),
    }
    if report.verdict == "fail" {
        std::process::exit(EXIT_PARITY_LOST);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Captured verbatim from `mutmut results --all true` (mutmut 3.8.0).
    const REAL_RESULTS: &str = include_str!("../tests/fixtures/mutmut-3.8-results-all.txt");

    fn run_from(text: &str) -> MutationRun {
        MutationRun {
            schema_version: BASELINE_SCHEMA,
            tool: "mutmut".into(),
            scope: Vec::new(),
            statuses: parse_results(text),
        }
    }

    #[test]
    fn parses_every_mutant_of_a_real_mutmut_report() {
        let run = run_from(REAL_RESULTS);
        assert_eq!(run.statuses.len(), 634);
        assert_eq!(run.counts()["killed"], 505);
        assert_eq!(run.counts()["survived"], 129);
        assert_eq!(run.statuses["marks.display.x_print_success__mutmut_1"], "survived");
    }

    #[test]
    fn statuses_with_spaces_survive_parsing() {
        let run = run_from("    a.x_f__mutmut_1: not checked\n    a.x_f__mutmut_2: no tests\n    a.x_f__mutmut_3: segfault\n");
        assert_eq!(run.statuses["a.x_f__mutmut_1"], "not checked");
        assert_eq!(run.statuses["a.x_f__mutmut_2"], "no tests");
        assert_eq!(run.checked(), 2);
        assert!(run.killed().is_empty(), "segfault must never count as killed");
    }

    #[test]
    fn tool_noise_is_not_parsed_as_a_mutant() {
        let run = run_from("UserWarning: something: odd\nFAILED tests/x.py::test_a - assert: False\n");
        assert!(run.statuses.is_empty());
    }

    #[test]
    fn identical_runs_have_parity() {
        let run = run_from(REAL_RESULTS);
        assert!(escaped(&run, &run).is_empty());
    }

    #[test]
    fn a_killed_mutant_that_survives_later_is_named() {
        let baseline = run_from(REAL_RESULTS);
        let flipped = REAL_RESULTS.replacen(
            "marks.display.x_print_error__mutmut_1: killed",
            "marks.display.x_print_error__mutmut_1: survived",
            1,
        );
        let lost = escaped(&baseline, &run_from(&flipped));
        assert_eq!(
            lost,
            [Escaped {
                mutant: "marks.display.x_print_error__mutmut_1".into(),
                now: "survived".into()
            }]
        );
    }

    #[test]
    fn same_killed_count_with_different_members_is_not_parity() {
        let baseline = run_from("    m.x_a__mutmut_1: killed\n    m.x_a__mutmut_2: survived\n");
        let current = run_from("    m.x_a__mutmut_1: survived\n    m.x_a__mutmut_2: killed\n");
        assert_eq!(escaped(&baseline, &current).len(), 1);
    }

    #[test]
    fn killed_turning_into_segfault_or_vanishing_breaks_parity() {
        let baseline = run_from("    m.x_a__mutmut_1: killed\n    m.x_a__mutmut_2: killed\n");
        let current = run_from("    m.x_a__mutmut_1: segfault\n");
        let lost = escaped(&baseline, &current);
        assert_eq!(lost[0].now, "segfault");
        assert_eq!(lost[1].now, "absent");
    }

    #[test]
    fn newly_killed_mutants_do_not_matter_for_parity() {
        let baseline = run_from("    m.x_a__mutmut_1: killed\n    m.x_a__mutmut_2: survived\n");
        let current = run_from("    m.x_a__mutmut_1: killed\n    m.x_a__mutmut_2: killed\n");
        assert!(escaped(&baseline, &current).is_empty());
    }

    #[test]
    fn module_is_derived_for_functions_and_methods() {
        assert_eq!(module_of("marks.services.check.x_run_check__mutmut_12"), "marks.services.check");
        assert_eq!(
            module_of("marks.integrations.importers.chrome.xǁ_ChromeHTMLParserǁ__init____mutmut_1"),
            "marks.integrations.importers.chrome"
        );
        assert_eq!(
            function_of("marks.services.check.x_run_check__mutmut_12"),
            "marks.services.check.x_run_check"
        );
    }
}
