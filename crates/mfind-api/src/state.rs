//! API state management

use std::sync::Arc;
use std::time::Instant;

use mfind_core::index::IndexEngine;

/// Shared API state
#[derive(Clone)]
pub struct ApiState {
    /// Reference to the index engine
    pub engine: Arc<tokio::sync::RwLock<IndexEngine>>,
    /// Server start time
    pub start_time: Arc<Instant>,
    /// API version
    pub version: Arc<String>,
}

impl ApiState {
    /// Create a new API state
    pub fn new(engine: Arc<tokio::sync::RwLock<IndexEngine>>) -> Self {
        Self {
            engine,
            start_time: Arc::new(Instant::now()),
            version: Arc::new(env!("CARGO_PKG_VERSION").to_string()),
        }
    }

    /// Get server uptime in seconds
    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}
