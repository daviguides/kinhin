//! `kinhin setup`: check (and for uv-managed Python projects, install)
//! what the other commands need.

use std::path::Path;
use std::process::{Command, Stdio};

use owo_colors::OwoColorize;

use crate::detect::{self, Language};
use crate::display;
use crate::env::{self, PyEnv};
use crate::pyproject;

struct PyDep {
    package: &'static str,
    module: &'static str,
    purpose: &'static str,
}

const PYTHON_DEPS: [PyDep; 5] = [
    PyDep { package: "pytest", module: "pytest", purpose: "test runner" },
    PyDep { package: "pytest-xdist", module: "xdist", purpose: "parallel runs" },
    PyDep { package: "pytest-randomly", module: "pytest_randomly", purpose: "randomized order" },
    PyDep { package: "pytest-testmon", module: "testmon", purpose: "changed-set selection (run --mode loop)" },
    PyDep { package: "mutmut", module: "mutmut", purpose: "mutation parity (gate, prune --verify)" },
];

struct PythonStatus {
    missing_packages: Vec<&'static str>,
    markers: Result<Vec<&'static str>, String>,
    mutmut_configured: bool,
}

impl PythonStatus {
    fn ready(&self) -> bool {
        self.missing_packages.is_empty() && self.markers.as_ref().is_ok_and(Vec::is_empty) && self.mutmut_configured
    }
}

fn ok(label: &str, detail: &str) {
    println!("    {} {label} {}", "✓".green(), detail.dimmed());
}

fn missing(label: &str, detail: &str) {
    println!("    {} {label} {}", "✗".red(), detail.dimmed());
}

fn python_status(py: &PyEnv, print: bool) -> PythonStatus {
    let modules: Vec<&str> = PYTHON_DEPS.iter().map(|d| d.module).collect();
    let available = py.modules_available(&modules);

    let mut missing_packages = Vec::new();
    for dep in &PYTHON_DEPS {
        if available[dep.module] {
            if print {
                ok(dep.package, dep.purpose);
            }
        } else {
            missing_packages.push(dep.package);
            if print {
                missing(dep.package, dep.purpose);
            }
        }
    }

    let markers = pyproject::markers_missing_in(&py.root);
    let mutmut_configured = pyproject::mutmut_configured(&py.root) || py.root.join("src").is_dir();
    if print {
        match &markers {
            Ok(list) if list.is_empty() => ok("pytest markers", "lifecycle markers registered"),
            Ok(list) => missing("pytest markers", &format!("not registered: {}", list.join(", "))),
            Err(reason) => missing("pytest markers", reason),
        }
        if mutmut_configured {
            ok("mutmut config", "source paths known");
        } else {
            missing("mutmut config", "no [tool.mutmut] source_paths");
        }
    }

    PythonStatus {
        missing_packages,
        markers,
        mutmut_configured,
    }
}

fn binary_check(label: &str, binary: &str, purpose: &str) -> bool {
    let found = env::binary_on_path(binary);
    if found {
        ok(label, purpose);
    } else {
        missing(label, purpose);
    }
    found
}

fn cargo_subcommand(name: &str) -> bool {
    Command::new("cargo")
        .args([name, "--version"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Checks for languages Kinhin does not install for. Returns readiness.
fn other_language_status(language: Language, root: &Path) -> bool {
    match language {
        Language::Python => true,
        Language::Rust => {
            let nextest = cargo_subcommand("nextest");
            if nextest {
                ok("cargo-nextest", "test runner");
            } else {
                missing("cargo-nextest", "test runner: cargo install cargo-nextest");
            }
            nextest
        }
        Language::Java => {
            let gradle = root.join("build.gradle").exists() || root.join("build.gradle.kts").exists();
            if gradle {
                binary_check("gradle", "gradle", "test runner")
            } else {
                binary_check("mvn", "mvn", "test runner")
            }
        }
        Language::TypeScript => binary_check(env::ts_package_runner(root), env::ts_package_runner(root), "package runner for jest/vitest"),
    }
}

fn install_python(py: &PyEnv) -> bool {
    let status = python_status(py, false);

    if !status.missing_packages.is_empty() {
        if !py.uv {
            display::print_error(&format!(
                "this project is not managed by uv; kinhin installs through uv only. Add these dev dependencies yourself: {}",
                status.missing_packages.join(" ")
            ));
        } else {
            println!("    {} uv add --dev {}", "$".dimmed(), status.missing_packages.join(" "));
            let added = Command::new("uv")
                .args(["add", "--dev"])
                .args(&status.missing_packages)
                .current_dir(&py.root)
                .stdin(Stdio::null())
                .status();
            if !added.is_ok_and(|s| s.success()) {
                display::print_error("`uv add --dev` failed; nothing else was changed");
                return false;
            }
        }
    }

    match pyproject::ensure_markers(&py.root) {
        Ok(added) if added.is_empty() => {}
        Ok(added) => println!(
            "    {} registered pytest markers in pyproject.toml: {}",
            "+".green(),
            added.join(", ")
        ),
        Err(reason) => display::print_warning(&format!("pytest markers not registered: {reason}")),
    }
    if !(pyproject::mutmut_configured(&py.root) || py.root.join("src").is_dir()) {
        match pyproject::ensure_mutmut(&py.root) {
            Ok(Some(paths)) => println!(
                "    {} wrote [tool.mutmut] source_paths = {:?} to pyproject.toml",
                "+".green(),
                paths
            ),
            Ok(None) => {}
            Err(reason) => display::print_warning(&format!("mutmut not configured: {reason}")),
        }
    }
    true
}

/// Returns the process exit code: 0 when everything is ready.
pub fn run(path: &str, install: bool) -> i32 {
    let root = env::canonical_root(path);
    let languages = detect::detect_languages(path);
    if languages.is_empty() {
        display::print_error("no language detected.");
        return 2;
    }

    let mut ready = true;
    for language in &languages {
        println!("  {}", language.to_string().bold());
        if *language == Language::Python {
            let py = PyEnv::detect(&root);
            println!(
                "    {} environment: {}",
                "·".dimmed(),
                if py.uv { "uv (commands run through `uv run`)" } else { "no uv project detected (PATH / .venv)" }
            );
            if install && !install_python(&py) {
                return 2;
            }
            ready &= python_status(&py, true).ready();
        } else {
            if install {
                display::print_warning(&format!(
                    "kinhin installs dependencies for uv-managed Python projects only; install {language} tooling yourself"
                ));
            }
            ready &= other_language_status(*language, &root);
        }
    }

    println!("  {}", "Agent sessions".bold());
    if !binary_check("claude", "claude", "Claude Code CLI, used by `kinhin tag`") {
        println!("      {}", "npm install -g @anthropic-ai/claude-code && claude login".dimmed());
        ready = false;
    }

    println!();
    if ready {
        display::print_success("ready");
        0
    } else {
        if !install && languages.contains(&Language::Python) {
            display::print_warning("not ready. Run: kinhin setup --install");
        } else {
            display::print_warning("not ready: the items marked ✗ need manual action");
        }
        1
    }
}
