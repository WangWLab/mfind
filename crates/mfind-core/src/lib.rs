//! mfind-core: Core indexing and search engine for mfind
//!
//! This crate provides the core functionality for file indexing and searching:
//! - High-performance file system scanning
//! - FST-based index for memory-efficient storage
//! - Real-time monitoring via FSEvents
//! - Flexible query parsing and execution

pub mod index;
pub mod query;
pub mod fs;
pub mod storage;
pub mod event;
pub mod util;

// Re-export main types
pub use index::{IndexEngine, IndexConfig, IndexStats};
pub use query::{Query, QueryParser, SearchOptions};
pub use fs::{FileSystemMonitor, FileSystemScanner, FSEvent};
pub use storage::Storage;
pub use event::EventBatch;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Result type alias
pub type Result<T> = anyhow::Result<T>;

/// Get the default index file path for all mfind components
pub fn get_default_index_path() -> std::path::PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~/.config"))
        .join("mfind");
    config_dir.join("index.mfind")
}
