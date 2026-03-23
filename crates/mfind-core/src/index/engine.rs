//! Index engine trait and implementation

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use async_trait::async_trait;

use crate::event::FSEvent;
use crate::query::{Query, SearchResult, QueryNode};
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
    #[cfg(target_os = "macos")]
    monitor_running: Arc<AtomicBool>,
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
            #[cfg(target_os = "macos")]
            monitor_running: Arc::new(AtomicBool::new(false)),
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

    /// Start background filesystem monitoring (macOS only)
    #[cfg(target_os = "macos")]
    pub async fn start_monitoring(&mut self, roots: &[PathBuf]) -> Result<()> {
        use crate::fs::{NativeFSEventsWatcher, FileSystemMonitor, MonitorConfig};

        if self.monitor_running.load(Ordering::SeqCst) {
            return Ok(()); // Already running
        }

        let config = MonitorConfig::default();
        let mut watcher = NativeFSEventsWatcher::new(config)?;
        watcher.start(roots).await?;

        let event_receiver = watcher.event_stream();
        self.monitor_running.store(true, Ordering::SeqCst);

        // Spawn background task to process events
        let running = self.monitor_running.clone();
        tokio::spawn(async move {
            while running.load(Ordering::SeqCst) {
                // Collect events with batching
                let mut events = Vec::new();

                // Collect events for up to 100ms or until we have 100 events
                loop {
                    tokio::select! {
                        Ok(event) = event_receiver.recv_async() => {
                            events.push(event);
                            if events.len() >= 100 {
                                break;
                            }
                        }
                        _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                            break;
                        }
                    }
                }

                // Process batch of events
                // Note: We can't update the index here directly since we don't have &mut self
                // In a real implementation, you'd want to use a channel to send events
                // to the IndexEngine or use interior mutability
                if !events.is_empty() {
                    tracing::debug!("Received {} filesystem events", events.len());
                    // Events would be processed here in a full implementation
                }
            }
        });

        Ok(())
    }

    /// Stop background monitoring (macOS only)
    #[cfg(target_os = "macos")]
    pub async fn stop_monitoring(&mut self) -> Result<()> {
        self.monitor_running.store(false, Ordering::SeqCst);
        Ok(())
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
                    crate::index::meta_cache::FileMetadata {
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
        use crate::event::FSEventType;

        let mut modified = false;

        for event in events {
            let path_bytes = event.path.to_string_lossy().as_bytes().to_vec();

            match &event.event_type {
                FSEventType::Create => {
                    // Add new path to FST index
                    if let Ok(new_fst) = self.fst_index.insert_and_rebuild(&path_bytes) {
                        self.fst_index = new_fst;
                        modified = true;
                    }

                    // Update inode map
                    if let Some(inode) = event.inode {
                        self.inode_map.insert(inode, event.path.clone());
                    }

                    // Update metadata cache
                    if self.config.index_metadata {
                        if let Ok(metadata) = std::fs::metadata(&event.path) {
                            self.meta_cache.insert(
                                event.inode.unwrap_or(0),
                                crate::index::meta_cache::FileMetadata {
                                    size: metadata.len(),
                                    modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                                    is_dir: metadata.is_dir(),
                                },
                            );
                        }
                    }
                }
                FSEventType::Delete => {
                    // Remove from FST index
                    if let Ok(new_fst) = self.fst_index.remove_and_rebuild(&path_bytes) {
                        self.fst_index = new_fst;
                        modified = true;
                    }

                    // Remove from inode map
                    if let Some(inode) = event.inode {
                        self.inode_map.remove(inode);
                    }

                    // Remove from metadata cache
                    if let Some(inode) = event.inode {
                        self.meta_cache.remove(inode);
                    }

                    // Update stats
                    if self.stats.total_files > 0 {
                        self.stats.total_files -= 1;
                    }
                }
                FSEventType::Modify | FSEventType::Metadata => {
                    // Update metadata cache for modifications
                    if self.config.index_metadata {
                        if let Ok(metadata) = std::fs::metadata(&event.path) {
                            self.meta_cache.insert(
                                event.inode.unwrap_or(0),
                                crate::index::meta_cache::FileMetadata {
                                    size: metadata.len(),
                                    modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                                    is_dir: metadata.is_dir(),
                                },
                            );
                        }
                    }
                }
                FSEventType::Rename { from, to } => {
                    // Remove old path
                    let from_bytes = from.to_string_lossy().as_bytes().to_vec();
                    if let Ok(new_fst) = self.fst_index.remove_and_rebuild(&from_bytes) {
                        self.fst_index = new_fst;
                    }

                    // Add new path
                    let to_bytes = to.to_string_lossy().as_bytes().to_vec();
                    if let Ok(new_fst) = self.fst_index.insert_and_rebuild(&to_bytes) {
                        self.fst_index = new_fst;
                        modified = true;
                    }

                    // Update inode map (try to preserve inode if available)
                    if let Some(inode) = event.inode {
                        self.inode_map.remove(inode);
                        self.inode_map.insert(inode, to.clone());
                    }

                    // Update metadata cache
                    if self.config.index_metadata {
                        if let Ok(metadata) = std::fs::metadata(to) {
                            self.meta_cache.insert(
                                event.inode.unwrap_or(0),
                                crate::index::meta_cache::FileMetadata {
                                    size: metadata.len(),
                                    modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                                    is_dir: metadata.is_dir(),
                                },
                            );
                        }
                    }
                }
            }
        }

        // Update timestamp if modified
        if modified {
            self.stats.last_update = Some(SystemTime::now());
            self.stats.health = IndexHealth::Healthy;
        }

        Ok(self.stats.clone())
    }

    fn search(&self, query: &Query) -> Result<SearchResult> {
        use crate::query::ast::Pattern;

        let matches = match &query.root {
            QueryNode::Filename { pattern, .. } => {
                match pattern {
                    Pattern::Prefix(prefix) => {
                        // For prefix search, we need to find paths containing the prefix
                        // since paths are stored as full paths
                        let all = self.fst_index.stream();
                        all.into_iter()
                            .filter(|path| {
                                // Check if path contains the prefix or the filename starts with prefix
                                path.contains(prefix) ||
                                std::path::Path::new(path)
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .map(|name| name.starts_with(prefix))
                                    .unwrap_or(false)
                            })
                            .collect()
                    }
                    Pattern::Regex(regex) => {
                        // Use FST regex search
                        self.fst_index.regex_search(regex)?
                    }
                    Pattern::Wildcard(w) => {
                        // Convert wildcard to regex and search
                        let regex = Pattern::wildcard_to_regex(w);
                        self.fst_index.regex_search(&regex)?
                    }
                    Pattern::Exact(s) => {
                        // Exact match - check if path exists
                        let path_bytes = s.as_bytes();
                        if self.fst_index.contains(path_bytes) {
                            vec![s.clone()]
                        } else {
                            vec![]
                        }
                    }
                }
            }
            QueryNode::Extension { ext } => {
                // Filter paths by extension
                let ext_pattern = format!(".{}", ext);
                self.fst_index.stream()
                    .into_iter()
                    .filter(|path| path.ends_with(&ext_pattern))
                    .collect()
            }
            _ => {
                // Default: return all entries
                self.fst_index.stream()
            }
        };

        let total = matches.len();
        Ok(SearchResult {
            matches,
            total,
            time_ms: 0,
        })
    }

    fn search_stream(
        &self,
        _query: &Query,
    ) -> flume::Receiver<Result<crate::query::SearchResultItem>> {
        let (tx, rx) = flume::unbounded();

        // TODO: Implement streaming search
        let _ = tx.send(Err(anyhow::anyhow!("Not implemented")));

        rx
    }

    async fn export(&self) -> Result<Vec<u8>> {
        use bincode::Options;

        // Export FST index
        let fst_data = self.fst_index.to_bytes()?;

        // Export inode map
        let inode_data = self.inode_map.to_bytes()?;

        // Export meta cache
        let meta_data = self.meta_cache.to_bytes()?;

        // Export stats
        let stats_data = bincode::DefaultOptions::new().serialize(&self.stats)?;

        // Combine all parts: [fst_len, fst_data, inode_len, inode_data, meta_len, meta_data, stats_len, stats_data]
        let mut buffer = Vec::new();

        // Write FST
        buffer.extend_from_slice(&(fst_data.len() as u64).to_le_bytes());
        buffer.extend_from_slice(&fst_data);

        // Write inode map
        buffer.extend_from_slice(&(inode_data.len() as u64).to_le_bytes());
        buffer.extend_from_slice(&inode_data);

        // Write meta cache
        buffer.extend_from_slice(&(meta_data.len() as u64).to_le_bytes());
        buffer.extend_from_slice(&meta_data);

        // Write stats
        buffer.extend_from_slice(&(stats_data.len() as u64).to_le_bytes());
        buffer.extend_from_slice(&stats_data);

        Ok(buffer)
    }

    async fn import(&mut self, data: &[u8]) -> Result<()> {
        use bincode::Options;

        let mut offset = 0;

        // Read FST
        let fst_len = u64::from_le_bytes(data[offset..offset + 8].try_into()?) as usize;
        offset += 8;
        let fst_data = &data[offset..offset + fst_len];
        self.fst_index = FSTIndex::from_bytes(fst_data)?;
        offset += fst_len;

        // Read inode map
        let inode_len = u64::from_le_bytes(data[offset..offset + 8].try_into()?) as usize;
        offset += 8;
        let inode_data = &data[offset..offset + inode_len];
        self.inode_map = InodeMap::from_bytes(inode_data)?;
        offset += inode_len;

        // Read meta cache
        let meta_len = u64::from_le_bytes(data[offset..offset + 8].try_into()?) as usize;
        offset += 8;
        let meta_data = &data[offset..offset + meta_len];
        self.meta_cache = MetaCache::from_bytes(meta_data)?;
        offset += meta_len;

        // Read stats
        let stats_len = u64::from_le_bytes(data[offset..offset + 8].try_into()?) as usize;
        offset += 8;
        let stats_data = &data[offset..offset + stats_len];
        self.stats = bincode::DefaultOptions::new().deserialize(stats_data)?;

        self.built = true;
        Ok(())
    }

    fn stats(&self) -> IndexStats {
        self.stats.clone()
    }

    fn health_check(&self) -> IndexHealth {
        IndexHealth::Healthy
    }
}
