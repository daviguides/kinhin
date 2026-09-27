use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use comfy_table::{Cell, Color};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::detect::{self, Language};
use crate::display;
use crate::OutputFormat;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationResult {
    pub killed: u32,
    pub survived: u32,
    pub timeout: u32,
    pub total: u32,
    pub per_file: HashMap<String, FileMutationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMutationResult {
    pub killed: u32,
    pub survived: u32,
    pub timeout: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateReport {
    pub result: MutationResult,
    pub baseline: Option<MutationResult>,
    pub verdict: GateVerdict,
    pub escaped: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateVerdict {
    Pass,
    Fail,
    NoBaseline,
}

struct MutationTool {
    name: &'static str,
    check_cmd: &'static str,
    check_arg: &'static str,
    install_hint: &'static str,
}

fn tool_for_language(lang: Language) -> MutationTool {
    match lang {
        Language::Python => MutationTool {
            name: "mutmut",
            check_cmd: "mutmut",
            check_arg: "--version",
            install_hint: "pip install mutmut",
        },
        Language::Rust => MutationTool {
            name: "cargo-mutants",
            check_cmd: "cargo",
            check_arg: "mutants",
            install_hint: "cargo install cargo-mutants",
        },
        Language::Java => MutationTool {
            name: "pitest",
            check_cmd: "mvn",
            check_arg: "--version",
            install_hint: "Add pitest plugin to pom.xml",
        },
        Language::TypeScript => MutationTool {
            name: "stryker",
            check_cmd: "npx",
            check_arg: "stryker",
            install_hint: "npm install --save-dev @stryker-mutator/core",
        },
    }
}

fn is_tool_available(tool: &MutationTool) -> bool {
    match tool.name {
        "cargo-mutants" => Command::new("cargo")
            .args(["mutants", "--version"])
            .output()
            .is_ok_and(|o| o.status.success()),
        _ => Command::new(tool.check_cmd)
            .arg(tool.check_arg)
            .output()
            .is_ok_and(|o| o.status.success()),
    }
}

fn get_changed_source_files(root: &str, lang: Language) -> Vec<String> {
    let patterns: &[&str] = match lang {
        Language::Python => &["*.py"],
        Language::Rust => &["*.rs"],
        Language::Java => &["*.java"],
        Language::TypeScript => &["*.ts", "*.tsx"],
    };

    let mut files = Vec::new();
    for pattern in patterns {
        if let Ok(output) = Command::new("git")
            .args(["diff", "--name-only", "main", "--", pattern])
            .current_dir(root)
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !is_test_file(trimmed, lang) {
                        files.push(trimmed.to_string());
                    }
                }
            }
        }
    }
    files
}

fn is_test_file(path: &str, lang: Language) -> bool {
    match lang {
        Language::Python => {
            let name = Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            name.starts_with("test_") || name.ends_with("_test.py")
        }
        Language::Rust => path.contains("/tests/") || path.ends_with("_test.rs"),
        Language::Java => {
            let name = Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            name.ends_with("Test.java") || name.ends_with("Tests.java")
        }
        Language::TypeScript => {
            path.contains(".test.") || path.contains(".spec.") || path.contains("__tests__")
        }
    }
}

fn run_mutation(root: &str, lang: Language, changed_files: &[String]) -> Option<MutationResult> {
    if changed_files.is_empty() {
        return Some(MutationResult {
            killed: 0,
            survived: 0,
            timeout: 0,
            total: 0,
            per_file: HashMap::new(),
        });
    }

    let output = match lang {
        Language::Python => {
            let paths = changed_files.join(",");
            Command::new("mutmut")
                .args(["run", "--paths-to-mutate", &paths, "--no-progress"])
                .current_dir(root)
                .output()
        }
        Language::Rust => Command::new("cargo")
            .args(["mutants", "--in-diff", "HEAD~1", "--json"])
            .current_dir(root)
            .output(),
        Language::Java => {
            let classes = changed_files
                .iter()
                .filter_map(|f| {
                    Path::new(f)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(String::from)
                })
                .collect::<Vec<_>>()
                .join(",");
            Command::new("mvn")
                .args([
                    "org.pitest:pitest-maven:mutationCoverage",
                    &format!("-DtargetClasses={classes}"),
                ])
                .current_dir(root)
                .output()
        }
        Language::TypeScript => {
            let mutate = changed_files
                .iter()
                .map(|f| format!("'{f}'"))
                .collect::<Vec<_>>()
                .join(",");
            Command::new("npx")
                .args(["stryker", "run", "--mutate", &mutate])
                .current_dir(root)
                .output()
        }
    };

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let combined = format!("{stdout}\n{stderr}");
            Some(parse_mutation_output(&combined, lang, changed_files))
        }
        Err(e) => {
            display::print_error(&format!("Failed to run mutation tool: {e}"));
            None
        }
    }
}

