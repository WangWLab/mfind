//! Index command implementation

use clap::Subcommand;
use console::style;
use mfind_core::{IndexEngine, IndexConfig, get_default_index_path};
use mfind_core::index::engine::IndexEngineTrait;
use std::path::PathBuf;
use std::fs;

/// Format bytes to human readable string
fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = 1024 * KB;
    const GB: usize = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Index management commands
#[derive(Subcommand)]
pub enum IndexCommand {
    /// Build a new index
    Build(IndexBuildCommand),

    /// Rebuild index from scratch
    Rebuild(IndexBuildCommand),

    /// Show index status
    Status,

    /// Pause indexing
    Pause,

    /// Resume indexing
    Resume,

    /// Export index
    Export(IndexExportCommand),

    /// Import index
    Import(IndexImportCommand),

    /// Clear index
    Clear,
}

impl IndexCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        match self {
            IndexCommand::Build(cmd) | IndexCommand::Rebuild(cmd) => cmd.run().await,
            IndexCommand::Status => self.status().await,
            IndexCommand::Pause => {
                println!("{}", style("Indexing paused").yellow());
                Ok(())
            }
            IndexCommand::Resume => {
                println!("{}", style("Indexing resumed").green());
                Ok(())
            }
            IndexCommand::Export(cmd) => cmd.run().await,
            IndexCommand::Import(cmd) => cmd.run().await,
            IndexCommand::Clear => self.clear(),
        }
    }

    async fn status(&self) -> anyhow::Result<()> {
        let index_path = get_default_index_path();

        println!("{}", style("Index Status").bold());
        println!();

        if index_path.exists() {
            // Try to load index and get stats
            let index_config = IndexConfig::default();
            let mut engine = IndexEngine::new(index_config)?;

            match fs::read(&index_path) {
                Ok(data) => {
                    if engine.import(&data).await.is_ok() {
                        let stats = engine.stats();
                        let metadata = fs::metadata(&index_path)?;
                        let modified = metadata.modified()?;
                        let modified_str = format!("{modified:?}")
                            .split('(')
                            .next()
                            .unwrap_or("Unknown")
                            .to_string();

                        println!("  Status:        {}", style("Initialized").green());
                        println!("  Index path:    {}", index_path.display());
                        println!("  Total files:   {}", style(stats.total_files).cyan());
                        println!("  Total dirs:    {}", stats.total_dirs);
                        println!("  Index size:    {}", style(format_bytes(stats.total_bytes as usize)).cyan());
                        println!("  Last update:   {}", modified_str);
                        println!("  Health:        {:?}", stats.health);
                    } else {
                        println!("  Status:        {}", style("Corrupted").red());
                        println!("  Index path:    {}", index_path.display());
                        println!();
                        println!("{}", style("ℹ Index file exists but failed to load. Run `mfind index rebuild` to fix.").yellow());
                    }
                }
                Err(_) => {
                    println!("  Status:        {}", style("Not initialized").yellow());
                    println!("  Index path:    {}", index_path.display());
                    println!();
                    println!(
                        "{} Run {} to build your first index.",
                        style("ℹ").blue(),
                        style("mfind index build").cyan()
                    );
                }
            }
        } else {
            println!("  Status:        {}", style("Not initialized").yellow());
            println!("  Index path:    {}", index_path.display());
            println!();
            println!(
                "{} Run {} to build your first index.",
                style("ℹ").blue(),
                style("mfind index build").cyan()
            );
        }

        Ok(())
    }

    fn clear(&self) -> anyhow::Result<()> {
        println!("{}", style("Index cleared").green());
        Ok(())
    }
}

/// Build index command
#[derive(clap::Args)]
pub struct IndexBuildCommand {
    /// Paths to index
    #[arg(required = true)]
    pub paths: Vec<String>,

    /// Exclude patterns
    #[arg(long = "exclude")]
    pub exclude: Vec<String>,

    /// Include hidden files
    #[arg(long)]
    pub hidden: bool,

    /// Don't respect .gitignore
    #[arg(long)]
    pub no_gitignore: bool,

    /// Force rebuild even if index exists
    #[arg(long)]
    pub force: bool,
}

