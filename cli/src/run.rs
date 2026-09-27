//! `kinhin run`: launch the project's test runner configured per the
//! Runner Contract (no bail, parallel, randomized, changed-set in the loop).

use std::path::Path;
use std::process::{Command, Stdio};

use clap::ValueEnum;
use owo_colors::OwoColorize;

use crate::detect::{self, Language};
use crate::display;
use crate::env::{self, PyEnv};

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum, Debug)]
pub enum RunMode {
    /// Changed-set only, parallel, randomized, no bail
    Loop,
    /// Full suite excluding scaffolds, pre-PR gate
    Full,
    /// Serial, fixed order, stop at first failure: isolate a flake
    Diagnostic,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum TsRunner {
    Jest,
    Vitest,
}

/// Which optional pytest plugins the project environment has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PytestPlugins {
    pub xdist: bool,
    pub randomly: bool,
    pub testmon: bool,
}

struct Invocation {
    program: String,
    args: Vec<String>,
    /// Exit codes of the runner that mean success in this mode.
    ok_codes: Vec<(i32, &'static str)>,
}

/// pytest arguments for a mode. Pure, so the contract is unit-testable.
pub fn pytest_args(mode: RunMode, plugins: PytestPlugins, seed: u32) -> Vec<String> {
    let mut args: Vec<String> = vec!["-ra".into(), "-q".into()];
    match mode {
        RunMode::Loop | RunMode::Full => {
            args.push("--maxfail=0".into());
            if plugins.xdist {
                args.extend(["-n".into(), "auto".into()]);
            }
            if plugins.randomly {
                args.push(format!("--randomly-seed={seed}"));
            }
            if mode == RunMode::Loop && plugins.testmon {
                args.push("--testmon".into());
            }
            if mode == RunMode::Full {
                args.extend(["-m".into(), "not scaffold".into()]);
            }
        }
        RunMode::Diagnostic => {
            args.push("-x".into());
            if plugins.randomly {
                args.extend(["-p".into(), "no:randomly".into()]);
            }
            if plugins.xdist {
                args.extend(["-p".into(), "no:xdist".into()]);
            }
        }
    }
    args
}

fn python_invocation(root: &Path, mode: RunMode) -> Invocation {
    let py = PyEnv::detect(root);
    let available = py.modules_available(&["pytest", "xdist", "pytest_randomly", "testmon"]);
    if !available["pytest"] {
        display::print_error("pytest is not installed in the project environment. Run: kinhin setup --install");
        std::process::exit(127);
    }
    let plugins = PytestPlugins {
        xdist: available["xdist"],
        randomly: available["pytest_randomly"],
        testmon: available["testmon"],
    };

    let mut missing: Vec<&str> = Vec::new();
    if !plugins.xdist {
        missing.push("pytest-xdist (parallel)");
    }
    if !plugins.randomly {
        missing.push("pytest-randomly (random order)");
    }
    if !plugins.testmon && mode == RunMode::Loop {
        missing.push("pytest-testmon (changed-set selection)");
    }
    if !missing.is_empty() && mode != RunMode::Diagnostic {
        display::print_warning(&format!(
            "running without {}. Run: kinhin setup --install",
            missing.join(", ")
        ));
    }

    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(1, |d| (d.as_micros() % 1_000_000) as u32 + 1);

    let mut words = py.display_words("pytest");
    let program = words.remove(0);
    words.extend(pytest_args(mode, plugins, seed));

    let mut ok_codes = Vec::new();
    if mode == RunMode::Loop && plugins.testmon {
        // pytest exits 5 when nothing was collected: with testmon that means
        // no test is affected by the changes since the last run.
        ok_codes.push((5, "no test is affected by the changes since the last run"));
    }
    Invocation {
        program,
        args: words,
        ok_codes,
    }
}

fn rust_invocation(mode: RunMode) -> Invocation {
    let mut args: Vec<String> = vec!["nextest".into(), "run".into(), "--no-fail-fast".into()];
    match mode {
        RunMode::Loop => {}
        RunMode::Full => args.extend(["-E".into(), "not test(scaffold::)".into()]),
        RunMode::Diagnostic => args.extend(["-j".into(), "1".into()]),
    }
    Invocation {
        program: "cargo".into(),
        args,
        ok_codes: Vec::new(),
    }
}

fn java_invocation(root: &Path, mode: RunMode) -> Invocation {
    let gradle = root.join("build.gradle").exists() || root.join("build.gradle.kts").exists();
    let (program, mut args): (&str, Vec<String>) = if gradle {
        ("gradle", vec!["test".into(), "--continue".into()])
    } else {
        ("mvn", vec!["test".into(), "-Dmaven.test.failure.ignore=true".into()])
    };
    match mode {
        RunMode::Loop => {}
        RunMode::Full => args.push("-DexcludedGroups=scaffold".into()),
        RunMode::Diagnostic => args.push("-Djunit.jupiter.execution.parallel.enabled=false".into()),
    }
    Invocation {
        program: program.into(),
        args,
        ok_codes: Vec::new(),
    }
}

fn typescript_invocation(root: &Path, mode: RunMode, runner: Option<TsRunner>) -> Invocation {
    let runner = runner.unwrap_or_else(|| {
        let vitest = ["vitest.config.ts", "vitest.config.js", "vitest.config.mts"]
            .iter()
            .any(|f| root.join(f).exists());
        if vitest { TsRunner::Vitest } else { TsRunner::Jest }
    });
    let mut args: Vec<String> = Vec::new();
    match runner {
        TsRunner::Jest => {
            args.extend(["jest".into(), "--bail=0".into()]);
            match mode {
                RunMode::Loop => args.extend(["--maxWorkers=50%".into(), "--randomize".into(), "--onlyChanged".into()]),
                RunMode::Full => args.extend([
                    "--maxWorkers=50%".into(),
                    "--randomize".into(),
                    "--testPathIgnorePatterns".into(),
                    "scaffold".into(),
                ]),
                RunMode::Diagnostic => args.push("--runInBand".into()),
            }
        }
        TsRunner::Vitest => {
            args.extend(["vitest".into(), "run".into(), "--bail=0".into()]);
            match mode {
                RunMode::Loop => args.extend(["--sequence.shuffle".into(), "--changed".into()]),
                RunMode::Full => args.extend([
                    "--sequence.shuffle".into(),
                    "--exclude".into(),
                    "**/*.scaffold.test.ts".into(),
                ]),
                RunMode::Diagnostic => args.push("--no-file-parallelism".into()),
            }
        }
    }
    Invocation {
        program: env::ts_package_runner(root).into(),
        args,
        ok_codes: Vec::new(),
    }
}

fn shell_quote(word: &str) -> String {
    if word.chars().all(|c| c.is_alphanumeric() || "-_=./:%*@,+".contains(c)) {
        word.to_string()
    } else {
        format!("'{}'", word.replace('\'', "'\\''"))
    }
}

pub fn run(mode: RunMode, lang: Option<Language>, ts_runner: Option<TsRunner>, path: &str, extra_args: &[String]) {
    let root = env::canonical_root(path);
    let languages = match lang {
        Some(l) => vec![l],
        None => detect::detect_languages(path),
    };
    let Some(&language) = languages.first() else {
        display::print_error("no language detected. Use --lang to specify.");
        std::process::exit(2);
    };
    if languages.len() > 1 {
        display::print_warning(&format!(
            "multiple languages detected ({}); running {language}. Use --lang to choose.",
            languages.iter().map(ToString::to_string).collect::<Vec<_>>().join(", "),
        ));
    }

    let mut invocation = match language {
        Language::Python => python_invocation(&root, mode),
        Language::Rust => rust_invocation(mode),
        Language::Java => java_invocation(&root, mode),
        Language::TypeScript => typescript_invocation(&root, mode, ts_runner),
    };
    invocation.args.extend(extra_args.iter().cloned());

    print_header(mode, language, &invocation);

    let status = Command::new(&invocation.program)
        .args(&invocation.args)
        .current_dir(&root)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(status) => {
            let code = status.code().unwrap_or(1);
            println!();
            if code == 0 {
                display::print_success("runner exited 0");
            } else if let Some((_, meaning)) = invocation.ok_codes.iter().find(|(c, _)| *c == code) {
                display::print_success(&format!("runner exited {code}: {meaning}"));
                std::process::exit(0);
            } else {
                display::print_error(&format!("runner exited {code}"));
            }
            std::process::exit(code);
        }
        Err(e) => {
            display::print_error(&format!("failed to execute `{}`: {e}", invocation.program));
            std::process::exit(127);
        }
    }
}

