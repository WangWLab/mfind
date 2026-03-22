//! Storage trait

use crate::Result;

/// Storage trait for index persistence
pub trait Storage: Send + Sync {
    /// Get value by key
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;

    /// Put key-value pair
    fn put(&self, key: &[u8], value: &[u8]) -> Result<()>;

    /// Delete key
    fn delete(&self, key: &[u8]) -> Result<()>;

    /// Check if key exists
    fn contains(&self, key: &[u8]) -> Result<bool>;

    /// Iterate over all keys
    fn iter(&self) -> Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>)>> + '_>;

    /// Get number of entries
    fn len(&self) -> Result<usize>;

    /// Check if empty
    fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    /// Clear all data
    fn clear(&self) -> Result<()>;

    /// Flush to disk
    fn flush(&self) -> Result<()>;
}
