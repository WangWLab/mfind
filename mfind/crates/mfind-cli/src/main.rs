//! mfind-cli: Command-line interface for mfind
//!
//! A fast, independent file search tool for macOS.

use clap::Parser;

mod commands;
mod config;
mod output;

use commands::{CliArgs, CliCommand};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let args = CliArgs::parse();

    match args.command {
        CliCommand::Search(cmd) => cmd.run().await,
        CliCommand::Index(cmd) => cmd.run().await,
        CliCommand::Config(cmd) => cmd.run().await,
        CliCommand::Service(cmd) => cmd.run().await,
        CliCommand::Completions(cmd) => cmd.run(),
    }
}
