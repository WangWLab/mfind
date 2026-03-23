//! Filesystem module for scanning and monitoring

pub mod backend;
pub mod monitor;
pub mod scanner;
pub mod watcher;

#[cfg(target_os = "macos")]
pub mod fsevents;
#[cfg(target_os = "macos")]
pub mod native_fsevents;

#[cfg(target_os = "linux")]
pub mod inotify;
#[cfg(target_os = "linux")]
pub use inotify::{create_watcher, InotifyWatcher};

#[cfg(target_os = "windows")]
pub mod windows_watch;
#[cfg(target_os = "windows")]
pub use windows_watch::{create_watcher, WindowsWatcher};

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub mod generic_watch;
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub use generic_watch::{create_watcher, GenericWatcher};

pub use backend::{FileSystemBackend, FileSystemType, FileSystemInfo};
#[cfg(target_os = "macos")]
pub use fsevents::{create_watcher as create_macos_watcher, FSEventsWatcher};
#[cfg(target_os = "macos")]
pub use native_fsevents::{create_native_watcher, NativeFSEventsWatcher};
#[cfg(not(target_os = "macos"))]
pub use watcher::create_watcher;
pub use monitor::{FileSystemMonitor, MonitorConfig, MonitorStatus};
pub use scanner::{FileSystemScanner, ScannerConfig, ScanEntry};
pub use watcher::{FSEvent, FSEventType};