fn parse_mutation_output(
    output: &str,
    lang: Language,
    changed_files: &[String],
) -> MutationResult {
    let mut killed = 0u32;
    let mut survived = 0u32;
    let mut timeout = 0u32;

    match lang {
        Language::Python => {
            for line in output.lines() {
                let lower = line.to_lowercase();
                if lower.contains("killed") {
                    killed += extract_count(line);
                } else if lower.contains("survived") {
                    survived += extract_count(line);
                } else if lower.contains("timeout") {
                    timeout += extract_count(line);
                }
            }
        }
        Language::Rust => {
            for line in output.lines() {
                let lower = line.to_lowercase();
                if lower.contains("caught") || lower.contains("killed") {
                    killed += extract_count(line);
                } else if lower.contains("missed") || lower.contains("survived") {
                    survived += extract_count(line);
                } else if lower.contains("timeout") {
                    timeout += extract_count(line);
                }
            }
        }
        Language::Java | Language::TypeScript => {
            for line in output.lines() {
                let lower = line.to_lowercase();
                if lower.contains("killed") {
                    killed += extract_count(line);
                } else if lower.contains("survived") {
                    survived += extract_count(line);
                } else if lower.contains("timed out") || lower.contains("timeout") {
                    timeout += extract_count(line);
                }
            }
        }
    }

    let total = killed + survived + timeout;

    let mut per_file = HashMap::new();
    for file in changed_files {
        per_file.insert(
            file.clone(),
            FileMutationResult {
                killed: 0,
                survived: 0,
                timeout: 0,
            },
        );
    }

    MutationResult {
        killed,
        survived,
        timeout,
        total,
        per_file,
    }
}

fn extract_count(line: &str) -> u32 {
    line.split_whitespace()
        .find_map(|word| word.trim_matches(|c: char| !c.is_ascii_digit()).parse::<u32>().ok())
        .unwrap_or(0)
}

fn compare_results(current: &MutationResult, baseline: &MutationResult) -> (GateVerdict, Vec<String>) {
    if current.killed >= baseline.killed && current.survived <= baseline.survived {
        (GateVerdict::Pass, Vec::new())
    } else {
        let mut escaped = Vec::new();
        if current.killed < baseline.killed {
            escaped.push(format!(
                "Killed dropped: {} (was {}) — {} mutant(s) lost",
                current.killed,
                baseline.killed,
                baseline.killed - current.killed
            ));
        }
        if current.survived > baseline.survived {
            escaped.push(format!(
                "Survived increased: {} (was {}) — {} mutant(s) escaped",
                current.survived,
                baseline.survived,
                current.survived - baseline.survived
            ));
        }
        (GateVerdict::Fail, escaped)
    }
}

fn display_report(report: &GateReport, output: OutputFormat) {
    match output {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(report).unwrap_or_default());
        }
        OutputFormat::Rich => {
            display_rich(report);
        }
    }
}

