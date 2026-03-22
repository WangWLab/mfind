//! FSEvents watcher implementation for macOS
//!
//! This module provides a wrapper around the FSEvents API for monitoring
//! filesystem changes on macOS.

#![cfg(target_os = "macos")]

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};

use async_trait::async_trait;

use crate::event::EventDeduplicator;
use crate::fs::FSEventType;
use crate::Result;

use super::{FSEvent, FileSystemMonitor, MonitorConfig, MonitorStatus};

/// FSEvents watcher implementation
pub struct FSEventsWatcher {
    config: MonitorConfig,
    status: MonitorStatus,
    event_sender: flume::Sender<FSEvent>,
    event_receiver: flume::Receiver<FSEvent>,
    running: Arc<AtomicBool>,
    paths: Vec<PathBuf>,
    _worker_thread: Option<thread::JoinHandle<()>>,
}

impl FSEventsWatcher {
    /// Create a new FSEvents watcher
    pub fn new(config: MonitorConfig) -> Result<Self> {
        let (sender, receiver) = flume::bounded(config.buffer_size);

        Ok(Self {
            config,
            status: MonitorStatus::Stopped,
            event_sender: sender,
            event_receiver: receiver,
            running: Arc::new(AtomicBool::new(false)),
            paths: Vec::new(),
            _worker_thread: None,
        })
    }

    /// Start monitoring with internal implementation
    fn start_impl(&mut self, paths: &[PathBuf]) -> Result<()> {
        self.paths = paths.to_vec();
        self.running.store(true, Ordering::SeqCst);
        self.status = MonitorStatus::Running;

        let sender = self.event_sender.clone();
        let running = self.running.clone();
        let config = self.config.clone();
        let watched_paths = self.paths.clone();

        // Spawn worker thread for polling-based monitoring
        self._worker_thread = Some(thread::spawn(move || {
            let mut dedup = EventDeduplicator::new(config.dedup_window_ms);
            let poll_interval = Duration::from_millis(config.batch_window_ms);
            let mut previous_state: std::collections::HashMap<PathBuf, u64> =
                std::collections::HashMap::new();

            // Initial scan
            for path in &watched_paths {
                if let Ok(metadata) = std::fs::metadata(path) {
                    previous_state.insert(path.clone(), metadata.len());
                }
            }

            while running.load(Ordering::SeqCst) {
                thread::sleep(poll_interval);

                for path in &watched_paths {
                    if let Ok(metadata) = std::fs::metadata(path) {
                        let current_size = metadata.len();
                        let previous_size = previous_state.get(path).copied().unwrap_or(0);

                        if current_size != previous_size {
                            let event = FSEvent {
                                path: path.clone(),
                                event_type: FSEventType::Modify,
                                inode: None,
                                timestamp: SystemTime::now(),
                                is_dir: metadata.is_dir(),
                            };

                            if !dedup.should_dedup(&event) {
                                let _ = sender.send(event);
                            }

                            previous_state.insert(path.clone(), current_size);
                        }

                        if metadata.is_dir() {
                            if let Ok(entries) = std::fs::read_dir(path) {
                                for entry in entries.flatten() {
                                    if let Ok(meta) = entry.metadata() {
                                        let entry_path = entry.path();
                                        if !previous_state.contains_key(&entry_path) {
                                            let event = FSEvent {
                                                path: entry_path.clone(),
                                                event_type: FSEventType::Create,
                                                inode: None,
                                                timestamp: SystemTime::now(),
                                                is_dir: meta.is_dir(),
                                            };

                                            if !dedup.should_dedup(&event) {
                                                let _ = sender.send(event);
                                            }

                                            previous_state.insert(entry_path, meta.len());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }));

        Ok(())
    }
}

#[async_trait]
impl FileSystemMonitor for FSEventsWatcher {
    async fn start(&mut self, paths: &[PathBuf]) -> Result<()> {
        if self.status == MonitorStatus::Running {
            return Ok(());
        }
        self.start_impl(paths)
    }

    async fn stop(&mut self) -> Result<()> {
        self.running.store(false, Ordering::SeqCst);
        self.status = MonitorStatus::Stopped;

        if let Some(handle) = self._worker_thread.take() {
            let _ = handle.join();
        }

        Ok(())
    }

    async fn pause(&mut self) -> Result<()> {
        self.status = MonitorStatus::Paused;
        Ok(())
    }

    async fn resume(&mut self) -> Result<()> {
        if self.running.load(Ordering::SeqCst) {
            self.status = MonitorStatus::Running;
        }
        Ok(())
    }

    fn event_stream(&self) -> flume::Receiver<FSEvent> {
        self.event_receiver.clone()
    }

    fn status(&self) -> MonitorStatus {
        self.status
    }
}

/// Create FSEvents watcher based on platform
pub fn create_watcher(config: MonitorConfig) -> Result<Box<dyn FileSystemMonitor>> {
    Ok(Box::new(FSEventsWatcher::new(config)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_watcher_creation() {
        let config = MonitorConfig::default();
        let watcher = FSEventsWatcher::new(config);
        assert!(watcher.is_ok());
    }

    #[tokio::test]
    async fn test_watcher_start_stop() {
        let config = MonitorConfig::default();
        let mut watcher = FSEventsWatcher::new(config).unwrap();

        let dir = tempdir().unwrap();
        let paths = vec![dir.path().to_path_buf()];

        watcher.start(&paths).await.unwrap();
        assert_eq!(watcher.status(), MonitorStatus::Running);

        watcher.stop().await.unwrap();
        assert_eq!(watcher.status(), MonitorStatus::Stopped);
    }

    #[tokio::test]
    async fn test_watcher_pause_resume() {
        let config = MonitorConfig::default();
        let mut watcher = FSEventsWatcher::new(config).unwrap();

        let dir = tempdir().unwrap();
        let paths = vec![dir.path().to_path_buf()];

        watcher.start(&paths).await.unwrap();
        watcher.pause().await.unwrap();
        assert_eq!(watcher.status(), MonitorStatus::Paused);

        watcher.resume().await.unwrap();
        assert_eq!(watcher.status(), MonitorStatus::Running);

        watcher.stop().await.unwrap();
    }
}