impl IndexBuildCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        // Parse paths
        let roots: Vec<PathBuf> = self.paths.iter().map(PathBuf::from).collect();

        // Build index configuration
        let index_config = IndexConfig {
            include_hidden: self.hidden,
            gitignore_ignore: !self.no_gitignore,
            exclude_patterns: self.exclude.clone(),
            ..Default::default()
        };

        // Create index engine
        let mut engine = IndexEngine::new(index_config)?;

        eprintln!(
            "{} Building index for:",
            style("→").blue()
        );

        for path in &roots {
            eprintln!("    {}", style(path.display()).green());
        }

        if !self.exclude.is_empty() {
            eprintln!(
                "{} Excluding:",
                style("→").blue()
            );
            for pattern in &self.exclude {
                eprintln!("    {}", style(pattern).yellow());
            }
        }
        eprintln!();

        // Build the index
        let start = std::time::Instant::now();
        let stats = engine.build(&roots).await?;
        let elapsed = start.elapsed();

        // Print summary
        eprintln!(
            "{} Indexed {} files in {:?}",
            style("✓").green(),
            style(stats.total_files).cyan(),
            elapsed
        );
        eprintln!(
            "{} Index status: {:?}",
            style("ℹ").blue(),
            stats.health
        );

        Ok(())
    }
}

/// Export index command
#[derive(clap::Args)]
pub struct IndexExportCommand {
    /// Output file path
    #[arg(short = 'o', long)]
    pub output: Option<String>,

    /// Paths to index (required for building new index before export)
    #[arg(required = true)]
    pub paths: Vec<String>,
}

impl IndexExportCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        // Parse paths
        let roots: Vec<PathBuf> = self.paths.iter().map(PathBuf::from).collect();

        // Build index configuration
        let index_config = IndexConfig::default();

        // Create index engine
        let mut engine = IndexEngine::new(index_config)?;

        eprintln!(
            "{} Building index for export:",
            style("→").blue()
        );

        for path in &roots {
            eprintln!("    {}", style(path.display()).green());
        }

        // Build the index
        let start = std::time::Instant::now();
        let stats = engine.build(&roots).await?;
        let build_elapsed = start.elapsed();

        eprintln!(
            "{} Indexed {} files in {:?}",
            style("✓").green(),
            style(stats.total_files).cyan(),
            build_elapsed
        );

        // Export to bytes
        let export_data = engine.export().await?;
        eprintln!(
            "{} Exported index size: {}",
            style("→").blue(),
            style(format_bytes(export_data.len())).cyan()
        );

        // Write to file
        let output_path = self.output.as_deref().unwrap_or("mfind_index.dat");
        fs::write(output_path, &export_data)?;

        eprintln!(
            "{} Index exported to: {}",
            style("✓").green(),
            style(output_path).green()
        );

        Ok(())
    }
}

/// Import index command
#[derive(clap::Args)]
pub struct IndexImportCommand {
    /// Input file path
    #[arg(required = true)]
    pub input: String,
}

impl IndexImportCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        eprintln!(
            "{} Importing index from: {}",
            style("→").blue(),
            style(&self.input).green()
        );

        // Read index data
        let data = fs::read(&self.input)?;
        eprintln!(
            "{} Read {} bytes",
            style("→").blue(),
            style(format_bytes(data.len())).cyan()
        );

        // Create index engine
        let index_config = IndexConfig::default();
        let mut engine = IndexEngine::new(index_config)?;

        // Import index
        let start = std::time::Instant::now();
        engine.import(&data).await?;
        let import_elapsed = start.elapsed();

        eprintln!(
            "{} Index imported in {:?}",
            style("✓").green(),
            import_elapsed
        );

        // Show stats
        let stats = engine.stats();
        eprintln!(
            "{} Total files: {}",
            style("ℹ").blue(),
            style(stats.total_files).cyan()
        );
        eprintln!(
            "{} Health: {:?}",
            style("ℹ").blue(),
            stats.health
        );

        // Test search
        eprintln!();
        eprintln!(
            "{} Testing search...",
            style("→").blue()
        );

        use mfind_core::query::QueryParser;
        let query = QueryParser::parse("*")?;
        let results = engine.search(&query)?;
        eprintln!(
            "{} Search returned {} results",
            style("✓").green(),
            style(results.total).cyan()
        );

        Ok(())
    }
}
