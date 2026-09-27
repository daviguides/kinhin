use std::process::{Command, Stdio};

use clap::ValueEnum;
use owo_colors::OwoColorize;

use crate::detect::{self, Language};
use crate::display;

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum RunMode {
    /// Changed-set only, parallel, randomized, no bail
    Loop,
    /// Full suite excluding scaffolds, pre-PR gate
    Full,
    /// Serial, fixed order, isolate a flake
    Diagnostic,
}

impl std::fmt::Display for RunMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunMode::Loop => write!(f, "loop"),
            RunMode::Full => write!(f, "full"),
            RunMode::Diagnostic => write!(f, "diagnostic"),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum TsRunner {
    Jest,
    Vitest,
}

pub fn run(
    mode: RunMode,
    lang: Option<Language>,
    ts_runner: Option<TsRunner>,
    path: &str,
    extra_args: &[String],
) {
    let languages = match lang {
        Some(l) => vec![l],
        None => detect::detect_languages(path),
    };

    if languages.is_empty() {
        display::print_error("No language detected. Use --lang to specify.");
        std::process::exit(1);
    }

    if languages.len() > 1 {
        display::print_warning(&format!(
            "Multiple languages detected: {}. Running first: {}",
            languages.iter().map(|l| l.to_string()).collect::<Vec<_>>().join(", "),
            languages[0],
        ));
    }

    let language = languages[0];
    let (program, args) = build_command(language, mode, ts_runner, extra_args);

    print_header(mode, language, &program, &args);

    let status = Command::new(&program)
        .args(&args)
        .current_dir(path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(s) => {
            let code = s.code().unwrap_or(1);
            print_result(code);
            std::process::exit(code);
        }
        Err(e) => {
            display::print_error(&format!("Failed to execute `{program}`: {e}"));
            std::process::exit(127);
        }
    }
}

