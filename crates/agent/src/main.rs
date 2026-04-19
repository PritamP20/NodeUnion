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
use std::io::{BufRead, BufReader};
use std::env;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
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

fn sanitize_public_url(raw: &str) -> String {
    let mut value = raw.trim().to_string();
    if value.starts_with("http://https://") {
        value = value.replacen("http://https://", "https://", 1);
    }
    if value.starts_with("https://http://") {
        value = value.replacen("https://http://", "http://", 1);
    }
    if !value.starts_with("http://") && !value.starts_with("https://") {
        value = format!("http://{}", value);
    }
    value.trim_end_matches('/').to_string()
}

fn local_agent_url(bind_addr: &str) -> String {
    let (_, port) = bind_addr.rsplit_once(':').unwrap_or(("127.0.0.1", "8090"));
    format!("http://127.0.0.1:{}", port)
}

fn extract_https_url(line: &str) -> Option<String> {
    line.split_whitespace().find_map(|token| {
        let cleaned = token
            .trim_matches(|c: char| c == '"' || c == '\'' || c == ',' || c == ';' || c == ')' || c == '(')
            .to_string();
        if cleaned.starts_with("https://") {
            Some(cleaned)
        } else {
            None
        }
    })
}

fn spawn_line_reader<R: std::io::Read + Send + 'static>(reader: R, tx: mpsc::Sender<String>) {
    thread::spawn(move || {
        let buffered = BufReader::new(reader);
        for line in buffered.lines() {
            if let Ok(text) = line {
                let _ = tx.send(text);
            }
        }
    });
}

fn start_cloudflare_tunnel(local_url: &str) -> Result<(Child, String), String> {
    let mut child = Command::new("cloudflared")
        .args(["tunnel", "--url", local_url, "--no-autoupdate"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to start cloudflared: {}", e))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "cloudflared stdout unavailable".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "cloudflared stderr unavailable".to_string())?;

    let (tx, rx) = mpsc::channel();
    spawn_line_reader(stdout, tx.clone());
    spawn_line_reader(stderr, tx);

    let deadline = Instant::now() + Duration::from_secs(20);
    let mut first_https: Option<String> = None;

    while Instant::now() < deadline {
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!("cloudflared exited early with status {}", status));
        }

        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(line) => {
                if let Some(url) = extract_https_url(&line) {
                    if url.contains("trycloudflare.com") {
                        return Ok((child, sanitize_public_url(&url)));
                    }
                    if first_https.is_none() {
                        first_https = Some(url);
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => break,
        }
    }

    if let Some(url) = first_https {
        return Ok((child, sanitize_public_url(&url)));
    }

    let _ = child.kill();
    Err("timed out waiting for cloudflared public URL".to_string())
}

fn resolve_agent_public_url(bind_addr: &str) -> Result<(String, Option<Child>), String> {
    if let Ok(value) = env::var("AGENT_PUBLIC_URL") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok((sanitize_public_url(trimmed), None));
        }
    }

    let provider = env::var("AGENT_PUBLIC_URL_PROVIDER")
        .unwrap_or_else(|_| "cloudflare".to_string())
        .to_ascii_lowercase();

    if provider == "cloudflare" {
        let local_url = local_agent_url(bind_addr);
        match start_cloudflare_tunnel(&local_url) {
            Ok((child, public_url)) => {
                println!("agent public URL via cloudflare tunnel: {}", public_url);
                return Ok((public_url, Some(child)));
            }
            Err(err) => {
                return Err(format!(
                    "cloudflare tunnel auto URL failed ({}). Set AGENT_PUBLIC_URL explicitly or install cloudflared.",
                    err
                ));
            }
        }
    }

    if provider == "none" {
        return Ok((agent_public_url(bind_addr), None));
    }

    Err(format!(
        "unsupported AGENT_PUBLIC_URL_PROVIDER '{}'. Use 'cloudflare' or 'none'.",
        provider
    ))
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
    let (public_agent_url, _tunnel_process) = resolve_agent_public_url(&config.bind_addr)
        .map_err(|err| format!("failed to resolve public agent URL: {}", err))?;

    // Build shared in-memory app state.
    let state = Arc::new(RwLock::new(AppState::new(
        config.node_id.clone(),
        config.idle_window_samples,
    )));

    {
        let mut guard = state.write().await;
        guard.public_url = Some(public_agent_url.clone());
    }

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
        agent_url: public_agent_url,
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