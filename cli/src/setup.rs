use std::process::{Command, Stdio};

use owo_colors::OwoColorize;

use crate::detect::{self, Language};
use crate::display;

struct Dep {
    name: &'static str,
    check: CheckKind,
    install: &'static str,
    required: bool,
}

enum CheckKind {
    PythonImport(&'static str),
    Binary(&'static str),
    NpmPkg(&'static str),
}

fn deps_for(lang: Language) -> Vec<Dep> {
    match lang {
        Language::Python => vec![
            Dep { name: "pytest", check: CheckKind::PythonImport("pytest"), install: "uv add --dev pytest", required: true },
            Dep { name: "pytest-xdist", check: CheckKind::PythonImport("xdist"), install: "uv add --dev pytest-xdist", required: false },
            Dep { name: "pytest-randomly", check: CheckKind::PythonImport("randomly"), install: "uv add --dev pytest-randomly", required: false },
            Dep { name: "pytest-testmon", check: CheckKind::PythonImport("testmon"), install: "uv add --dev pytest-testmon", required: false },
            Dep { name: "mutmut", check: CheckKind::PythonImport("mutmut"), install: "uv add --dev mutmut", required: false },
        ],
        Language::Rust => vec![
            Dep { name: "cargo-nextest", check: CheckKind::Binary("cargo-nextest"), install: "cargo install cargo-nextest", required: true },
            Dep { name: "cargo-mutants", check: CheckKind::Binary("cargo-mutants"), install: "cargo install cargo-mutants", required: false },
        ],
        Language::Java => vec![
            Dep { name: "mvn", check: CheckKind::Binary("mvn"), install: "brew install maven", required: true },
        ],
        Language::TypeScript => vec![
            Dep { name: "jest", check: CheckKind::NpmPkg("jest"), install: "npm install --save-dev jest", required: false },
            Dep { name: "vitest", check: CheckKind::NpmPkg("vitest"), install: "npm install --save-dev vitest", required: false },
            Dep { name: "@stryker-mutator/core", check: CheckKind::NpmPkg("@stryker-mutator/core"), install: "npm install --save-dev @stryker-mutator/core", required: false },
        ],
    }
}

fn is_available(dep: &Dep) -> bool {
    match &dep.check {
        CheckKind::PythonImport(module) => Command::new("python")
            .args(["-c", &format!("import {module}")])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success()),
        CheckKind::Binary(name) => Command::new("which")
            .arg(name)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success()),
        CheckKind::NpmPkg(name) => Command::new("npx")
            .args(["--no", name, "--version"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success()),
    }
}

pub fn check(path: &str) -> bool {
    let languages = detect::detect_languages(path);
    if languages.is_empty() {
        display::print_error("No language detected.");
        return false;
    }

    let mut all_ok = true;

    for lang in &languages {
        println!("{}", format!("  {lang}").bold());
        let deps = deps_for(*lang);
        for dep in &deps {
            let available = is_available(dep);
            if available {
                println!("    {} {}", "✓".green(), dep.name);
            } else if dep.required {
                println!("    {} {} {}", "✗".red(), dep.name, "(required)".red());
                all_ok = false;
            } else {
                println!("    {} {} {}", "○".yellow(), dep.name, "(optional)".dimmed());
            }
        }
    }

    all_ok
}

pub fn install(path: &str) {
    let languages = detect::detect_languages(path);
    if languages.is_empty() {
        display::print_error("No language detected.");
        std::process::exit(1);
    }

    for lang in &languages {
        let deps = deps_for(*lang);
        let missing: Vec<&Dep> = deps.iter().filter(|d| !is_available(d)).collect();

        if missing.is_empty() {
            display::print_success(&format!("{lang}: all dependencies installed"));
            continue;
        }

        println!("{}", format!("  {lang}: installing {} dep(s)...", missing.len()).bold());

        for dep in &missing {
            println!("    {} {}", "⧗".dimmed(), dep.install);
            let parts: Vec<&str> = dep.install.split_whitespace().collect();
            let status = Command::new(parts[0])
                .args(&parts[1..])
                .current_dir(path)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status();

            match status {
                Ok(s) if s.success() => {
                    println!("    {} {}", "✓".green(), dep.name);
                }
                Ok(s) => {
                    display::print_error(&format!("{} failed (exit {})", dep.name, s.code().unwrap_or(-1)));
                }
                Err(e) => {
                    display::print_error(&format!("{} failed: {e}", dep.install));
                }
            }
        }
    }
}

#[allow(dead_code)]
pub fn missing_for_run(_path: &str, lang: Language) -> Vec<String> {
    let deps = deps_for(lang);
    deps.iter()
        .filter(|d| !is_available(d))
        .map(|d| format!("{} → {}", d.name, d.install))
        .collect()
}
