//! Storage module for index persistence

pub mod trait_mod;
pub mod memory;

pub use trait_mod::Storage;
pub use memory::MemoryStorage;
