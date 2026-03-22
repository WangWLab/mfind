//! In-memory storage implementation

use std::collections::HashMap;
use std::sync::RwLock;

use super::Storage;
use crate::Result;

/// In-memory storage
pub struct MemoryStorage {
    data: RwLock<HashMap<Vec<u8>, Vec<u8>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl Storage for MemoryStorage {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let data = self.data.read().unwrap();
        Ok(data.get(key).cloned())
    }

    fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        let mut data = self.data.write().unwrap();
        data.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn delete(&self, key: &[u8]) -> Result<()> {
        let mut data = self.data.write().unwrap();
        data.remove(key);
        Ok(())
    }

    fn contains(&self, key: &[u8]) -> Result<bool> {
        let data = self.data.read().unwrap();
        Ok(data.contains_key(key))
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>)>> + '_> {
        let data = self.data.read().unwrap();
        let items: Vec<_> = data
            .iter()
            .map(|(k, v)| Ok((k.clone(), v.clone())))
            .collect();
        Box::new(items.into_iter())
    }

    fn len(&self) -> Result<usize> {
        let data = self.data.read().unwrap();
        Ok(data.len())
    }

    fn clear(&self) -> Result<()> {
        let mut data = self.data.write().unwrap();
        data.clear();
        Ok(())
    }

    fn flush(&self) -> Result<()> {
        // No-op for in-memory storage
        Ok(())
    }
}
