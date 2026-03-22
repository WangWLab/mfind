//! Filesystem module for scanning and monitoring

pub mod backend;
pub mod monitor;
pub mod scanner;
pub mod watcher;

#[cfg(target_os = "macos")]
pub mod fsevents;

pub use backend::{FileSystemBackend, FileSystemType, FileSystemInfo};
#[cfg(target_os = "macos")]
pub use fsevents::{create_watcher, FSEventsWatcher};
pub use monitor::{FileSystemMonitor, MonitorConfig, MonitorStatus};
pub use scanner::{FileSystemScanner, ScannerConfig, ScanEntry};
pub use watcher::{FSEvent, FSEventType};
