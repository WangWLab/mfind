//! Common test utilities

use std::path::PathBuf;
use tempfile::tempdir;
use tokio::process::Command;

/// Test result
pub struct TestResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

/// Setup a test directory with sample files
pub async fn setup_test_dir() -> PathBuf {
    let dir = tempdir().unwrap();
    let path = dir.path().to_path_buf();

    // Create sample files
    tokio::fs::create_dir_all(path.join("src")).await.unwrap();
    tokio::fs::write(path.join("src/main.rs"), "fn main() {}").await.unwrap();
    tokio::fs::write(path.join("src/lib.rs"), "// lib").await.unwrap();
    tokio::fs::write(path.join("README.md"), "# Test").await.unwrap();
    tokio::fs::write(path.join("test.txt"), "test").await.unwrap();

    path
}

/// Run search command
pub async fn run_search(dir: &PathBuf, args: &str) -> TestResult {
    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--bin")
        .arg("mfind")
        .arg("--")
        .args(args.split_whitespace());

    let output = cmd.output().await.unwrap();

    TestResult {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

/// Run index build command
pub async fn run_index_build(dir: &PathBuf) -> TestResult {
    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--bin")
        .arg("mfind")
        .arg("--")
        .arg("index")
        .arg("build")
        .arg(dir.to_str().unwrap());

    let output = cmd.output().await.unwrap();

    TestResult {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

/// Run index status command
pub async fn run_index_status() -> TestResult {
    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--bin")
        .arg("mfind")
        .arg("--")
        .arg("index")
        .arg("status");

    let output = cmd.output().await.unwrap();

    TestResult {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}
