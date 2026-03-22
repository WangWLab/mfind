//! CLI configuration module

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// CLI configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct CliConfig {
    /// Global settings
    pub global: GlobalConfig,
    /// Index settings
    pub index: IndexConfig,
    /// Search settings
    pub search: SearchConfig,
    /// UI settings
    pub ui: UiConfig,
}

/// Global configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Memory limit in MB
    pub memory_limit: usize,
    /// Parallelism level
    pub parallelism: Option<usize>,
    /// Log level
    pub log_level: String,
}

/// Index configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexConfig {
    /// Root paths to index
    pub roots: Vec<PathBuf>,
    /// Exclude directories
    pub exclude_dirs: Vec<String>,
    /// Exclude patterns
    pub exclude_patterns: Vec<String>,
    /// Include hidden files
    pub include_hidden: bool,
    /// Respect .gitignore
    pub gitignore: bool,
    /// Follow symlinks
    pub follow_symlinks: bool,
    /// Index metadata
    pub index_metadata: bool,
}

/// Search configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Default result limit
    pub default_limit: usize,
    /// Enable highlighting
    pub highlight: bool,
    /// Fuzzy search threshold
    pub fuzzy_threshold: f64,
}

/// UI configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct UiConfig {
    /// Color theme
    pub theme: String,
    /// Date format
    pub date_format: String,
    /// Size format (si or iec)
    pub size_format: String,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            global: GlobalConfig {
                memory_limit: 512,
                parallelism: None,
                log_level: "info".to_string(),
            },
            index: IndexConfig {
                roots: vec![],
                exclude_dirs: vec![
                    "node_modules".to_string(),
                    ".git".to_string(),
                    "target".to_string(),
                    "__pycache__".to_string(),
                ],
                exclude_patterns: vec![],
                include_hidden: false,
                gitignore: true,
                follow_symlinks: false,
                index_metadata: true,
            },
            search: SearchConfig {
                default_limit: 1000,
                highlight: true,
                fuzzy_threshold: 0.6,
            },
            ui: UiConfig {
                theme: "dark".to_string(),
                date_format: "%Y-%m-%d %H:%M".to_string(),
                size_format: "iec".to_string(),
            },
        }
    }
}

impl CliConfig {
    /// Load configuration from file
    pub fn load() -> anyhow::Result<Self> {
        let config_path = Self::config_path();

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: CliConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::config_path();

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;

        Ok(())
    }

    /// Get configuration file path
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("~/.config"))
            .join("mfind")
            .join("config.toml")
    }
}
