//! CLI commands module

use clap::{Args, Parser, Subcommand};

mod config;
mod index;
mod search;
mod service;

pub use config::ConfigCommand;
pub use index::IndexCommand;
pub use search::SearchCommand;
pub use service::ServiceCommand;

/// mfind - Fast, independent file search for macOS
#[derive(Parser)]
#[command(name = "mfind")]
#[command(author = "mfind Contributors")]
#[command(version)]
#[command(about = "Fast, independent file search for macOS", long_about = None)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(Subcommand)]
pub enum CliCommand {
    /// Search for files (default command)
    Search(SearchCommand),

    /// Index management
    #[command(subcommand)]
    Index(IndexCommand),

    /// Configuration management
    #[command(subcommand)]
    Config(ConfigCommand),

    /// Background service management
    #[command(subcommand)]
    Service(ServiceCommand),

    /// Generate shell completions
    Completions(CompletionsCommand),
}

/// Generate shell completions
#[derive(Args)]
pub struct CompletionsCommand {
    /// Shell type
    #[arg(value_enum)]
    shell: clap_complete::Shell,
}

impl CompletionsCommand {
    pub fn run(self) -> anyhow::Result<()> {
        use clap::CommandFactory;

        let mut cmd = CliArgs::command();
        let name = cmd.get_name().to_string();
        clap_complete::generate(self.shell, &mut cmd, name, &mut std::io::stdout());
        Ok(())
    }
}

// Default to search command when no subcommand is given
impl Default for CliCommand {
    fn default() -> Self {
        CliCommand::Search(SearchCommand::default())
    }
}
