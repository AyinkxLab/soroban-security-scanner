//! Command-line interface for the Soroban Security Scanner.
//!
//! This is the bootstrap entry point established in Phase 1. Subcommands are
//! added in later phases (`scan`, `rules`, `explain`, `config`, `init`).

use clap::{Parser, Subcommand};

/// Deterministic security analysis for Stellar Soroban smart contracts.
#[derive(Debug, Parser)]
#[command(
    name = "soroban-scan",
    version,
    about = "Deterministic security analysis for Stellar Soroban smart contracts",
    long_about = None,
    disable_help_subcommand = false
)]
struct Cli {
    /// Subcommand to execute. Defaults to printing version information.
    #[command(subcommand)]
    command: Option<Command>,
}

/// Available top-level commands.
#[derive(Debug, Subcommand)]
enum Command {
    /// Print the scanner name and version.
    Version,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        None | Some(Command::Version) => {
            println!("{}", soroban_scan_core::version_string());
        }
    }
    Ok(())
}
