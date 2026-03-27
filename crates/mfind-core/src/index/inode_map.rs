//! Inode to path mapping

use std::path::PathBuf;

use dashmap::DashMap;

/// Map inode numbers to file paths
#[derive(Clone)]
pub struct InodeMap {
    map: DashMap<u64, PathBuf>,
}

impl InodeMap {
    /// Create a new inode map
    pub fn new() -> Self {
        Self {
            map: DashMap::new(),
        }
    }

    /// Insert an inode mapping
    pub fn insert(&self, inode: u64, path: PathBuf) {
        self.map.insert(inode, path);
    }

    /// Get path by inode
    pub fn get(&self, inode: u64) -> Option<PathBuf> {
        self.map.get(&inode).map(|r| r.clone())
    }

    /// Remove an inode mapping
    pub fn remove(&self, inode: u64) -> Option<PathBuf> {
        self.map.remove(&inode).map(|(_, v)| v)
    }

    /// Check if inode exists
    pub fn contains(&self, inode: u64) -> bool {
        self.map.contains_key(&inode)
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Clear all entries
    pub fn clear(&self) {
        self.map.clear();
    }

    /// Iterate over all entries
    pub fn iter(&self) -> impl Iterator<Item = (u64, PathBuf)> + '_ {
        self.map.iter().map(|r| (*r.key(), r.value().clone()))
    }

    /// Export to bytes (simple binary format: count + [inode + path_len + path]...)
    pub fn to_bytes(&self) -> crate::Result<Vec<u8>> {
        use bincode::Options;
        let entries: Vec<(u64, String)> = self.iter()
            .map(|(inode, path)| (inode, path.to_string_lossy().to_string()))
            .collect();
        let data = bincode::DefaultOptions::new().serialize(&entries)?;
        Ok(data)
    }

    /// Import from bytes
    pub fn from_bytes(data: &[u8]) -> crate::Result<Self> {
        use bincode::Options;
        let entries: Vec<(u64, String)> = bincode::DefaultOptions::new().deserialize(data)?;
        let map = Self::new();
        for (inode, path) in entries {
            map.insert(inode, PathBuf::from(path));
        }
        Ok(map)
    }
}

impl Default for InodeMap {
    fn default() -> Self {
        Self::new()
    }
}
