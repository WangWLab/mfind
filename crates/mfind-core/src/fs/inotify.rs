//! Linux inotify-based filesystem watcher
//!
//! Uses the notify crate which internally uses inotify on Linux.

use crate::fs::watcher::{FSEvent, FSEventType};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// Linux inotify-based filesystem watcher
pub struct InotifyWatcher {
    watcher: RecommendedWatcher,
    running: Arc<AtomicBool>,
    roots: Vec<PathBuf>,
}

impl InotifyWatcher {
    /// Create a new inotify watcher
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

/// Create a new inotify watcher
pub fn create_watcher(tx: mpsc::Sender<FSEvent>) -> Result<InotifyWatcher, notify::Error> {
    InotifyWatcher::new(tx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use tokio::time::{sleep, timeout};

    #[tokio::test]
    async fn test_inotify_watcher_create() {
        let (tx, mut rx) = mpsc::channel(100);
        let mut watcher = InotifyWatcher::new(tx).expect("Failed to create watcher");

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
    async fn test_inotify_watcher_delete() {
        let (tx, mut rx) = mpsc::channel(100);
        let mut watcher = InotifyWatcher::new(tx).expect("Failed to create watcher");

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

    #[tokio::test]
    async fn test_inotify_watcher_modify() {
        let (tx, mut rx) = mpsc::channel(100);
        let mut watcher = InotifyWatcher::new(tx).expect("Failed to create watcher");

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let test_file = temp_dir.path().join("test.txt");

        // Create file first
        fs::write(&test_file, "test").expect("Failed to write file");
        sleep(Duration::from_millis(100)).await;

        watcher.watch(temp_dir.path(), true).expect("Failed to watch");

        // Modify the file
        fs::write(&test_file, "modified").expect("Failed to modify file");

        // Wait for event
        let result = timeout(Duration::from_secs(2), rx.recv()).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_inotify_watcher_roots() {
        let (tx, _) = mpsc::channel(100);
        let mut watcher = InotifyWatcher::new(tx).expect("Failed to create watcher");

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        watcher.watch(temp_dir.path(), true).expect("Failed to watch");

        assert_eq!(watcher.roots().len(), 1);
        assert!(watcher.roots().contains(&temp_dir.path().to_path_buf()));
    }

    #[test]
    fn test_inotify_watcher_unwatch() {
        let (tx, _) = mpsc::channel(100);
        let mut watcher = InotifyWatcher::new(tx).expect("Failed to create watcher");

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        watcher.watch(temp_dir.path(), true).expect("Failed to watch");
        assert_eq!(watcher.roots().len(), 1);

        watcher.unwatch(temp_dir.path()).expect("Failed to unwatch");
        assert_eq!(watcher.roots().len(), 0);
    }
}
