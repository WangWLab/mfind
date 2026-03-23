//! Tests for mfind-gui commands

use mfind_core::index::engine::IndexEngineTrait;
use mfind_core::{IndexConfig, IndexEngine, QueryParser};
use mfind_gui::commands::{BuildIndexResponse, SearchResponse, SearchResultItem, FilePreviewResponse};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_get_stats() {
    let config = IndexConfig::default();
    let engine = IndexEngine::new(config).expect("Failed to create index engine");
    let engine = Arc::new(RwLock::new(engine));

    let read_engine = engine.read().await;
    let stats = read_engine.stats();

    // Initial stats should be empty/zero
    assert_eq!(stats.total_files, 0);
    assert_eq!(stats.total_dirs, 0);
}

#[tokio::test]
async fn test_search_empty_index() {
    let config = IndexConfig::default();
    let engine = IndexEngine::new(config).expect("Failed to create index engine");
    let engine = Arc::new(RwLock::new(engine));

    let read_engine = engine.read().await;
    let query = QueryParser::parse("test").unwrap();
    let result = read_engine.search(&query);

    // Should return empty results for empty index
    assert!(result.is_ok());
    let matches = result.unwrap();
    assert!(matches.matches.is_empty());
}

#[tokio::test]
async fn test_build_index_response_serialization() {
    let response = BuildIndexResponse {
        total_files: 100,
        total_dirs: 10,
        build_time_ms: 50.5,
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("100"));
    assert!(json.contains("10"));
}

#[tokio::test]
async fn test_search_response_serialization() {
    let response = SearchResponse {
        results: vec![
            SearchResultItem {
                path: "/test/file.txt".to_string(),
                name: "file.txt".to_string(),
                is_dir: false,
                size: Some(1024),
            }
        ],
        total: 1,
        query_time_ms: 10.5,
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("file.txt"));
    assert!(json.contains("1024"));
}

#[tokio::test]
async fn test_file_preview_response_serialization() {
    let response = FilePreviewResponse {
        path: "/test/file.txt".to_string(),
        r#type: "text".to_string(),
        content: Some("Hello, World!".to_string()),
        data_uri: None,
        size: 1024,
        mime: "text/plain".to_string(),
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("text"));
    assert!(json.contains("Hello, World!"));
    assert!(json.contains("1024"));
}

#[tokio::test]
async fn test_file_preview_response_image() {
    let response = FilePreviewResponse {
        path: "/test/image.png".to_string(),
        r#type: "image".to_string(),
        content: None,
        data_uri: Some("data:image/png;base64,iVBORw0KGgo=".to_string()),
        size: 2048,
        mime: "image/png".to_string(),
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("image"));
    assert!(json.contains("base64"));
    assert!(json.contains("image/png"));
}

#[test]
fn test_is_text_file() {
    use mfind_gui::commands::is_text_file;

    // Text files
    assert!(is_text_file("rs"));
    assert!(is_text_file("txt"));
    assert!(is_text_file("md"));
    assert!(is_text_file("json"));
    assert!(is_text_file("py"));

    // Non-text files
    assert!(!is_text_file("png"));
    assert!(!is_text_file("jpg"));
    assert!(!is_text_file("exe"));
}

#[test]
fn test_is_image_file() {
    use mfind_gui::commands::is_image_file;

    // Image files
    assert!(is_image_file("png"));
    assert!(is_image_file("jpg"));
    assert!(is_image_file("jpeg"));
    assert!(is_image_file("gif"));
    assert!(is_image_file("webp"));

    // Non-image files
    assert!(!is_image_file("txt"));
    assert!(!is_image_file("rs"));
    assert!(!is_image_file("pdf"));
}

#[test]
fn test_get_mime_type() {
    use mfind_gui::commands::get_mime_type;

    assert_eq!(get_mime_type("txt"), "text/plain");
    assert_eq!(get_mime_type("png"), "image/png");
    assert_eq!(get_mime_type("jpg"), "image/jpeg");
    assert_eq!(get_mime_type("json"), "application/json");
    assert_eq!(get_mime_type("unknown"), "application/octet-stream");
}

#[test]
fn test_base64_encode() {
    use mfind_gui::commands::base64_encode;

    // Test basic encoding
    assert_eq!(base64_encode(b"Hello"), "SGVsbG8=");
    assert_eq!(base64_encode(b"Hi"), "SGk=");
    assert_eq!(base64_encode(b""), "");

    // Test with binary data
    let data = vec![0x00, 0x01, 0x02, 0x03];
    let encoded = base64_encode(&data);
    assert_eq!(encoded.len(), 8); // 4 bytes -> 8 base64 chars (with padding)
}
