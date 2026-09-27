use clap::{Parser, Subcommand, ValueEnum};

mod agent;
mod audit;
mod census;
mod detect;
mod display;
mod env;
mod gate;
mod prune;
mod pycheck;
mod pyproject;
mod pytests;
mod run;
mod setup;
mod tag;
mod tags;

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Rich,
    Json,
}

#[derive(Parser)]
#[command(
    name = "kinhin",
    about = "TDD lifecycle manager for code-assistant workflows",
    long_about = "Red → Green → Refactor → Prune.\n\n\
        Kinhin manages the lifecycle of tests in code-assistant workflows:\n\
        tag at birth, prune before merge, verify with mutation parity.",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List every test with its lifecycle tag and check that permanent tags name an authority
    Audit {
        /// Project directory
        #[arg(default_value = ".")]
        path: String,

        #[arg(long, value_enum, default_value_t = OutputFormat::Rich)]
        output: OutputFormat,
    },

    /// Count tests by lifecycle tag
    Census {
        /// Project directory
        #[arg(default_value = ".")]
        path: String,

        #[arg(long, value_enum, default_value_t = OutputFormat::Rich)]
        output: OutputFormat,
    },

    /// Check what the other commands need; with --install, set it up (uv-managed Python projects)
    Setup {
        /// Project directory
        #[arg(default_value = ".")]
        path: String,

        /// Add missing dev dependencies with `uv add --dev` and write the pytest/mutmut config
        #[arg(long)]
        install: bool,
    },

    /// Run the test suite under the Runner Contract (no bail, parallel, randomized)
    Run {
        /// loop: tests affected by changes. full: whole suite minus scaffolds. diagnostic: serial, stop at first failure
        #[arg(long, value_enum, default_value_t = run::RunMode::Loop)]
        mode: run::RunMode,

        /// Language override (auto-detected if omitted)
        #[arg(long, value_enum)]
        lang: Option<detect::Language>,

        /// TypeScript runner override (auto-detected if omitted)
        #[arg(long, value_enum)]
        ts_runner: Option<run::TsRunner>,

        /// Project directory
        #[arg(long, default_value = ".")]
        path: String,

        /// Extra arguments passed through to the test runner (after `--`)
        #[arg(last = true)]
        extra: Vec<String>,
    },

    /// Mutation parity gate (K₁ ⊇ K₀) on mutmut: Python projects
    Gate {
        /// Compare this run against a saved baseline; exit 1 if a baseline-killed mutant is no longer killed
        #[arg(long)]
        baseline: Option<String>,

        /// Save this run as the baseline (K₀)
        #[arg(long)]
        save_baseline: Option<String>,

        /// Restrict the run to mutant-name globs, e.g. 'pkg.services.billing*' (repeatable)
        #[arg(long)]
        scope: Vec<String>,

        /// Project directory
        #[arg(long, default_value = ".")]
        path: String,

        #[arg(long, value_enum, default_value_t = OutputFormat::Rich)]
        output: OutputFormat,
    },

    /// Classify untagged tests in one agent session and write the lifecycle markers
    Tag {
        /// Run the session and save the suggestions without touching any file
        #[arg(long)]
        dry_run: bool,

        /// Ignore suggestions cached by a previous --dry-run and run a new session
        #[arg(long)]
        force: bool,

        /// Claude model for the session
        #[arg(long, default_value = "haiku")]
        model: String,

        /// Project directory
        #[arg(long, default_value = ".")]
        path: String,

        #[arg(long, value_enum, default_value_t = OutputFormat::Rich)]
        output: OutputFormat,
    },

    /// Delete scaffold tests; with --verify, keep the ones mutation parity proves load-bearing
    Prune {
        /// Delete the tests (default: show the plan only)
        #[arg(long)]
        apply: bool,

        /// Prove K₁ ⊇ K₀ with mutmut; scaffolds whose removal lets mutants escape are kept
        #[arg(long, requires = "apply")]
        verify: bool,

        /// Project directory
        #[arg(long, default_value = ".")]
        path: String,

        #[arg(long, value_enum, default_value_t = OutputFormat::Rich)]
        output: OutputFormat,
    },
}

fn main() {
    let cli = Cli::parse();

    let code = match cli.command {
        Commands::Audit { path, output } => audit::run(&path, output),
        Commands::Census { path, output } => {
            let languages = detect::detect_languages(&path);
            let root = env::canonical_root(&path);
            let tests = tags::scan_test_files(&root.display().to_string(), &languages);
            census::Census::from_tests(&tests).display(output);
            0
        }
        Commands::Setup { path, install } => setup::run(&path, install),
        Commands::Run {
            mode,
            lang,
            ts_runner,
            path,
            extra,
        } => {
            run::run(mode, lang, ts_runner, &path, &extra);
            0
        }
        Commands::Gate {
            baseline,
            save_baseline,
            scope,
            path,
            output,
        } => {
            gate::run(&path, baseline.as_deref(), save_baseline.as_deref(), &scope, output);
            0
        }
        Commands::Tag {
            dry_run,
            force,
            model,
            path,
            output,
        } => {
            let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
            runtime.block_on(tag::run(&path, dry_run, force, &model, output))
        }
        Commands::Prune {
            apply,
            verify,
            path,
            output,
        } => prune::run(&path, apply, verify, output),
    };
    std::process::exit(code);
}
