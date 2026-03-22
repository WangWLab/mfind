//! Search command implementation

use clap::Args;
use console::style;

use crate::output::{OutputFormat, OutputWriter};

/// Search for files
#[derive(Args, Default)]
pub struct SearchCommand {
    /// Search pattern
    #[arg(index = 1)]
    pub pattern: Option<String>,

    /// Use regular expression
    #[arg(short = 'r', long)]
    pub regex: bool,

    /// Case sensitive search
    #[arg(short = 's', long)]
    pub case_sensitive: bool,

    /// Search only in specific path
    #[arg(short = 'p', long = "path")]
    pub paths: Vec<String>,

    /// Exclude directories
    #[arg(long = "exclude-dir")]
    pub exclude_dirs: Vec<String>,

    /// Filter by file extension
    #[arg(short = 'e', long = "ext")]
    pub extensions: Vec<String>,

    /// Filter by minimum size
    #[arg(long = "min-size")]
    pub min_size: Option<String>,

    /// Filter by maximum size
    #[arg(long = "max-size")]
    pub max_size: Option<String>,

    /// Filter by file type
    #[arg(long = "type", value_parser = ["f", "file", "d", "dir", "directory", "l", "link", "symlink"])]
    pub file_type: Option<String>,

    /// Include hidden files
    #[arg(long = "hidden")]
    pub hidden: bool,

    /// Do not respect .gitignore
    #[arg(long = "no-gitignore")]
    pub no_gitignore: bool,

    /// Follow symlinks
    #[arg(long = "follow")]
    pub follow: bool,

    /// Maximum number of results
    #[arg(short = 'n', long, default_value = "1000")]
    pub limit: usize,

    /// Output format
    #[arg(short = 'o', long, value_enum, default_value = "list")]
    pub output: OutputFormat,

    /// Print results as JSON
    #[arg(long)]
    pub json: bool,

    /// Print null-separated results (for xargs -0)
    #[arg(long)]
    pub print0: bool,

    /// Show detailed information
    #[arg(short = 'l', long)]
    pub long_list: bool,

    /// Show color in output
    #[arg(long, default_value = "auto")]
    pub color: String,
}

impl SearchCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        let pattern = self.pattern.as_deref().unwrap_or("");

        // Handle --json flag
        let output_format = if self.json {
            OutputFormat::Json
        } else {
            self.output
        };

        let writer = OutputWriter::new(output_format);

        // Show help if no pattern and no flags
        if pattern.is_empty() && std::env::args().len() == 2 {
            self.show_help();
            return Ok(());
        }

        // TODO: Implement actual search
        // For now, show a message
        eprintln!(
            "{} Searching for: {}",
            style("→").blue(),
            style(pattern).green()
        );

        println!(
            "{} Search functionality is under development.",
            style("⚠").yellow()
        );
        println!(
            "{} Run {} to build your first index.",
            style("ℹ").blue(),
            style("mfind index build").cyan()
        );

        Ok(())
    }

    fn show_help(&self) {
        println!("{}", style("mfind - Fast file search for macOS").bold());
        println!();
        println!("{}", style("USAGE:").bold());
        println!("    mfind [OPTIONS] [PATTERN]");
        println!();
        println!("{}", style("EXAMPLES:").bold());
        println!("    mfind apps                # Search for files starting with 'apps'");
        println!("    mfind \"*.pdf\"             # Find all PDF files");
        println!("    mfind --regex '.*\\.rs$'   # Find Rust files with regex");
        println!("    mfind -e rs               # Find files with .rs extension");
        println!("    mfind --type file         # Only files (not directories)");
        println!();
        println!("{}", style("OPTIONS:").bold());
        println!("    -r, --regex         Use regular expression");
        println!("    -s, --case-sensitive  Case sensitive search");
        println!("    -e, --ext <EXT>     Filter by extension");
        println!("    -n, --limit <N>     Maximum number of results");
        println!("    -o, --output <FMT>  Output format (list, json, table)");
        println!("    --hidden            Include hidden files");
        println!("    --no-gitignore      Don't respect .gitignore");
        println!();
        println!("{}", style("Run 'mfind --help' for more details.").dim());
    }
}