fn display_rich(report: &GateReport) {
    println!();

    let mut table = display::styled_table();
    table.set_header(vec![
        Cell::new("Metric"),
        Cell::new("Count"),
        Cell::new(""),
    ]);

    let r = &report.result;
    table.add_row(vec![
        Cell::new("Killed").fg(Color::Green),
        Cell::new(r.killed),
        Cell::new(bar(r.killed, r.total)).fg(Color::Green),
    ]);
    table.add_row(vec![
        Cell::new("Survived").fg(Color::Red),
        Cell::new(r.survived),
        Cell::new(bar(r.survived, r.total)).fg(Color::Red),
    ]);
    table.add_row(vec![
        Cell::new("Timeout").fg(Color::Yellow),
        Cell::new(r.timeout),
        Cell::new(bar(r.timeout, r.total)).fg(Color::Yellow),
    ]);
    table.add_row(vec![
        Cell::new("Total"),
        Cell::new(r.total),
        Cell::new(""),
    ]);

    display::print_titled("Mutation Results", &table);
    println!();

    match report.verdict {
        GateVerdict::Pass => {
            println!(
                "  {} {}",
                "✓".green().bold(),
                "K₁ ⊇ K₀ — PASS".green().bold()
            );
        }
        GateVerdict::Fail => {
            println!(
                "  {} {}",
                "✗".red().bold(),
                "K₁ ⊉ K₀ — FAIL".red().bold()
            );
            for escaped in &report.escaped {
                println!("    {} {escaped}", "→".red());
            }
        }
        GateVerdict::NoBaseline => {
            println!(
                "  {} {}",
                "○".dimmed(),
                "No baseline — results recorded, no comparison".dimmed()
            );
        }
    }
    println!();

    if let Some(baseline) = &report.baseline {
        let mut cmp_table = display::styled_table();
        cmp_table.set_header(vec![
            Cell::new(""),
            Cell::new("Baseline (K₀)"),
            Cell::new("Current (K₁)"),
            Cell::new("Delta"),
        ]);
        cmp_table.add_row(vec![
            Cell::new("Killed"),
            Cell::new(baseline.killed),
            Cell::new(r.killed),
            Cell::new(delta_str(r.killed, baseline.killed)),
        ]);
        cmp_table.add_row(vec![
            Cell::new("Survived"),
            Cell::new(baseline.survived),
            Cell::new(r.survived),
            Cell::new(delta_str(r.survived, baseline.survived)),
        ]);

        display::print_titled("Baseline Comparison", &cmp_table);
        println!();
    }
}

fn bar(count: u32, total: u32) -> String {
    if total == 0 {
        return String::new();
    }
    let width = 20;
    let filled = ((count as f64 / total as f64) * width as f64).round() as usize;
    let empty = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

fn delta_str(current: u32, baseline: u32) -> String {
    match current.cmp(&baseline) {
        std::cmp::Ordering::Greater => format!("+{}", current - baseline),
        std::cmp::Ordering::Less => format!("-{}", baseline - current),
        std::cmp::Ordering::Equal => "=".to_string(),
    }
}

fn load_baseline(path: &Path) -> Option<MutationResult> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_baseline(path: &Path, result: &MutationResult) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(result)?;
    std::fs::write(path, json)
}

pub fn run(
    path: &str,
    lang_override: Option<Language>,
    baseline_path: Option<&str>,
    save_baseline_path: Option<&str>,
    output: OutputFormat,
) {
    let languages = match lang_override {
        Some(l) => vec![l],
        None => detect::detect_languages(path),
    };

    if languages.is_empty() {
        display::print_error("No language detected. Use --lang to specify.");
        std::process::exit(2);
    }

    let lang = languages[0];
    let tool = tool_for_language(lang);

    if !is_tool_available(&tool) {
        display::print_error(&format!(
            "{} not found. Install: {}",
            tool.name, tool.install_hint
        ));
        std::process::exit(2);
    }

    let changed_files = get_changed_source_files(path, lang);

    if changed_files.is_empty() {
        display::print_warning("No changed source files found (compared to main). Nothing to mutate.");
        std::process::exit(0);
    }

    println!(
        "  {} Running {} on {} changed file(s)...",
        "⧗".dimmed(),
        tool.name,
        changed_files.len()
    );
    for f in &changed_files {
        println!("    {f}");
    }
    println!();

    let result = match run_mutation(path, lang, &changed_files) {
        Some(r) => r,
        None => {
            display::print_error("Mutation testing failed.");
            std::process::exit(1);
        }
    };

    if let Some(save_path) = save_baseline_path {
        match save_baseline(Path::new(save_path), &result) {
            Ok(()) => display::print_success(&format!("Baseline saved to {save_path}")),
            Err(e) => display::print_error(&format!("Failed to save baseline: {e}")),
        }
    }

    let baseline = baseline_path.and_then(|p| {
        let b = load_baseline(Path::new(p));
        if b.is_none() {
            display::print_warning(&format!("Could not load baseline from {p}"));
        }
        b
    });

    let (verdict, escaped) = match &baseline {
        Some(b) => compare_results(&result, b),
        None => (GateVerdict::NoBaseline, Vec::new()),
    };

    let report = GateReport {
        result,
        baseline,
        verdict,
        escaped,
    };

    display_report(&report, output);

    match report.verdict {
        GateVerdict::Pass | GateVerdict::NoBaseline => std::process::exit(0),
        GateVerdict::Fail => std::process::exit(1),
    }
}
