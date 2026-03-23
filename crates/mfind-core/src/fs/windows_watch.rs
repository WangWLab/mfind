//! Windows ReadDirectoryChangesW-based filesystem watcher
//!
//! Uses the notify crate which internally uses ReadDirectoryChangesW on Windows.

use crate::fs::watcher::{FSEvent, FSEventType};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Windows filesystem watcher using ReadDirectoryChangesW
pub struct WindowsWatcher {
    watcher: RecommendedWatcher,
    running: Arc<AtomicBool>,
    roots: Vec<PathBuf>,
}

impl WindowsWatcher {
    /// Create a new Windows watcher
    pub fn new(tx: mpsc::Sender<FSEvent>) -> Result<Self, notify::Error> {
        let watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    let fs_event = convert_event(event);
                    let _ = tx.blocking_send(fs_event);
                }
            },
        )?;

        Ok(Self {
            watcher,
            running: Arc::new(AtomicBool::new(false)),
            roots: Vec::new(),
        })
    }

    /// Start watching a path
    pub fn watch(&mut self, path: &Path, recursive: bool) -> Result<(), notify::Error> {
        let mode = if recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
        self.watcher.watch(path, mode)?;
        self.roots.push(path.to_path_buf());
        Ok(())
    }

    /// Stop watching a path
    pub fn unwatch(&mut self, path: &Path) -> Result<(), notify::Error> {
        self.watcher.unwatch(path)?;
        self.roots.retain(|p| p != path);
        Ok(())
    }

    /// Get watched roots
    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }

    /// Check if watcher is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

/// Convert notify event to FSEvent
fn convert_event(event: notify::Event) -> FSEvent {
    let event_type = match event.kind {
        EventKind::Create(_) => FSEventType::Create,
        EventKind::Remove(_) => FSEventType::Delete,
        EventKind::Modify(_) => FSEventType::Modify,
        EventKind::Rename { .. } => FSEventType::Rename,
        _ => FSEventType::Other,
    };

    FSEvent {
        path: event.paths.into_iter().next().unwrap_or_default(),
        event_type,
        timestamp: std::time::SystemTime::now(),
    }
}

/// Create a new Windows watcher
pub fn create_watcher(tx: mpsc::Sender<FSEvent>) -> Result<WindowsWatcher, notify::Error> {
    WindowsWatcher::new(tx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use tokio::time::{sleep, timeout};
    use std::time::Duration;

    #[tokio::test]
    async fn test_windows_watcher_create() {
        let (tx, mut rx) = mpsc::channel(100);
        let mut watcher = WindowsWatcher::new(tx).expect("Failed to create watcher");

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let test_file = temp_dir.path().join("test.txt");

        watcher.watch(temp_dir.path(), true).expect("Failed to watch");

        // Create a file
        fs::write(&test_file, "test").expect("Failed to write file");

        // Wait for event
        let result = timeout(Duration::from_secs(2), rx.recv()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_windows_watcher_delete() {
        let (tx, mut rx) = mpsc::channel(100);
        let mut watcher = WindowsWatcher::new(tx).expect("Failed to create watcher");

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let test_file = temp_dir.path().join("test.txt");

        // Create file first
        fs::write(&test_file, "test").expect("Failed to write file");
        sleep(Duration::from_millis(100)).await;

        watcher.watch(temp_dir.path(), true).expect("Failed to watch");

        // Delete the file
        fs::remove_file(&test_file).expect("Failed to delete file");

        // Wait for event
        let result = timeout(Duration::from_secs(2), rx.recv()).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_windows_watcher_roots() {
        let (tx, _) = mpsc::channel(100);
        let mut watcher = WindowsWatcher::new(tx).expect("Failed to create watcher");

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        watcher.watch(temp_dir.path(), true).expect("Failed to watch");

        assert_eq!(watcher.roots().len(), 1);
    }
}
