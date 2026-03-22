//! Index engine trait and implementation

use std::path::PathBuf;
use std::time::Duration;

use async_trait::async_trait;

use crate::event::FSEvent;
use crate::query::{Query, SearchResult};
use crate::Result;

use super::{IndexStats, IndexHealth, FSTIndex, InodeMap, MetaCache};

/// Index engine configuration
#[derive(Debug, Clone)]
pub struct IndexConfig {
    /// Memory limit in bytes
    pub memory_limit: Option<usize>,
    /// Parallelism level
    pub parallelism: usize,
    /// Exclude patterns
    pub exclude_patterns: Vec<String>,
    /// Include hidden files
    pub include_hidden: bool,
    /// Respect .gitignore
    pub gitignore_ignore: bool,
    /// Follow symlinks
    pub follow_symlinks: bool,
    /// Index metadata
    pub index_metadata: bool,
    /// Index extended attributes
    pub index_xattr: bool,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            memory_limit: None,
            parallelism: num_cpus::get(),
            exclude_patterns: vec![],
            include_hidden: false,
            gitignore_ignore: true,
            follow_symlinks: false,
            index_metadata: true,
            index_xattr: false,
        }
    }
}

/// Build configuration for index construction
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Root paths to index
    pub roots: Vec<PathBuf>,
    /// Incremental build from existing index
    pub incremental: bool,
}

/// Index engine trait
#[async_trait]
pub trait IndexEngineTrait: Send + Sync {
    /// Build index from scratch
    async fn build(&mut self, roots: &[PathBuf]) -> Result<IndexStats>;

    /// Update index with filesystem events
    async fn update(&mut self, events: &[FSEvent]) -> Result<IndexStats>;

    /// Search the index
    fn search(&self, query: &Query) -> Result<SearchResult>;

    /// Search with streaming results
    fn search_stream(
        &self,
        query: &Query,
    ) -> flume::Receiver<Result<crate::query::SearchResultItem>>;

    /// Export index to bytes
    async fn export(&self) -> Result<Vec<u8>>;

    /// Import index from bytes
    async fn import(&mut self, data: &[u8]) -> Result<()>;

    /// Get index statistics
    fn stats(&self) -> IndexStats;

    /// Check index health
    fn health_check(&self) -> IndexHealth;
}

/// Main index engine implementation
pub struct IndexEngine {
    config: IndexConfig,
    fst_index: FSTIndex,
    inode_map: InodeMap,
    meta_cache: MetaCache,
    stats: IndexStats,
    built: bool,
}

impl IndexEngine {
    /// Create a new index engine
    pub fn new(config: IndexConfig) -> Result<Self> {
        Ok(Self {
            config,
            fst_index: FSTIndex::new()?,
            inode_map: InodeMap::new(),
            meta_cache: MetaCache::new(),
            stats: IndexStats::default(),
            built: false,
        })
    }

    /// Get configuration reference
    pub fn config(&self) -> &IndexConfig {
        &self.config
    }

    /// Check if index is built
    pub fn is_built(&self) -> bool {
        self.built
    }
}

#[async_trait]
impl IndexEngineTrait for IndexEngine {
    async fn build(&mut self, roots: &[PathBuf]) -> Result<IndexStats> {
        use crate::fs::{FileSystemScanner, ScannerConfig};

        let config = ScannerConfig {
            parallelism: self.config.parallelism,
            gitignore_ignore: self.config.gitignore_ignore,
            include_hidden: self.config.include_hidden,
            follow_symlinks: self.config.follow_symlinks,
            exclude_patterns: self.config.exclude_patterns.clone(),
        };

        let scanner = FileSystemScanner::new(config);
        let entries = scanner.scan(roots).await?;

        // Build FST index
        let mut paths: Vec<Vec<u8>> = entries
            .iter()
            .map(|e| e.path.to_string_lossy().bytes().collect())
            .collect();
        paths.sort();

        self.fst_index = FSTIndex::build(&paths)?;

        // Populate inode map and metadata cache
        for entry in entries {
            let inode = entry.inode.unwrap_or(0);
            self.inode_map.insert(inode, entry.path.clone());

            if self.config.index_metadata {
                self.meta_cache.insert(
                    inode,
                    crate::query::FileMetadata {
                        size: entry.size,
                        modified: entry.modified,
                        is_dir: entry.is_dir,
                    },
                );
            }
        }

        self.stats = IndexStats {
            total_files: paths.len() as u64,
            build_time: Duration::from_secs(1), // TODO: measure properly
            last_update: Some(std::time::SystemTime::now()),
            health: IndexHealth::Healthy,
            ..Default::default()
        };

        self.built = true;
        Ok(self.stats.clone())
    }

    async fn update(&mut self, events: &[FSEvent]) -> Result<IndexStats> {
        // TODO: Implement incremental update
        for event in events {
            match event.event_type {
                crate::event::FSEventType::Create => {
                    self.fst_index.insert(event.path.to_string_lossy().as_bytes())?;
                }
                crate::event::FSEventType::Delete => {
                    self.fst_index
                        .remove(event.path.to_string_lossy().as_bytes())?;
                }
                _ => {}
            }
        }
        Ok(self.stats.clone())
    }

    fn search(&self, query: &Query) -> Result<SearchResult> {
        // TODO: Implement full search
        let matches = self.fst_index.prefix_search(&query.pattern)?;
        Ok(SearchResult {
            matches,
            total: matches.len(),
            time_ms: 0,
        })
    }

    fn search_stream(
        &self,
        query: &Query,
    ) -> flume::Receiver<Result<crate::query::SearchResultItem>> {
        let (tx, rx) = flume::unbounded();

        // TODO: Implement streaming search
        let _ = tx.send(Err(anyhow::anyhow!("Not implemented")));

        rx
    }

    async fn export(&self) -> Result<Vec<u8>> {
        // TODO: Implement export
        Ok(vec![])
    }

    async fn import(&mut self, _data: &[u8]) -> Result<()> {
        // TODO: Implement import
        Ok(())
    }

    fn stats(&self) -> IndexStats {
        self.stats.clone()
    }

    fn health_check(&self) -> IndexHealth {
        IndexHealth::Healthy
    }
}
