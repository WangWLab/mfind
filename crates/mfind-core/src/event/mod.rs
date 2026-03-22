//! Event module for filesystem event handling

pub mod batch;
pub mod dedup;

pub use batch::EventBatch;
pub use dedup::EventDeduplicator;
// Re-export FSEvent and FSEventType from fs module
pub use crate::fs::{FSEvent, FSEventType};
