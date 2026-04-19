mod api;
mod db;
mod model;
mod state;
mod solana_client;

use api::{
    build_router, hydrate_runtime_state_from_db, run_pending_scheduler_loop,
    run_status_maintenance_loop, AppState,
};
use db::schema::NetworkRow;
use dotenvy::dotenv;
use reqwest::Client;
use solana_client::SolanaClient;
use state::{OrchestratorState, SharedState};
use std::env;
use std::io::{BufRead, BufReader};
use std::net::UdpSocket;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;
use tokio::sync::RwLock;

fn now_epoch_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn detect_local_ip() -> Option<std::net::IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let addr = socket.local_addr().ok()?;
    Some(addr.ip())
}

fn advertised_bind_url(bind_addr: &str) -> Option<String> {
    let (host, port) = bind_addr.rsplit_once(':')?;
    let public_host = if host == "0.0.0.0" {
        detect_local_ip()?.to_string()
    } else {
        host.to_string()
    };
    Some(format!("http://{}:{}", public_host, port))
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

fn local_orchestrator_url(bind_addr: &str) -> String {
    let (_, port) = bind_addr.rsplit_once(':').unwrap_or(("127.0.0.1", "8080"));
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

fn resolve_orchestrator_public_url(bind_addr: &str) -> Result<(String, Option<Child>), String> {
    if let Ok(value) = env::var("ORCHESTRATOR_PUBLIC_URL") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok((sanitize_public_url(trimmed), None));
        }
    }

    let provider = env::var("ORCHESTRATOR_PUBLIC_URL_PROVIDER")
        .unwrap_or_else(|_| "cloudflare".to_string())
        .to_ascii_lowercase();

    if provider == "cloudflare" {
        let local_url = local_orchestrator_url(bind_addr);
        match start_cloudflare_tunnel(&local_url) {
            Ok((child, public_url)) => {
                println!("orchestrator public URL via cloudflare tunnel: {}", public_url);
                return Ok((public_url, Some(child)));
            }
            Err(err) => {
                return Err(format!(
                    "cloudflare tunnel auto URL failed ({}). Set ORCHESTRATOR_PUBLIC_URL or install cloudflared.",
                    err
                ));
            }
        }
    }

    if provider == "none" {
        return Ok((
            advertised_bind_url(bind_addr).unwrap_or_else(|| local_orchestrator_url(bind_addr)),
            None,
        ));
    }

    Err(format!(
        "unsupported ORCHESTRATOR_PUBLIC_URL_PROVIDER '{}'. Use 'cloudflare' or 'none'.",
        provider
    ))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let bind_addr = env::var("ORCHESTRATOR_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let (orchestrator_public_url, _public_url_tunnel) = resolve_orchestrator_public_url(&bind_addr)
        .map_err(|err| anyhow::anyhow!("failed to resolve orchestrator public URL: {}", err))?;

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL is required (set Neon connection string)");
    let db_pool = db::new_pool(&database_url).await?;
    db::init_schema(&db_pool).await?;

    let state: SharedState = Arc::new(RwLock::new(OrchestratorState::default()));
    let http = Client::new();
    let solana = SolanaClient::from_env()?;
    let managed_network_id = env::var("ORCHESTRATOR_NETWORK_ID")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    let managed_network_name = env::var("ORCHESTRATOR_NETWORK_NAME")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    let managed_network_description = env::var("ORCHESTRATOR_NETWORK_DESCRIPTION")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());

    if let Some(network_id) = managed_network_id.clone() {
        let network_row = NetworkRow {
            network_id: network_id.clone(),
            name: managed_network_name.unwrap_or_else(|| network_id.clone()),
            description: managed_network_description,
            orchestrator_url: Some(orchestrator_public_url.clone()),
            status: "Active".to_string(),
            created_at_epoch_secs: now_epoch_secs(),
        };

        db::networks_repo::create_network(&db_pool, &network_row).await?;
    }

    let app_state = AppState {
        state,
        http,
        db: db_pool,
        solana,
        managed_network_id,
        orchestrator_public_url: Some(orchestrator_public_url.clone()),
    };

    hydrate_runtime_state_from_db(&app_state).await;

    // Background scheduler retries pending jobs periodically.
    {
        let scheduler_state = app_state.clone();
        tokio::spawn(async move {
            run_pending_scheduler_loop(scheduler_state, 5).await;
        });
    }

    {
        let maintenance_state = app_state.clone();
        tokio::spawn(async move {
            run_status_maintenance_loop(maintenance_state, 5, 180).await;
        });
    }

    let app = build_router(app_state);

    let listener = TcpListener::bind(&bind_addr).await?;
    println!("orchestrator listening on {}", bind_addr);
    println!("public orchestrator URL: {}", orchestrator_public_url);
    axum::serve(listener, app).await?;
    Ok(())
}
