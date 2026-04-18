// AgentCluster Backend in Rust with Axum
mod config;
mod datasource;
mod domain;
mod infra;
mod state;
mod utils;
mod api;
mod proto;
mod error;

use std::error::Error;
use tracing::{info, error, warn, debug};
use tracing_subscriber::{fmt, EnvFilter};

use crate::config::config::{AppConfig, parse_config_path_from_args};
use crate::api::v1::handlers::cluster_handlers::{
    get_cluster_metrics, get_cluster_status,
};
use crate::api::v1::handlers::node_handlers::{
    get_node, list_nodes, register_node, update_node_status,
};
use crate::api::v1::handlers::task_handlers::{
    cancel_task, get_task, list_tasks, submit_task,
};
use crate::infra::registry::Registry;
use crate::state::state::AppState;
use crate::error::AppResult;

fn health_check_route() -> axum::Router<AppState> {
    axum::Router::new().route("/health", axum::routing::get(|| async { "OK" }))
}

fn api_v1_route() -> axum::Router<AppState> {
    axum::Router::new()
        .nest("/nodes", api::v1::handlers::node_handlers::node_handlers::routes())
        .nest("/tasks", api::v1::handlers::task_handlers::task_handlers::routes())
        .nest("/cluster", api::v1::handlers::cluster_handlers::cluster_handlers::routes())
}

fn app_route() -> axum::Router<AppState> {
    axum::Router::new().nest("/api/v1", api_v1_route())
}

#[tokio::main]
async fn main() -> AppResult<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Load configuration
    let config = match parse_config_path_from_args(std::env::args()) {
        Some(config_path) => {
            info!("Loading configuration from {}", config_path);
            AppConfig::load_from_path(&config_path)?
        },
        None => {
            info!("Loading default configuration");
            AppConfig::load()?
        },
    };

    info!("Configuration loaded successfully");

    // Initialize infrastructure registry
    info!("Initializing infrastructure registry");
    let registry = Registry::new(&config).await?;
    info!("Infrastructure registry initialized");

    // Create app state
    info!("Creating application state");
    let app_state = AppState {
        cluster_controller: crate::domain::cluster_controller::ClusterController::new(
            registry.db_dao.clone(),
            registry.scylla_dao.clone(),
            registry.vector_dao.clone(),
            registry.etcd_client.clone(),
            registry.state_store.clone(),
        )?,
        state_store: registry.state_store.clone(),
    };
    info!("Application state created");

    // Create Axum router with basic routes
    info!("Setting up HTTP routes");
    let app = health_check_route()
        .merge(app_route())
        .with_state(app_state);

    // Run the server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!("Starting AgentCluster server on {}", addr);
    
    match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => {
            info!("TCP listener bound successfully");
            if let Err(e) = axum::serve(listener, app).await {
                error!("Server error: {}", e);
                return Err(e);
            }
        },
        Err(e) => {
            error!("Failed to bind to address {}: {}", addr, e);
            return Err(e);
        }
    }

    info!("AgentCluster server shutdown gracefully");
    Ok(())
}