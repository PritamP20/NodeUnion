mod api;
mod app_state;
mod config;
mod container_manager;
mod errors;
mod heartbeat;
mod idle_detector;
mod models;
mod orchestrator_client;

use std::sync::Arc;
use axum::Router;
use bollard::Docker;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use crate::api::{build_router, AppApiState};
use crate::app_state::AppState;
use crate::config::Config;
use crate::heartbeat::run_heartbeat_loop;
use crate::idle_detector::run_idle_detector;
use crate::orchestrator_client::OrchestratorClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env();

    // Build shared in-memory app state.
    let state = Arc::new(RwLock::new(AppState::new(
        config.node_id.clone(),
        config.idle_window_samples,
    )));

    // Connect to local Docker daemon.
    let docker = Docker::connect_with_local_defaults()?;

    // Create outbound orchestrator client.
    let orchestrator_client = OrchestratorClient::new(&config);

    // Spawn idle detector loop in background.
    {
        let state_clone = state.clone();
        let config_clone = config.clone();

        tokio::spawn(async move {
            run_idle_detector(state_clone, config_clone).await;
        });
    }

    // Spawn heartbeat loop in background.
    {
        let state_clone = state.clone();
        let config_clone = config.clone();
        let client_clone = orchestrator_client.clone();

        tokio::spawn(async move {
            run_heartbeat_loop(state_clone, client_clone, config_clone).await;
        });
    }

    // Build API shared state.
    let api_state = AppApiState {
        state,
        config: config.clone(),
        orchestrator_client,
        docker,
    };

    // Build router with /health, /run, and /stop endpoints.
    let app: Router = build_router(api_state);

    // Bind HTTP listener on configured address.
    let listener = TcpListener::bind(&config.bind_addr).await?;

    // Start serving requests.
    axum::serve(listener, app).await?;

    Ok(())
}