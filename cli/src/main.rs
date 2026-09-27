use clap::{Parser, Subcommand, ValueEnum};
use tracing_subscriber::EnvFilter;

mod agent;
mod audit;
mod census;
mod detect;
mod display;
mod gate;
mod prune;
mod run;
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

    #[arg(long, value_enum, default_value_t = OutputFormat::Rich, global = true)]
    output: OutputFormat,

    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan test files, report tags, validate refs
    Audit {
        /// Directory to scan (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
    },

    /// Count tests by lifecycle tag
    Census {
        /// Directory to scan (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
    },

    /// Wrap the test runner with Runner Contract config
    Run {
        /// Run mode: loop (changed-set), full (gate), diagnostic (isolate flake)
        #[arg(long, value_enum, default_value_t = run::RunMode::Loop)]
        mode: run::RunMode,

        /// Language override (auto-detects if omitted)
        #[arg(long, value_enum)]
        lang: Option<detect::Language>,

        /// TypeScript runner override: jest or vitest (auto-detects if omitted)
        #[arg(long, value_enum)]
        ts_runner: Option<run::TsRunner>,

        /// Directory to run in (defaults to current directory)
        #[arg(long, default_value = ".")]
        path: String,

        /// Extra arguments passed through to the underlying test runner
        #[arg(last = true)]
        extra: Vec<String>,
    },

    /// Run mutation parity gate (K₁ ⊇ K₀)
    Gate {
        /// Language override (auto-detects if omitted)
        #[arg(long, value_enum)]
        lang: Option<detect::Language>,

        /// Path to baseline file (K₀) for comparison
        #[arg(long)]
        baseline: Option<String>,

        /// Save current results as baseline (K₀) to this path
        #[arg(long)]
        save_baseline: Option<String>,

        /// Directory to run in (defaults to current directory)
        #[arg(long, default_value = ".")]
        path: String,
    },

    /// Auto-tag tests via AI agent session
    Tag {
        /// Language override (auto-detects if omitted)
        #[arg(long, value_enum)]
        lang: Option<detect::Language>,

        /// Apply suggested tags to files (default: dry run)
        #[arg(long)]
        apply: bool,

        /// Directory to scan (defaults to current directory)
        #[arg(long, default_value = ".")]
        path: String,
    },

    /// Prune construction tests: census → classify → collapse → verify
    Prune {
        /// Language override (auto-detects if omitted)
        #[arg(long, value_enum)]
        lang: Option<detect::Language>,

        /// Apply deletions (default: dry run showing the plan)
        #[arg(long)]
        apply: bool,

        /// Run mutation parity gate after pruning
        #[arg(long)]
        verify: bool,

        /// Directory to scan (defaults to current directory)
        #[arg(long, default_value = ".")]
        path: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let filter = match cli.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(filter))
        .without_time()
        .init();

    match cli.command {
        Commands::Audit { ref path } => audit::run(path, cli.output),
        Commands::Census { ref path } => {
            let languages = detect::detect_languages(path);
            let tests = tags::scan_test_files(path, &languages);
            let census = census::Census::from_tests(&tests);
            census.display(cli.output);
        }
        Commands::Run {
            mode,
            lang,
            ts_runner,
            ref path,
            ref extra,
        } => {
            run::run(mode, lang, ts_runner, path, extra);
        }
        Commands::Gate {
            lang,
            ref baseline,
            ref save_baseline,
            ref path,
        } => {
            gate::run(path, lang, baseline.as_deref(), save_baseline.as_deref(), cli.output);
        }
        Commands::Tag {
            lang,
            apply,
            ref path,
        } => {
            let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
            rt.block_on(tag::run(path, lang, apply, cli.output));
        }
        Commands::Prune {
            lang,
            apply,
            verify,
            ref path,
        } => {
            let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
            rt.block_on(prune::run(path, lang, apply, verify, cli.output));
        }
    }
}
