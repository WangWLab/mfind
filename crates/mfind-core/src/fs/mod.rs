//! Filesystem module for scanning and monitoring

pub mod backend;
pub mod monitor;
pub mod scanner;
pub mod watcher;

#[cfg(target_os = "macos")]
pub mod fsevents;
#[cfg(target_os = "macos")]
pub mod native_fsevents;

pub use backend::{FileSystemBackend, FileSystemType, FileSystemInfo};
#[cfg(target_os = "macos")]
pub use fsevents::{create_watcher, FSEventsWatcher};
#[cfg(target_os = "macos")]
pub use native_fsevents::{create_native_watcher, NativeFSEventsWatcher};
pub use monitor::{FileSystemMonitor, MonitorConfig, MonitorStatus};
pub use scanner::{FileSystemScanner, ScannerConfig, ScanEntry};
pub use watcher::{FSEvent, FSEventType};
