//! Search command implementation

use clap::Args;
use console::style;

use crate::output::{OutputFormat, OutputWriter};
use mfind_core::{IndexEngine, IndexConfig, QueryParser};
use mfind_core::index::engine::IndexEngineTrait;
use std::path::PathBuf;

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

        // Determine search paths
        let search_paths: Vec<PathBuf> = if self.paths.is_empty() {
            vec![std::env::current_dir()?]
        } else {
            self.paths.iter().map(PathBuf::from).collect()
        };

        // Build index configuration
        let index_config = IndexConfig {
            include_hidden: self.hidden,
            gitignore_ignore: !self.no_gitignore,
            follow_symlinks: self.follow,
            ..Default::default()
        };

        // Create index engine
        let mut engine = IndexEngine::new(index_config)?;

        // Build or load index
        eprintln!(
            "{} Building index for: {}",
            style("→").blue(),
            style(search_paths.iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
            ).green()
        );

        let stats = engine.build(&search_paths).await?;
        eprintln!(
            "{} Indexed {} files in {:?}",
            style("✓").green(),
            style(stats.total_files).cyan(),
            stats.build_time
        );

        // Parse query
        let query_pattern = if self.regex {
            format!("regex:{}", pattern)
        } else if !self.extensions.is_empty() {
            // Handle extension filter
            if self.extensions.len() == 1 {
                format!("ext:{}", self.extensions[0].trim_start_matches('.'))
            } else {
                // Multiple extensions - use pattern matching
                pattern.to_string()
            }
        } else {
            pattern.to_string()
        };

        let query = QueryParser::parse_with_options(
            &query_pattern,
            self.case_sensitive,
        )?;

        // Execute search
        eprintln!(
            "{} Searching for: {}",
            style("→").blue(),
            style(&query.pattern).green()
        );

        let start = std::time::Instant::now();
        let results = engine.search(&query)?;
        let elapsed = start.elapsed();

        // Format and print results
        let total_matches = results.matches.len();
        let mut paths: Vec<String> = results.matches;

        if paths.is_empty() {
            println!(
                "{} No files found matching: {}",
                style("ℹ").blue(),
                style(pattern).yellow()
            );
            return Ok(());
        }

        // Apply limit
        if paths.len() > self.limit {
            paths.truncate(self.limit);
        }

        // Print results using output writer
        writer.print_string_results(&paths);

        // Print summary to stderr
        eprintln!(
            "{} Found {} files in {:?}",
            style("✓").green(),
            style(total_matches).cyan(),
            elapsed
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
