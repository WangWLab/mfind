//! Filesystem monitor trait

use std::path::PathBuf;

use async_trait::async_trait;

use crate::event::FSEvent;
use crate::Result;

/// Monitor configuration
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// Batch window for event collection
    pub batch_window_ms: u64,
    /// Batch size threshold
    pub batch_size: usize,
    /// Deduplication window
    pub dedup_window_ms: u64,
    /// Buffer size for events
    pub buffer_size: usize,
    /// Recursive monitoring
    pub recursive: bool,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            batch_window_ms: 100,
            batch_size: 100,
            dedup_window_ms: 50,
            buffer_size: 1000,
            recursive: true,
        }
    }
}

/// Monitor status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorStatus {
    Stopped,
    Running,
    Paused,
    Error,
}

/// Filesystem monitor trait
#[async_trait]
pub trait FileSystemMonitor: Send + Sync {
    /// Start monitoring paths
    async fn start(&mut self, paths: &[PathBuf]) -> Result<()>;

    /// Stop monitoring
    async fn stop(&mut self) -> Result<()>;

    /// Pause monitoring
    async fn pause(&mut self) -> Result<()>;

    /// Resume monitoring
    async fn resume(&mut self) -> Result<()>;

    /// Get event stream
    fn event_stream(&self) -> flume::Receiver<FSEvent>;

    /// Get monitor status
    fn status(&self) -> MonitorStatus;
}
