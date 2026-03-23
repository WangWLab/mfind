//! mfind API - HTTP/REST API server for mfind
//!
//! Provides a RESTful interface for file search and index management.

mod handlers;
pub mod routes;
mod server;
mod state;

pub use server::ApiServer;
pub use state::ApiState;

/// API configuration
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// Host to bind to
    pub host: String,
    /// Port to listen on
    pub port: u16,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
        }
    }
}

/// Search request
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    /// Search pattern
    pub pattern: String,
    /// Root paths to search (optional, defaults to indexed roots)
    pub roots: Option<Vec<String>>,
    /// Maximum results (optional, defaults to 100)
    pub limit: Option<usize>,
    /// Use regex pattern
    #[serde(default)]
    pub regex: bool,
    /// Case sensitive search
    #[serde(default)]
    pub case_sensitive: bool,
}

/// Search response
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    /// Search results
    pub results: Vec<SearchResult>,
    /// Total count (may be estimated)
    pub total: usize,
    /// Query time in milliseconds
    pub query_time_ms: f64,
}

/// Single search result
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    /// File path
    pub path: String,
    /// File name
    pub name: String,
    /// File size in bytes
    pub size: Option<u64>,
    /// Is directory
    pub is_dir: bool,
    /// Last modified timestamp
    pub modified: Option<u64>,
    /// Match score (for fuzzy search)
    pub score: Option<f64>,
}

/// Index statistics response
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexStatsResponse {
    /// Total files indexed
    pub total_files: usize,
    /// Total directories indexed
    pub total_dirs: usize,
    /// Index size in bytes
    pub index_size: usize,
    /// Number of root paths
    pub root_count: usize,
    /// Last update timestamp
    pub last_update: Option<u64>,
}

/// Health check response
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    /// Service status
    pub status: String,
    /// Service version
    pub version: String,
    /// Uptime in seconds
    pub uptime_secs: u64,
}

/// Common error response
#[derive(Debug, serde::Serialize)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
    /// Error code
    pub code: Option<String>,
}

impl ErrorResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            error: message.into(),
            code: None,
        }
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}
