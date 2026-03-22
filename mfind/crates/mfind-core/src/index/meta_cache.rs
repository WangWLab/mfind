//! Metadata cache for file information

use std::time::SystemTime;

use dashmap::DashMap;
use lru::LruCache;
use std::num::NonZeroUsize;

/// File metadata
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub modified: SystemTime,
    pub is_dir: bool,
}

/// LRU cache for file metadata
pub struct MetaCache {
    cache: DashMap<u64, FileMetadata>,
}

impl MetaCache {
    /// Create a new metadata cache
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
        }
    }

    /// Create with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            cache: DashMap::new(),
        }
    }

    /// Insert metadata
    pub fn insert(&self, inode: u64, metadata: FileMetadata) {
        self.cache.insert(inode, metadata);
    }

    /// Get metadata
    pub fn get(&self, inode: u64) -> Option<FileMetadata> {
        self.cache.get(&inode).map(|r| r.clone())
    }

    /// Remove metadata
    pub fn remove(&self, inode: u64) -> Option<FileMetadata> {
        self.cache.remove(&inode).map(|(_, v)| v)
    }

    /// Check if inode has cached metadata
    pub fn contains(&self, inode: u64) -> bool {
        self.cache.contains_key(&inode)
    }

    /// Get the number of cached entries
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Clear the cache
    pub fn clear(&self) {
        self.cache.clear();
    }
}

impl Default for MetaCache {
    fn default() -> Self {
        Self::new()
    }
}
