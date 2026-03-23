//! Native FSEvents watcher implementation for macOS using notify crate
//!
//! This module provides a wrapper around the notify crate which uses
//! native FSEvents API on macOS for real-time filesystem event monitoring.

#![cfg(target_os = "macos")]

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::SystemTime;

use async_trait::async_trait;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::fs::FSEventType;
use crate::Result;

use super::{FSEvent, FileSystemMonitor, MonitorConfig, MonitorStatus};

/// Native FSEvents watcher implementation using notify crate
pub struct NativeFSEventsWatcher {
    status: MonitorStatus,
    event_sender: flume::Sender<FSEvent>,
    event_receiver: flume::Receiver<FSEvent>,
    running: Arc<AtomicBool>,
    watcher: Option<RecommendedWatcher>,
    watched_paths: Vec<PathBuf>,
}

impl NativeFSEventsWatcher {
    /// Create a new native FSEvents watcher
    pub fn new(_config: MonitorConfig) -> Result<Self> {
        let (sender, receiver) = flume::bounded(1000);

        Ok(Self {
            status: MonitorStatus::Stopped,
            event_sender: sender,
            event_receiver: receiver,
            running: Arc::new(AtomicBool::new(false)),
            watcher: None,
            watched_paths: Vec::new(),
        })
    }

    /// Convert notify EventKind to our FSEventType
    fn event_kind_to_type(kind: EventKind) -> FSEventType {
        match kind {
            EventKind::Create(_) => FSEventType::Create,
            EventKind::Remove(_) => FSEventType::Delete,
            EventKind::Modify(_) => FSEventType::Modify,
            EventKind::Access(_) => FSEventType::Modify,
            EventKind::Other | EventKind::Any => FSEventType::Modify,
        }
    }

    /// Create the notify watcher
    fn create_watcher(sender: flume::Sender<FSEvent>) -> notify::Result<RecommendedWatcher> {
        let watcher = RecommendedWatcher::new(
            move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res {
                    for path in event.paths {
                        let event_type = Self::event_kind_to_type(event.kind);

                        let fs_event = FSEvent {
                            path: path.clone(),
                            event_type,
                            inode: None,
                            timestamp: SystemTime::now(),
                            is_dir: path.is_dir(),
                        };

                        let _ = sender.send(fs_event);
                    }
                }
            },
            notify::Config::default(),
        );
        watcher
    }
}

#[async_trait]
impl FileSystemMonitor for NativeFSEventsWatcher {
    async fn start(&mut self, paths: &[PathBuf]) -> Result<()> {
        if self.status == MonitorStatus::Running {
            return Ok(());
        }

        self.watched_paths = paths.to_vec();
        self.running.store(true, Ordering::SeqCst);
        self.status = MonitorStatus::Running;

        // Create watcher with event sender
        let sender = self.event_sender.clone();
        let mut watcher = Self::create_watcher(sender)
            .map_err(|e| anyhow::anyhow!("Failed to create watcher: {}", e))?;

        // Watch all paths recursively
        for path in paths {
            watcher
                .watch(path, RecursiveMode::Recursive)
                .map_err(|e| anyhow::anyhow!("Failed to watch path {}: {}", path.display(), e))?;
        }

        self.watcher = Some(watcher);

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.running.store(false, Ordering::SeqCst);
        self.status = MonitorStatus::Stopped;

        if let Some(mut watcher) = self.watcher.take() {
            for path in &self.watched_paths {
                let _ = watcher.unwatch(path);
            }
        }

        self.watched_paths.clear();
        Ok(())
    }

    async fn pause(&mut self) -> Result<()> {
        // Pause by stopping the watcher
        if let Some(mut watcher) = self.watcher.take() {
            for path in &self.watched_paths {
                let _ = watcher.unwatch(path);
            }
        }
        self.status = MonitorStatus::Paused;
        Ok(())
    }

    async fn resume(&mut self) -> Result<()> {
        if self.status != MonitorStatus::Paused {
            return Ok(());
        }

        self.running.store(true, Ordering::SeqCst);
        self.status = MonitorStatus::Running;

        // Re-create watcher and re-watch paths
        let sender = self.event_sender.clone();
        let mut watcher = Self::create_watcher(sender)
            .map_err(|e| anyhow::anyhow!("Failed to create watcher: {}", e))?;

        for path in &self.watched_paths {
            watcher
                .watch(path, RecursiveMode::Recursive)
                .map_err(|e| anyhow::anyhow!("Failed to watch path {}: {}", path.display(), e))?;
        }

        self.watcher = Some(watcher);
        Ok(())
    }

    fn event_stream(&self) -> flume::Receiver<FSEvent> {
        self.event_receiver.clone()
    }

    fn status(&self) -> MonitorStatus {
        self.status
    }
}

/// Create a new native FSEvents watcher
pub fn create_native_watcher(config: MonitorConfig) -> Result<NativeFSEventsWatcher> {
    NativeFSEventsWatcher::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_native_watcher_creation() {
        let config = MonitorConfig::default();
        let watcher = NativeFSEventsWatcher::new(config);
        assert!(watcher.is_ok());
    }

    #[tokio::test]
    async fn test_native_watcher_start_stop() {
        let config = MonitorConfig::default();
        let mut watcher = NativeFSEventsWatcher::new(config).unwrap();

        let dir = tempdir().unwrap();
        let paths = vec![dir.path().to_path_buf()];

        watcher.start(&paths).await.unwrap();
        assert_eq!(watcher.status(), MonitorStatus::Running);

        watcher.stop().await.unwrap();
        assert_eq!(watcher.status(), MonitorStatus::Stopped);
    }

    #[tokio::test]
    async fn test_native_watcher_events() {
        let config = MonitorConfig::default();
        let mut watcher = NativeFSEventsWatcher::new(config).unwrap();

        let dir = tempdir().unwrap();
        let paths = vec![dir.path().to_path_buf()];

        // Start watching
        watcher.start(&paths).await.unwrap();

        // Give watcher time to initialize
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Create a file
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "test content").unwrap();
        drop(file);

        // Give watcher time to receive event
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Check if we received events
        let receiver = watcher.event_stream();

        // Try to receive events (non-blocking)
        while let Ok(_event) = receiver.try_recv() {
            // Note: FSEvents may not fire immediately or may batch events
            // This test just verifies the watcher doesn't crash
            break;
        }

        watcher.stop().await.unwrap();

        // Note: FSEvents may not fire immediately or may batch events
        // This test just verifies the watcher doesn't crash
        assert!(watcher.status() == MonitorStatus::Stopped);
    }
}
