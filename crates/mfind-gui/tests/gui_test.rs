//! Tests for mfind-gui commands

use mfind_core::index::engine::IndexEngineTrait;
use mfind_core::{IndexConfig, IndexEngine, QueryParser};
use mfind_gui::commands::{BuildIndexResponse, SearchResponse, SearchResultItem};
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
