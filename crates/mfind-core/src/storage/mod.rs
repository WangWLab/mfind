//! Storage module for index persistence

pub mod trait_mod;
pub mod memory;
#[cfg(target_os = "macos")]
pub mod lmdb;

pub use trait_mod::Storage;
pub use memory::MemoryStorage;
#[cfg(target_os = "macos")]
pub use lmdb::LmdbStorage;