fn print_header(mode: RunMode, language: Language, invocation: &Invocation) {
    let (badge, icon, description) = match mode {
        RunMode::Loop => (
            format!("{}", " LOOP ".on_cyan().bold()),
            "🔄",
            "changed-set, parallel, randomized, no bail",
        ),
        RunMode::Full => (
            format!("{}", " FULL ".on_green().bold()),
            "🛡",
            "full suite, scaffolds excluded, pre-PR gate",
        ),
        RunMode::Diagnostic => (
            format!("{}", " DIAG ".on_yellow().bold()),
            "🔍",
            "serial, fixed order, stop at first failure",
        ),
    };
    let command: Vec<String> = std::iter::once(invocation.program.clone())
        .chain(invocation.args.iter().map(|a| shell_quote(a)))
        .collect();

    println!();
    println!("{icon} {badge}  {}  {}", language.to_string().bold(), description.dimmed());
    println!("  {} {}", "$".dimmed(), command.join(" "));
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: PytestPlugins = PytestPlugins {
        xdist: true,
        randomly: true,
        testmon: true,
    };
    const NONE: PytestPlugins = PytestPlugins {
        xdist: false,
        randomly: false,
        testmon: false,
    };

    #[test]
    fn no_mode_except_diagnostic_bails_on_first_failure() {
        for mode in [RunMode::Loop, RunMode::Full] {
            let args = pytest_args(mode, ALL, 7);
            assert!(!args.contains(&"-x".to_string()), "{mode:?}");
            assert!(args.contains(&"--maxfail=0".to_string()), "{mode:?}");
        }
        assert!(pytest_args(RunMode::Diagnostic, ALL, 7).contains(&"-x".to_string()));
    }

    #[test]
    fn full_mode_excludes_scaffolds_and_never_selects_by_change() {
        let args = pytest_args(RunMode::Full, ALL, 7);
        let m = args.iter().position(|a| a == "-m").unwrap();
        assert_eq!(args[m + 1], "not scaffold");
        assert!(!args.contains(&"--testmon".to_string()));
    }

    #[test]
    fn loop_mode_selects_changed_set_with_a_printed_seed() {
        let args = pytest_args(RunMode::Loop, ALL, 4242);
        assert!(args.contains(&"--testmon".to_string()));
        assert!(args.contains(&"--randomly-seed=4242".to_string()));
        assert!(args.windows(2).any(|w| w == ["-n", "auto"]));
    }

    #[test]
    fn missing_plugins_never_produce_their_flags() {
        let args = pytest_args(RunMode::Loop, NONE, 7).join(" ");
        assert_eq!(args, "-ra -q --maxfail=0");
        assert_eq!(pytest_args(RunMode::Diagnostic, NONE, 7).join(" "), "-ra -q -x");
    }

    #[test]
    fn diagnostic_mode_is_serial_and_fixed_order() {
        let args = pytest_args(RunMode::Diagnostic, ALL, 7).join(" ");
        assert!(args.contains("-p no:randomly") && args.contains("-p no:xdist"));
        assert!(!args.contains("-n auto"));
    }

    #[test]
    fn arguments_with_spaces_are_quoted_for_display() {
        assert_eq!(shell_quote("not scaffold"), "'not scaffold'");
        assert_eq!(shell_quote("--maxfail=0"), "--maxfail=0");
    }
}
