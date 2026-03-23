//! Metadata cache for file information

use std::time::{Duration, SystemTime};

use dashmap::DashMap;

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
    pub fn with_capacity(_capacity: usize) -> Self {
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

    /// Iterate over all entries
    pub fn iter(&self) -> impl Iterator<Item = (u64, FileMetadata)> + '_ {
        self.cache.iter().map(|r| (*r.key(), r.value().clone()))
    }

    /// Export to bytes
    pub fn to_bytes(&self) -> crate::Result<Vec<u8>> {
        use bincode::Options;
        // Convert to serializable format (inode -> (size, secs, nanos, is_dir))
        let entries: Vec<(u64, (u64, u64, u32, bool))> = self.iter()
            .map(|(inode, meta)| {
                let duration = meta.modified.duration_since(SystemTime::UNIX_EPOCH).unwrap_or(Duration::ZERO);
                (inode, (meta.size, duration.as_secs(), duration.subsec_nanos(), meta.is_dir))
            })
            .collect();
        let data = bincode::DefaultOptions::new().serialize(&entries)?;
        Ok(data)
    }

    /// Import from bytes
    pub fn from_bytes(data: &[u8]) -> crate::Result<Self> {
        use bincode::Options;
        let entries: Vec<(u64, (u64, u64, u32, bool))> = bincode::DefaultOptions::new().deserialize(data)?;
        let cache = Self::new();
        for (inode, (size, secs, nanos, is_dir)) in entries {
            let modified = SystemTime::UNIX_EPOCH + Duration::new(secs, nanos);
            cache.insert(inode, FileMetadata { size, modified, is_dir });
        }
        Ok(cache)
    }
}

impl Default for MetaCache {
    fn default() -> Self {
        Self::new()
    }
}
