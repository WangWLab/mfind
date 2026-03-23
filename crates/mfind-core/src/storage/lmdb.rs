//! LMDB storage implementation for persistent index storage

use std::path::Path;

use lmdb::{Environment, Transaction, Database, WriteFlags, Cursor};

use super::Storage;
use crate::Result;

/// LMDB storage backend
pub struct LmdbStorage {
    env: Environment,
}

impl LmdbStorage {
    /// Create new LMDB storage
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let env = Environment::new()
            .set_max_dbs(10)
            .set_max_readers(126)
            .set_map_size(10 * 1024 * 1024 * 1024) // 10GB max
            .open(path.as_ref())?;

        Ok(Self { env })
    }

    /// Get database handle
    fn get_db(&self) -> Result<Database> {
        Ok(self.env.open_db(None)?)
    }
}

impl Storage for LmdbStorage {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let db = self.get_db()?;
        let txn = self.env.begin_ro_txn()?;

        match txn.get(db, &key) {
            Ok(value) => Ok(Some(value.to_vec())),
            Err(lmdb::Error::NotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        let db = self.get_db()?;
        let mut txn = self.env.begin_rw_txn()?;
        txn.put(db, &key, &value, WriteFlags::empty())?;
        txn.commit()?;
        Ok(())
    }

    fn delete(&self, key: &[u8]) -> Result<()> {
        let db = self.get_db()?;
        let mut txn = self.env.begin_rw_txn()?;
        txn.del(db, &key, None)?;
        txn.commit()?;
        Ok(())
    }

    fn contains(&self, key: &[u8]) -> Result<bool> {
        let db = self.get_db()?;
        let txn = self.env.begin_ro_txn()?;
        match txn.get(db, &key) {
            Ok(_) => Ok(true),
            Err(lmdb::Error::NotFound) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>)>> + '_> {
        let db = match self.get_db() {
            Ok(db) => db,
            Err(e) => return Box::new(std::iter::once(Err(e))),
        };

        let txn = match self.env.begin_ro_txn() {
            Ok(txn) => txn,
            Err(e) => return Box::new(std::iter::once(Err(e.into()))),
        };

        let cursor = match txn.open_ro_cursor(db) {
            Ok(cursor) => cursor,
            Err(e) => return Box::new(std::iter::once(Err(e.into()))),
        };

        // Collect all items into a Vec since we can't return an iterator with lifetime issues
        let mut cursor_owned = cursor;
        let items: Vec<Result<(Vec<u8>, Vec<u8>)>> = {
            let mut result = Vec::new();
            let mut iter = cursor_owned.iter();
            while let Some(result_item) = iter.next() {
                match result_item {
                    Ok((k, v)) => result.push(Ok((k.to_vec(), v.to_vec()))),
                    Err(e) => result.push(Err(e.into())),
                }
            }
            result
        };

        Box::new(items.into_iter())
    }

    fn len(&self) -> Result<usize> {
        let db = self.get_db()?;
        let txn = self.env.begin_ro_txn()?;
        let stat = txn.stat(db)?;
        Ok(stat.entries())
    }

    fn clear(&self) -> Result<()> {
        let db = self.get_db()?;
        let mut txn = self.env.begin_rw_txn()?;
        txn.clear_db(db)?;
        txn.commit()?;
        Ok(())
    }

    fn flush(&self) -> Result<()> {
        // LMDB auto-flushes on commit
        Ok(())
    }
}

impl Drop for LmdbStorage {
    fn drop(&mut self) {
        // LMDB will be properly closed when env is dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_lmdb_basic() {
        let dir = tempdir().unwrap();
        let storage = LmdbStorage::new(dir.path()).unwrap();

        // Test put/get
        storage.put(b"key1", b"value1").unwrap();
        let value = storage.get(b"key1").unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));

        // Test contains
        assert!(storage.contains(b"key1").unwrap());
        assert!(!storage.contains(b"key2").unwrap());

        // Test len
        assert_eq!(storage.len().unwrap(), 1);

        // Test delete
        storage.delete(b"key1").unwrap();
        assert!(!storage.contains(b"key1").unwrap());
        assert_eq!(storage.len().unwrap(), 0);
    }

    #[test]
    fn test_lmdb_iter() {
        let dir = tempdir().unwrap();
        let storage = LmdbStorage::new(dir.path()).unwrap();

        storage.put(b"a", b"1").unwrap();
        storage.put(b"b", b"2").unwrap();
        storage.put(b"c", b"3").unwrap();

        let items: Vec<_> = storage.iter().collect::<Result<Vec<_>>>().unwrap();
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_lmdb_clear() {
        let dir = tempdir().unwrap();
        let storage = LmdbStorage::new(dir.path()).unwrap();

        storage.put(b"a", b"1").unwrap();
        storage.put(b"b", b"2").unwrap();

        storage.clear().unwrap();
        assert_eq!(storage.len().unwrap(), 0);
    }
}
