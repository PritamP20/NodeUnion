use nodeunion_agent::models::AgentStateResponse;
use reqwest::Client;
use std::env;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};

struct Snapshot {
    healthy: bool,
    state: Option<AgentStateResponse>,
    errors: Vec<String>,
}

async fn fetch_snapshot(client: &Client, base_url: &str) -> Snapshot {
    let mut errors = Vec::new();

    let health_url = format!("{}/health", base_url);
    let healthy = match client.get(&health_url).send().await {
        Ok(resp) => resp.status().is_success(),
        Err(err) => {
            errors.push(format!("health check failed: {}", err));
            false
        }
    };

    let state = match fetch_json::<AgentStateResponse>(client, base_url, "/state").await {
        Ok(v) => Some(v),
        Err(e) => {
            errors.push(e);
            None
        }
    };

    Snapshot {
        healthy,
        state,
        errors,
    }
}

async fn fetch_json<T>(client: &Client, base_url: &str, path: &str) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    let url = format!("{}{}", base_url, path);
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("GET {} failed: {}", path, e))?;

    if !resp.status().is_success() {
        return Err(format!("GET {} returned status {}", path, resp.status()));
    }

    resp.json::<T>()
        .await
        .map_err(|e| format!("parse {} JSON failed: {}", path, e))
}

fn render(snapshot: &Snapshot, base_url: &str) {
    print!("\x1B[2J\x1B[H");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    println!("NodeUnion Agent Local TUI");
    println!("agent: {}", base_url);
    println!("now epoch: {}", now);
    println!("ctrl+c to exit");
    println!();

    println!("status");
    println!(
        "  health: {}",
        if snapshot.healthy { "UP" } else { "DOWN" }
    );

    if let Some(state) = &snapshot.state {
        println!("  node_id: {}", state.node_id);
        println!("  node_status: {:?}", state.status);
        println!("  is_idle: {}", state.is_idle);
        println!("  running_chunks: {}", state.running_chunks);
        println!(
            "  cpu: used={:.1}% available={:.1}% avg_window={}",
            state.cpu_usage_pct,
            state.cpu_available_pct,
            state
                .avg_cpu_window_pct
                .map(|v| format!("{:.1}%", v))
                .unwrap_or_else(|| "n/a".to_string())
        );
        println!(
            "  ram: available={}MB / total={}MB",
            state.ram_available_mb, state.ram_total_mb
        );
        println!("  disk_available_gb: {}", state.disk_available_gb);
        println!("  preempt_spikes: {}", state.consecutive_preempt_spikes);
        println!();

        println!("active chunks (top 10)");
        if state.active_chunks.is_empty() {
            println!("  (none)");
        } else {
            for chunk in state.active_chunks.iter().take(10) {
                println!(
                    "  {} | job={} | status={:?} | container={}",
                    chunk.chunk_id, chunk.job_id, chunk.status, chunk.container_id
                );
            }
        }
    } else {
        println!("  state: unavailable");
    }

    if !snapshot.errors.is_empty() {
        println!();
        println!("errors");
        for err in &snapshot.errors {
            println!("  - {}", err);
        }
    }

    let _ = io::stdout().flush();
}

#[tokio::main]
async fn main() {
    let base_url = env::var("AGENT_URL").unwrap_or_else(|_| "http://127.0.0.1:8090".to_string());
    let base_url = base_url.trim_end_matches('/').to_string();

    let client = Client::new();

    loop {
        let snapshot = fetch_snapshot(&client, &base_url).await;
        render(&snapshot, &base_url);

        tokio::select! {
            _ = sleep(Duration::from_secs(2)) => {}
            _ = tokio::signal::ctrl_c() => {
                println!("\nshutting down local tui...");
                break;
            }
        }
    }
}
