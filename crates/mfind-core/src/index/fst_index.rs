//! FST (Finite State Transducer) based index for memory-efficient string storage

use fst::{Set, Streamer};

use crate::Result;

/// FST-based string index
pub struct FSTIndex {
    set: Set<Vec<u8>>,
}

impl FSTIndex {
    /// Create an empty FST index
    pub fn new() -> Result<Self> {
        let set = Set::from_iter(std::iter::empty::<&[u8]>())?;
        Ok(Self { set })
    }

    /// Build FST index from sorted paths
    pub fn build(paths: &[Vec<u8>]) -> Result<Self> {
        let mut builder = fst::SetBuilder::memory();

        for path in paths {
            builder.insert(path)?;
        }

        let data = builder.into_inner()?;
        let set = Set::new(data)?;

        Ok(Self { set })
    }

    /// Insert a single path
    pub fn insert(&mut self, path: &[u8]) -> Result<()> {
        // FST is immutable, so we need to rebuild
        // For production, use a different approach
        let mut paths: Vec<Vec<u8>> = self
            .stream()
            .into_iter()
            .map(|s| s.into_bytes())
            .collect();
        paths.push(path.to_vec());
        paths.sort();

        let mut builder = fst::SetBuilder::memory();
        for p in &paths {
            builder.insert(p)?;
        }

        let data = builder.into_inner()?;
        self.set = Set::new(data)?;

        Ok(())
    }

    /// Remove a path
    pub fn remove(&mut self, path: &[u8]) -> Result<()> {
        let paths: Vec<Vec<u8>> = self
            .stream()
            .into_iter()
            .map(|s| s.into_bytes())
            .filter(|p| p.as_slice() != path)
            .collect();

        let mut builder = fst::SetBuilder::memory();
        for p in &paths {
            builder.insert(p)?;
        }

        let data = builder.into_inner()?;
        self.set = Set::new(data)?;

        Ok(())
    }

    /// Prefix search
    pub fn prefix_search(&self, prefix: &str) -> Result<Vec<String>> {
        let prefix_bytes = prefix.as_bytes();

        // Find the range of keys starting with prefix
        let mut results = Vec::new();
        let mut stream = self.set.stream();
        while let Some(key) = stream.next() {
            if key.starts_with(prefix_bytes) {
                if let Ok(s) = std::str::from_utf8(key) {
                    results.push(s.to_string());
                }
            }
        }

        Ok(results)
    }

    /// Regex search
    pub fn regex_search(&self, pattern: &regex::Regex) -> Result<Vec<String>> {
        let mut results = Vec::new();
        let mut stream = self.set.stream();
        while let Some(key) = stream.next() {
            if let Ok(s) = std::str::from_utf8(key) {
                if pattern.is_match(s) {
                    results.push(s.to_string());
                }
            }
        }

        Ok(results)
    }

    /// Check if a path exists
    pub fn contains(&self, path: &[u8]) -> bool {
        self.set.contains(path)
    }

    /// Get the number of entries
    pub fn len(&self) -> u64 {
        self.set.len() as u64
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    /// Get all entries as a stream
    pub fn stream(&self) -> Vec<String> {
        let mut results = Vec::new();
        let mut stream = self.set.stream();
        while let Some(key) = stream.next() {
            if let Ok(s) = std::str::from_utf8(key) {
                results.push(s.to_string());
            }
        }
        results
    }

    /// Get memory usage estimate
    pub fn memory_usage(&self) -> usize {
        self.set.as_fst().as_bytes().len()
    }

    /// Export to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.set.as_fst().as_bytes().to_vec())
    }

    /// Import from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let set = Set::new(data.to_vec())?;
        Ok(Self { set })
    }
}

impl Default for FSTIndex {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_and_search() {
        let paths = vec![
            b"/Users/test/file1.txt".to_vec(),
            b"/Users/test/file2.txt".to_vec(),
            b"/Users/test/file3.rs".to_vec(),
        ];

        let index = FSTIndex::build(&paths).unwrap();

        let results = index.prefix_search("/Users/test/file").unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_prefix_search() {
        // Paths must be in lexicographic order for FST
        let paths = vec![
            b"/app/lib.rs".to_vec(),
            b"/app/main.rs".to_vec(),
            b"/src/main.rs".to_vec(),
        ];

        let index = FSTIndex::build(&paths).unwrap();

        let results = index.prefix_search("/app").unwrap();
        assert_eq!(results.len(), 2);
    }
}
