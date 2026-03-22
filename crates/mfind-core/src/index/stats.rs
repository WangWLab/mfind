//! Index statistics and health information

use std::time::{Duration, SystemTime};

/// Index statistics
#[derive(Debug, Clone, Default)]
pub struct IndexStats {
    /// Total number of files
    pub total_files: u64,
    /// Total number of directories
    pub total_dirs: u64,
    /// Total number of symlinks
    pub total_symlinks: u64,
    /// Total size in bytes
    pub total_bytes: u64,
    /// Index size in bytes
    pub index_size_bytes: u64,
    /// Time taken to build
    pub build_time: Duration,
    /// Last update time
    pub last_update: Option<SystemTime>,
    /// Health status
    pub health: IndexHealth,
}

/// Index health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexHealth {
    /// Index is healthy and up-to-date
    Healthy,
    /// Index needs refresh
    Stale,
    /// Index may be corrupted
    Corrupted,
    /// Index is being built
    Building,
}

impl std::fmt::Display for IndexHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexHealth::Healthy => write!(f, "Healthy"),
            IndexHealth::Stale => write!(f, "Stale"),
            IndexHealth::Corrupted => write!(f, "Corrupted"),
            IndexHealth::Building => write!(f, "Building"),
        }
    }
}
