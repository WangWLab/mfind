//! Route definitions

use axum::{
    routing::{get, post},
    Router,
};

use crate::{handlers, ApiState};

/// Create the API router
pub fn create_router(state: ApiState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(handlers::health))
        // Index statistics
        .route("/stats", get(handlers::stats))
        // Search endpoints
        .route("/search", post(handlers::search))
        .route("/search/:pattern", get(handlers::search_simple))
        // Index management (not yet implemented)
        .route("/index/build", post(handlers::index_build))
        .route("/index/rebuild", get(handlers::index_rebuild))
        .with_state(state)
}
