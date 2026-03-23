//! HTTP request handlers

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use mfind_core::index::engine::IndexEngineTrait;
use mfind_core::query::QueryParser;
use serde::Deserialize;
use std::time::Instant;
use tracing::{error, info};

use crate::{ApiState, ErrorResponse, SearchRequest, SearchResponse, SearchResult};

/// GET /health - Health check endpoint
pub async fn health(State(state): State<ApiState>) -> Json<crate::HealthResponse> {
    Json(crate::HealthResponse {
        status: "healthy".to_string(),
        version: state.version.to_string(),
        uptime_secs: state.uptime_secs(),
    })
}

/// GET /stats - Index statistics endpoint
pub async fn stats(State(state): State<ApiState>) -> Result<Json<crate::IndexStatsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let engine = state.engine.read().await;
    let stats = engine.stats();

    Ok(Json(crate::IndexStatsResponse {
        total_files: stats.total_files as usize,
        total_dirs: stats.total_dirs as usize,
        index_size: stats.index_size_bytes as usize,
        root_count: 0, // TODO: Add root_count to IndexStats
        last_update: stats.last_update.map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()),
    }))
}

/// Query parameters for search endpoint
#[derive(Debug, Deserialize)]
pub struct SearchParams {
    /// Search pattern (alternative to POST body)
    q: Option<String>,
    /// Maximum results
    limit: Option<usize>,
    /// Use regex
    regex: Option<bool>,
    /// Case sensitive
    case_sensitive: Option<bool>,
}

/// POST /search - Search endpoint
pub async fn search(
    State(state): State<ApiState>,
    Query(query_params): Query<SearchParams>,
    body: Option<Json<SearchRequest>>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    let start = Instant::now();

    // Get pattern from query param or body
    let pattern = if let Some(Json(body)) = body {
        body.pattern
    } else if let Some(q) = query_params.q {
        q
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Missing search pattern. Use 'q' query param or JSON body with 'pattern' field.")),
        ));
    };

    let limit = query_params.limit.unwrap_or(100);

    info!("Search request: pattern={}, limit={}", pattern, limit);

    let engine = state.engine.read().await;

    // Build query
    let query = match QueryParser::parse(&pattern) {
        Ok(q) => q,
        Err(e) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(format!("Invalid query: {}", e))),
            ));
        }
    };

    // Execute search
    let search_result = engine.search(&query);

    match search_result {
        Ok(result) => {
            let search_results: Vec<SearchResult> = result.matches
                .into_iter()
                .take(limit)
                .map(|path| {
                    let name = std::path::Path::new(&path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    SearchResult {
                        path,
                        name,
                        size: None,
                        is_dir: false,
                        modified: None,
                        score: None,
                    }
                })
                .collect();

            let total = search_results.len();
            let query_time_ms = start.elapsed().as_secs_f64() * 1000.0;

            Ok(Json(SearchResponse {
                results: search_results,
                total,
                query_time_ms,
            }))
        }
        Err(e) => {
            error!("Search error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Search failed: {}", e))),
            ))
        }
    }
}

/// GET /search/:pattern - Simple search endpoint (alternative to POST)
pub async fn search_simple(
    State(state): State<ApiState>,
    Path(pattern): Path<String>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    search(
        State(state),
        Query(SearchParams {
            q: Some(pattern),
            ..params
        }),
        None,
    )
    .await
}

/// POST /index/build - Trigger index build
pub async fn index_build(
    State(_state): State<ApiState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Implement index building via API
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse::new("Index building via API is not yet implemented")),
    ))
}

/// GET /index/rebuild - Trigger index rebuild
pub async fn index_rebuild(
    State(_state): State<ApiState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse::new("Index rebuilding via API is not yet implemented")),
    ))
}
