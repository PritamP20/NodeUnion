mod api;
mod app_state;
mod config;
mod container_manager;
mod container_monitor;
mod errors;
mod heartbeat;
mod idle_detector;
mod models;
mod orchestrator_client;

use std::sync::Arc;
use axum::Router;
use bollard::Docker;
use std::net::UdpSocket;
use std::env;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use crate::api::{build_router, AppApiState};
use crate::app_state::AppState;
use crate::config::Config;
use crate::container_monitor::run_container_monitor;
use crate::heartbeat::run_heartbeat_loop;
use crate::idle_detector::run_idle_detector;
use crate::models::RegisterNodeRequest;
use crate::orchestrator_client::OrchestratorClient;

fn detect_local_ip() -> Option<std::net::IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let addr = socket.local_addr().ok()?;
    Some(addr.ip())
}

fn agent_public_url(bind_addr: &str) -> String {
    if let Ok(value) = env::var("AGENT_PUBLIC_URL") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    let (host, port) = bind_addr
        .rsplit_once(':')
        .unwrap_or(("127.0.0.1", "8090"));

    let public_host = if host == "0.0.0.0" {
        detect_local_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| "127.0.0.1".to_string())
    } else {
        host.to_string()
    };

    format!("http://{}:{}", public_host, port)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env();

    // Build shared in-memory app state.
    let state = Arc::new(RwLock::new(AppState::new(
        config.node_id.clone(),
        config.idle_window_samples,
    )));

    // Connect to local Docker daemon.
    let docker = Docker::connect_with_local_defaults().map_err(|err| {
        format!(
            "failed to connect to Docker daemon: {}. Make sure Docker Desktop/service is running before starting the agent",
            err
        )
    })?;

    // Create outbound orchestrator client.
    let orchestrator_client = OrchestratorClient::new(&config);
    let register_client = orchestrator_client.clone();

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

    // Spawn container monitor loop in background.
    {
        let state_clone = state.clone();
        let docker_clone = docker.clone();
        let client_clone = orchestrator_client.clone();
        let node_id_clone = config.node_id.clone();

        tokio::spawn(async move {
            run_container_monitor(state_clone, docker_clone, client_clone, node_id_clone).await;
        });
    }

    // Build API shared state.
    let api_state = AppApiState {
        state,
        config: config.clone(),
        orchestrator_client,
        docker,
    };

    let register_payload = RegisterNodeRequest {
        node_id: config.node_id.clone(),
        network_id: config.network_id.clone(),
        agent_url: agent_public_url(&config.bind_addr),
        provider_wallet: env::var("PROVIDER_WALLET").ok().map(|value| value.trim().to_string()).filter(|value| !value.is_empty()),
        region: None,
        labels: None,
    };

    register_client
        .register_node(&register_payload)
        .await
        .map_err(|err| format!("failed to register node with orchestrator: {}", err))?;

    // Build router with /health, /run, and /stop endpoints.
    let app: Router = build_router(api_state);

    // Bind HTTP listener on configured address.
    let listener = TcpListener::bind(&config.bind_addr).await?;

    // Start serving requests.
    axum::serve(listener, app).await?;

    Ok(())
}