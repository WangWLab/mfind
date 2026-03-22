//! Event module for filesystem event handling

pub mod batch;
pub mod dedup;

pub use batch::EventBatch;
pub use dedup::EventDeduplicator;
