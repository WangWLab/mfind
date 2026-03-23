//! Integration tests for mfind-api

use std::sync::Arc;
use tokio::sync::RwLock;

use mfind_api::{ApiServer, ApiConfig, ApiState};
use mfind_core::{IndexEngine, IndexConfig};

/// Helper to create a test index engine
async fn create_test_engine() -> Arc<RwLock<IndexEngine>> {
    let config = IndexConfig::default();
    let engine = IndexEngine::new(config).expect("Failed to create index engine");
    Arc::new(RwLock::new(engine))
}

#[tokio::test]
async fn test_api_server_creation() {
    let engine = create_test_engine().await;
    let config = ApiConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
    };

    let server = ApiServer::new(config, engine);
    assert_eq!(server.config().host, "127.0.0.1");
    assert_eq!(server.config().port, 0);
}

#[tokio::test]
async fn test_api_state_creation() {
    let engine = create_test_engine().await;
    let state = ApiState::new(engine);

    assert_eq!(state.uptime_secs(), 0);
    assert!(!state.version.is_empty());
}

#[tokio::test]
async fn test_api_config_default() {
    let config = ApiConfig::default();
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 3000);
}
