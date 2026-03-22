//! Filesystem scanner for indexing

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use ignore::WalkBuilder;

use crate::Result;

/// Scanner configuration
#[derive(Debug, Clone)]
pub struct ScannerConfig {
    /// Number of parallel threads
    pub parallelism: usize,
    /// Respect .gitignore files
    pub gitignore_ignore: bool,
    /// Include hidden files
    pub include_hidden: bool,
    /// Follow symlinks
    pub follow_symlinks: bool,
    /// Exclude patterns
    pub exclude_patterns: Vec<String>,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            parallelism: num_cpus::get(),
            gitignore_ignore: true,
            include_hidden: false,
            follow_symlinks: false,
            exclude_patterns: vec![],
        }
    }
}

/// Scan entry representing a file or directory
#[derive(Debug, Clone)]
pub struct ScanEntry {
    pub path: PathBuf,
    pub inode: Option<u64>,
    pub size: u64,
    pub modified: SystemTime,
    pub is_dir: bool,
}

/// Filesystem scanner
pub struct FileSystemScanner {
    config: ScannerConfig,
}

impl FileSystemScanner {
    /// Create a new scanner
    pub fn new(config: ScannerConfig) -> Self {
        Self { config }
    }

    /// Scan paths and return entries
    pub async fn scan(&self, roots: &[PathBuf]) -> Result<Vec<ScanEntry>> {
        let mut all_entries = Vec::new();

        for root in roots {
            let entries = self.scan_single(root).await?;
            all_entries.extend(entries);
        }

        Ok(all_entries)
    }

    /// Scan a single root path
    async fn scan_single(&self, root: &Path) -> Result<Vec<ScanEntry>> {
        let config = self.config.clone();
        let root = root.to_path_buf();

        // Run blocking I/O in thread pool
        let entries = tokio::task::spawn_blocking(move || {
            let mut builder = WalkBuilder::new(&root);

            builder
                .hidden(!config.include_hidden)
                .git_ignore(config.gitignore_ignore)
                .follow_links(config.follow_symlinks)
                .threads(config.parallelism);

            // Add exclude patterns
            for pattern in &config.exclude_patterns {
                builder.add_ignore(pattern.clone());
            }

            // Collect entries from the walker
            let mut final_entries = Vec::new();
            for entry_result in builder.build() {
                let entry = match entry_result {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                let path = entry.path().to_path_buf();
                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let is_dir = metadata.is_dir();
                let size = if is_dir { 0 } else { metadata.len() };
                let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);

                #[cfg(unix)]
                let inode = {
                    use std::os::unix::fs::MetadataExt;
                    Some(metadata.ino())
                };
                #[cfg(not(unix))]
                let inode = None;

                final_entries.push(ScanEntry {
                    path,
                    inode,
                    size,
                    modified,
                    is_dir,
                });
            }

            final_entries
        })
        .await?;

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_scan_basic() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("file1.txt"), "content").unwrap();
        std::fs::write(dir.path().join("file2.rs"), "code").unwrap();

        let scanner = FileSystemScanner::new(ScannerConfig::default());
        let entries = scanner.scan(&[dir.path().to_path_buf()]).await.unwrap();

        assert!(entries.len() >= 2);
    }
}
