//! Search command implementation

use clap::Args;
use console::style;
use std::path::PathBuf;
use std::fs;

use crate::output::{OutputFormat, OutputWriter};
use mfind_core::{IndexEngine, IndexConfig, QueryParser};
use mfind_core::index::engine::IndexEngineTrait;

/// Get cache directory path
fn get_cache_dir() -> anyhow::Result<PathBuf> {
    let cache_dir = if cfg!(target_os = "macos") {
        dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."))
    } else {
        dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."))
    };
    let mfind_cache = cache_dir.join("mfind");
    fs::create_dir_all(&mfind_cache)?;
    Ok(mfind_cache)
}

/// Generate cache key from search paths
fn generate_cache_key(paths: &[PathBuf]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    for path in paths {
        path.hash(&mut hasher);
    }
    format!("index_{:016x}.bin", hasher.finish())
}

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

    /// Watch for file changes after initial search (macOS only)
    #[arg(long)]
    pub watch: bool,
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

        // Try to load cached index
        let cache_dir = get_cache_dir()?;
        let cache_key = generate_cache_key(&search_paths);
        let cache_path = cache_dir.join(&cache_key);

        if cache_path.exists() {
            // Load from cache
            let start = std::time::Instant::now();
            match fs::read(&cache_path) {
                Ok(data) => {
                    if let Ok(_) = engine.import(&data).await {
                        let elapsed = start.elapsed();
                        eprintln!(
                            "{} Loaded {} files from cache in {:?}",
                            style("✓").green(),
                            style(engine.stats().total_files).cyan(),
                            elapsed
                        );

                        // Rebuild to get updated stats
                        engine.build(&search_paths).await?
                    } else {
                        // Cache corrupted, rebuild
                        eprintln!(
                            "{} Building index for: {}",
                            style("→").blue(),
                            style(search_paths.iter()
                                .map(|p| p.display().to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                            ).green()
                        );
                        engine.build(&search_paths).await?
                    }
                }
                Err(_) => {
                    eprintln!(
                        "{} Building index for: {}",
                        style("→").blue(),
                        style(search_paths.iter()
                            .map(|p| p.display().to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                        ).green()
                    );
                    engine.build(&search_paths).await?
                }
            }
        } else {
            // Build new index
            eprintln!(
                "{} Building index for: {}",
                style("→").blue(),
                style(search_paths.iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
                ).green()
            );
            engine.build(&search_paths).await?
        };

        // Save to cache
        if let Ok(data) = engine.export().await {
            let _ = fs::write(&cache_path, data);
        }

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

        // Watch mode - monitor for file changes
        #[cfg(target_os = "macos")]
        if self.watch {
            use mfind_core::fs::{NativeFSEventsWatcher, FileSystemMonitor, MonitorConfig};
            use mfind_core::event::FSEventType;

            eprintln!(
                "{} Starting filesystem monitor (watch mode)...",
                style("→").blue()
            );
            eprintln!("{} Press Ctrl+C to stop", style("ℹ").yellow());

            // Start FSEvents monitoring
            let mut watcher = NativeFSEventsWatcher::new(MonitorConfig::default())?;
            watcher.start(&search_paths).await?;

            let receiver = watcher.event_stream();

            // Wait for events
            loop {
                tokio::select! {
                    Ok(event) = receiver.recv_async() => {
                        let event_type_str = match &event.event_type {
                            FSEventType::Create => "Create",
                            FSEventType::Delete => "Delete",
                            FSEventType::Modify => "Modify",
                            FSEventType::Rename { .. } => "Rename",
                            FSEventType::Metadata => "Metadata",
                        };

                        eprintln!(
                            "{} [{}] {}",
                            style("●").cyan(),
                            style(event_type_str).yellow(),
                            style(event.path.display()).dim()
                        );
                    }
                    _ = tokio::signal::ctrl_c() => {
                        eprintln!(
                            "\n{} Stopping filesystem monitor...",
                            style("→").blue()
                        );
                        watcher.stop().await?;
                        break;
                    }
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        if self.watch {
            eprintln!(
                "{} Watch mode is only supported on macOS",
                style("⚠").yellow()
            );
        }

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
