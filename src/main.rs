use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;

use cli::{Cli, Commands};
use commands::{handle_check, handle_init, handle_fix, handle_config};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { 
            path, 
            format, 
            output, 
            include_unknown, 
            quiet, 
            verbose, 
            exit_zero 
        } => {
            // Global options override subcommand options
            let final_quiet = cli.quiet || quiet;
            let final_verbose = cli.verbose || verbose;
            handle_check(path, format, output, include_unknown, final_quiet, final_verbose, exit_zero)
        }
        Commands::Init { policy } => {
            handle_init(policy, cli.quiet)
        }
        Commands::Fix { path, dry_run, interactive, format } => {
            handle_fix(path, dry_run, interactive, format, cli.quiet)
        }
        Commands::Config { show, validate } => {
            handle_config(show, validate, cli.quiet)
        }
    }
}
