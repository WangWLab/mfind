//! Filesystem module for scanning and monitoring

pub mod backend;
pub mod monitor;
pub mod scanner;
pub mod watcher;

pub use backend::{FileSystemBackend, FileSystemType, FileSystemInfo};
pub use monitor::{FileSystemMonitor, MonitorConfig, MonitorStatus};
pub use scanner::{FileSystemScanner, ScannerConfig, ScanEntry};
pub use watcher::{FSEvent, FSEventType};
