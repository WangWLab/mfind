//! Filesystem event types

use std::path::PathBuf;
use std::time::SystemTime;

/// Filesystem event type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FSEventType {
    /// File or directory created
    Create,
    /// File or directory deleted
    Delete,
    /// File modified
    Modify,
    /// File or directory renamed
    Rename { from: PathBuf, to: PathBuf },
    /// Metadata changed
    Metadata,
}

/// Filesystem event
#[derive(Debug, Clone)]
pub struct FSEvent {
    /// Path involved in the event
    pub path: PathBuf,
    /// Type of event
    pub event_type: FSEventType,
    /// File inode if available
    pub inode: Option<u64>,
    /// Event timestamp
    pub timestamp: SystemTime,
    /// Is a directory
    pub is_dir: bool,
}

impl FSEvent {
    /// Create a new event
    pub fn new(path: PathBuf, event_type: FSEventType) -> Self {
        Self {
            path,
            event_type,
            inode: None,
            timestamp: SystemTime::now(),
            is_dir: false,
        }
    }

    /// Set inode
    pub fn with_inode(mut self, inode: u64) -> Self {
        self.inode = Some(inode);
        self
    }

    /// Set is_dir flag
    pub fn with_is_dir(mut self, is_dir: bool) -> Self {
        self.is_dir = is_dir;
        self
    }
}
