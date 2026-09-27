use clap::{Parser, Subcommand, ValueEnum};
use tracing_subscriber::EnvFilter;

mod audit;
mod census;
mod detect;
mod display;
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
    Run,

    /// Run mutation parity gate (K₁ ⊇ K₀)
    Gate,

    /// Auto-tag tests via AI agent session
    Tag,

    /// Prune construction tests: census → classify → collapse → verify
    Prune,
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
        Commands::Run => {
            eprintln!("kinhin run: not yet implemented (Phase 7b)");
            std::process::exit(1);
        }
        Commands::Gate => {
            eprintln!("kinhin gate: not yet implemented (Phase 7c)");
            std::process::exit(1);
        }
        Commands::Tag => {
            eprintln!("kinhin tag: not yet implemented (Phase 7d)");
            std::process::exit(1);
        }
        Commands::Prune => {
            eprintln!("kinhin prune: not yet implemented (Phase 7e)");
            std::process::exit(1);
        }
    }
}
