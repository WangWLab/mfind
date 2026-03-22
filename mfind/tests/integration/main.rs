//! Integration tests for mfind

mod common;

#[cfg(test)]
mod search_tests {
    use super::common::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_search_prefix() {
        let dir = setup_test_dir().await;
        let result = run_search(&dir, "test").await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_search_wildcard() {
        let dir = setup_test_dir().await;
        let result = run_search(&dir, "*.txt").await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_search_regex() {
        let dir = setup_test_dir().await;
        let result = run_search(&dir, "--regex").await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_search_extension() {
        let dir = setup_test_dir().await;
        let result = run_search(&dir, "-e rs").await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_search_limit() {
        let dir = setup_test_dir().await;
        let result = run_search(&dir, "-n 10").await;
        assert!(result.success);
    }
}

#[cfg(test)]
mod index_tests {
    use super::common::*;

    #[tokio::test]
    async fn test_index_build() {
        let dir = setup_test_dir().await;
        let result = run_index_build(&dir).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_index_status() {
        let result = run_index_status().await;
        assert!(result.success);
    }
}

#[cfg(test)]
mod fs_tests {
    use mfind_core::fs::{FileSystemScanner, ScannerConfig};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_scanner_basic() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("file1.txt"), "content").unwrap();
        std::fs::write(dir.path().join("file2.rs"), "code").unwrap();

        let scanner = FileSystemScanner::new(ScannerConfig::default());
        let entries = scanner.scan(&[dir.path().to_path_buf()]).await.unwrap();

        assert!(entries.len() >= 2);
    }

    #[tokio::test]
    async fn test_scanner_gitignore() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("file.txt"), "content").unwrap();
        std::fs::write(dir.path().join(".gitignore"), "*.txt").unwrap();

        let mut config = ScannerConfig::default();
        config.gitignore_ignore = true;

        let scanner = FileSystemScanner::new(config);
        let entries = scanner.scan(&[dir.path().to_path_buf()]).await.unwrap();

        // .txt file should be ignored
        assert!(entries.iter().all(|e| !e.path.ends_with("file.txt")));
    }
}
