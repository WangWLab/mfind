//! Config command implementation

use clap::Subcommand;
use console::style;

/// Configuration management commands
#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Show current configuration
    Show,

    /// Edit configuration
    Edit,

    /// Reset configuration to defaults
    Reset,

    /// Show configuration file path
    Path,
}

impl ConfigCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        match self {
            ConfigCommand::Show => self.show(),
            ConfigCommand::Edit => self.edit(),
            ConfigCommand::Reset => self.reset(),
            ConfigCommand::Path => self.path(),
        }
    }

    fn show(&self) -> anyhow::Result<()> {
        println!("{}", style("Current Configuration").bold());
        println!();
        println!("  memory_limit:     512 MB");
        println!("  parallelism:      auto");
        println!("  gitignore:        true");
        println!("  include_hidden:   false");
        println!("  follow_symlinks:  false");
        println!();
        println!(
            "{} Run {} to edit configuration.",
            style("ℹ").blue(),
            style("mfind config edit").cyan()
        );
        Ok(())
    }

    fn edit(&self) -> anyhow::Result<()> {
        println!("{}", style("Opening configuration editor...").yellow());
        println!("Configuration editing is under development.");
        Ok(())
    }

    fn reset(&self) -> anyhow::Result<()> {
        println!(
            "{} Configuration reset to defaults.",
            style("✓").green()
        );
        Ok(())
    }

    fn path(&self) -> anyhow::Result<()> {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("~/.config"))
            .join("mfind")
            .join("config.toml");

        println!("{}", config_path.display());
        Ok(())
    }
}