fn build_command(
    language: Language,
    mode: RunMode,
    ts_runner: Option<TsRunner>,
    extra: &[String],
) -> (String, Vec<String>) {
    let mut args: Vec<String> = Vec::new();

    let program = match language {
        Language::Python => {
            args.extend(["--maxfail=0".into(), "-ra".into(), "-q".into()]);

            let has_plugin = |name: &str| -> bool {
                Command::new("python")
                    .args(["-c", &format!("import {name}")])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .is_ok_and(|s| s.success())
            };

            let has_xdist = has_plugin("xdist");
            let has_randomly = has_plugin("randomly");
            let has_testmon = has_plugin("testmon");

            if has_xdist {
                args.extend(["-n".into(), "auto".into()]);
            }
            if has_randomly {
                args.extend(["-p".into(), "randomly".into()]);
            }

            match mode {
                RunMode::Loop => {
                    if has_testmon {
                        args.push("--testmon".into());
                    }
                }
                RunMode::Full => args.extend(["-m".into(), "not scaffold".into()]),
                RunMode::Diagnostic => {
                    args = vec!["--maxfail=0".into(), "-ra".into(), "-q".into()];
                    if has_randomly {
                        args.extend(["-p".into(), "no:randomly".into()]);
                    }
                    if has_xdist {
                        args.extend(["-p".into(), "no:xdist".into()]);
                    }
                    args.push("-x".into());
                }
            }

            let mut missing: Vec<&str> = Vec::new();
            if !has_xdist { missing.push("pytest-xdist"); }
            if !has_randomly { missing.push("pytest-randomly"); }
            if !has_testmon && mode == RunMode::Loop { missing.push("pytest-testmon"); }
            if !missing.is_empty() {
                display::print_warning(&format!(
                    "Optional plugins not found: {}. Run: kinhin setup --install",
                    missing.join(", "),
                ));
            }

            "pytest".to_string()
        }

        Language::Rust => {
            args.extend(["nextest".into(), "run".into(), "--no-fail-fast".into()]);
            match mode {
                RunMode::Loop => {} // user can pass `-p <crate>` via extra args
                RunMode::Full => {
                    args.extend([
                        "--filter-expr".into(),
                        "not test(scaffold::)".into(),
                        "--profile".into(),
                        "ci".into(),
                    ]);
                }
                RunMode::Diagnostic => args.push("-j=1".into()),
            }
            "cargo".to_string()
        }

        Language::Java => {
            let is_gradle = std::path::Path::new("build.gradle").exists()
                || std::path::Path::new("build.gradle.kts").exists();

            if is_gradle {
                args.extend(["test".into(), "--continue".into()]);
                match mode {
                    RunMode::Loop => {}
                    RunMode::Full => {
                        args.push("-DexcludedGroups=scaffold".into());
                    }
                    RunMode::Diagnostic => {
                        args.push("-Djunit.jupiter.execution.parallel.enabled=false".into());
                    }
                }
                "gradle".to_string()
            } else {
                args.extend([
                    "test".into(),
                    "-Dmaven.test.failure.ignore=true".into(),
                ]);
                match mode {
                    RunMode::Loop => {}
                    RunMode::Full => {
                        args.push("-DexcludedGroups=scaffold".into());
                    }
                    RunMode::Diagnostic => {
                        args.push("-Djunit.jupiter.execution.parallel.enabled=false".into());
                    }
                }
                "mvn".to_string()
            }
        }

        Language::TypeScript => {
            let runner = ts_runner.unwrap_or_else(|| detect_ts_runner());
            match runner {
                TsRunner::Jest => {
                    args.extend([
                        "--no-bail".into(),
                        "--maxWorkers=50%".into(),
                        "--randomize".into(),
                    ]);
                    match mode {
                        RunMode::Loop => args.push("--onlyChanged".into()),
                        RunMode::Full => {
                            args.extend([
                                "--testPathIgnorePatterns".into(),
                                "scaffold".into(),
                            ]);
                        }
                        RunMode::Diagnostic => args.push("--runInBand".into()),
                    }
                    "jest".to_string()
                }
                TsRunner::Vitest => {
                    args.extend(["run".into(), "--reporter=json".into()]);
                    match mode {
                        RunMode::Loop => args.push("--changed".into()),
                        RunMode::Full => {
                            args.extend([
                                "--exclude".into(),
                                "**/**.scaffold.test.ts".into(),
                            ]);
                        }
                        RunMode::Diagnostic => {
                            args.extend([
                                "--pool".into(), "forks".into(),
                                "--poolOptions.forks.singleFork".into(),
                            ]);
                        }
                    }
                    "vitest".to_string()
                }
            }
        }
    };

    args.extend(extra.iter().cloned());
    (program, args)
}

fn detect_ts_runner() -> TsRunner {
    if std::path::Path::new("vitest.config.ts").exists()
        || std::path::Path::new("vitest.config.js").exists()
        || std::path::Path::new("vitest.config.mts").exists()
    {
        TsRunner::Vitest
    } else {
        TsRunner::Jest
    }
}

fn print_header(mode: RunMode, language: Language, program: &str, args: &[String]) {
    let mode_badge = match mode {
        RunMode::Loop => format!("{}", " LOOP ".on_cyan().bold()),
        RunMode::Full => format!("{}", " FULL ".on_green().bold()),
        RunMode::Diagnostic => format!("{}", " DIAG ".on_yellow().bold()),
    };

    let mode_icon = match mode {
        RunMode::Loop => "🔄",
        RunMode::Full => "🛡",
        RunMode::Diagnostic => "🔍",
    };

    let cmd_str = format!("{program} {}", args.join(" "));

    println!();
    println!("{mode_icon} {mode_badge}  {}  {}", language.to_string().bold(), mode_description(mode).dimmed());
    println!("  {} {}", "$".dimmed(), cmd_str);
    println!();
}

fn mode_description(mode: RunMode) -> &'static str {
    match mode {
        RunMode::Loop => "changed-set, parallel, randomized, no bail",
        RunMode::Full => "full suite, exclude scaffolds, pre-PR gate",
        RunMode::Diagnostic => "serial, fixed order, isolate flake",
    }
}

fn print_result(code: i32) {
    println!();
    if code == 0 {
        display::print_success(&format!("Runner exited {}", code.to_string().green()));
    } else {
        display::print_error(&format!("Runner exited {}", code.to_string().red()));
    }
}
