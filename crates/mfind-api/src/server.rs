//! API server implementation

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use mfind_core::index::IndexEngine;

use crate::{ApiConfig, ApiState, routes};

/// API server
pub struct ApiServer {
    config: ApiConfig,
    state: ApiState,
}

impl ApiServer {
    /// Create a new API server
    pub fn new(config: ApiConfig, engine: Arc<RwLock<IndexEngine>>) -> Self {
        let state = ApiState::new(engine);
        Self { config, state }
    }

    /// Run the API server (blocking)
    pub async fn run(&self) -> anyhow::Result<()> {
        // Initialize logging
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .with_target(false)
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .map_err(|e| anyhow::anyhow!("Logging initialization failed: {}", e))?;

        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()
            .expect("Invalid address");

        let app = routes::create_router(self.state.clone());

        info!("Starting API server on http://{}", addr);

        let listener = TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }

    /// Run the API server with a specific listener
    pub async fn run_with_listener(&self, listener: TcpListener) -> anyhow::Result<()> {
        let addr = listener.local_addr()?;
        let app = routes::create_router(self.state.clone());

        info!("Starting API server on http://{}", addr);

        axum::serve(listener, app).await?;
        Ok(())
    }

    /// Get the server configuration
    pub fn config(&self) -> &ApiConfig {
        &self.config
    }

    /// Get the API state
    pub fn state(&self) -> &ApiState {
        &self.state
    }
}

/// Start the API server in the background
pub async fn spawn_api_server(
    config: ApiConfig,
    engine: Arc<RwLock<IndexEngine>>,
) -> anyhow::Result<tokio::task::JoinHandle<anyhow::Result<()>>> {
    let server = ApiServer::new(config, engine);
    Ok(tokio::spawn(async move { server.run().await }))
}
