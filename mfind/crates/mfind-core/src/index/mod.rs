//! Index module for file metadata storage and retrieval

pub mod engine;
pub mod fst_index;
pub mod inode_map;
pub mod meta_cache;
pub mod stats;

pub use engine::{IndexEngine, IndexConfig, BuildConfig};
pub use fst_index::FSTIndex;
pub use inode_map::InodeMap;
pub use meta_cache::MetaCache;
pub use stats::{IndexStats, IndexHealth};
