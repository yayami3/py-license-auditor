use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "py-license-auditor")]
#[command(about = "Extract license information from Python packages")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,
    
    /// Suppress non-error output
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run license audit on packages
    Check {
        /// Path to site-packages directory or virtual environment
        path: Option<PathBuf>,

        /// Output format
        #[arg(short, long)]
        format: Option<OutputFormat>,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Include packages without license information
        #[arg(long)]
        include_unknown: bool,

        /// Show errors only
        #[arg(short, long)]
        quiet: bool,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,

        /// Exit with code 0 even on violations
        #[arg(long)]
        exit_zero: bool,
    },
    /// Initialize configuration with preset policy
    Init {
        /// Policy preset
        policy: InitPreset,
    },
    /// Automatically fix violations by adding exceptions
    Fix {
        /// Path to site-packages directory or virtual environment
        path: Option<PathBuf>,

        /// Show changes without applying them
        #[arg(long)]
        dry_run: bool,

        /// Interactive mode for license-based exception handling
        #[arg(long)]
        interactive: bool,

        /// Output format for changes
        #[arg(short, long)]
        format: Option<OutputFormat>,
    },
    /// Show or validate configuration
    Config {
        /// Show current configuration
        #[arg(long)]
        show: bool,

        /// Validate configuration file
        #[arg(long)]
        validate: bool,
    },
}

#[derive(Clone, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
}

#[derive(Clone, ValueEnum)]
pub enum InitPreset {
    Green,
    Yellow,
    Red,
}
