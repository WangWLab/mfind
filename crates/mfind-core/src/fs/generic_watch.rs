//! Generic filesystem watcher for unsupported platforms
//!
//! Uses the notify crate's recommended watcher which will use
//! the best available backend for the platform.

use crate::fs::watcher::{FSEvent, FSEventType};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Generic filesystem watcher
pub struct GenericWatcher {
    watcher: RecommendedWatcher,
    running: Arc<AtomicBool>,
    roots: Vec<PathBuf>,
}

impl GenericWatcher {
    /// Create a new generic watcher
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

/// Create a new generic watcher
pub fn create_watcher(tx: mpsc::Sender<FSEvent>) -> Result<GenericWatcher, notify::Error> {
    GenericWatcher::new(tx)
}
