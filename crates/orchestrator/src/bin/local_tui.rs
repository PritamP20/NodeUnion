use nodeunion_orchestrator::model::{JobRecord, NetworkRecord, NodeRecord};
use reqwest::Client;
use std::env;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};

struct Snapshot {
    healthy: bool,
    networks: Vec<NetworkRecord>,
    nodes: Vec<NodeRecord>,
    jobs: Vec<JobRecord>,
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

    let networks = match fetch_json::<Vec<NetworkRecord>>(client, base_url, "/networks").await {
        Ok(v) => v,
        Err(e) => {
            errors.push(e);
            Vec::new()
        }
    };

    let nodes = match fetch_json::<Vec<NodeRecord>>(client, base_url, "/nodes").await {
        Ok(v) => v,
        Err(e) => {
            errors.push(e);
            Vec::new()
        }
    };

    let jobs = match fetch_json::<Vec<JobRecord>>(client, base_url, "/jobs").await {
        Ok(v) => v,
        Err(e) => {
            errors.push(e);
            Vec::new()
        }
    };

    Snapshot {
        healthy,
        networks,
        nodes,
        jobs,
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

    println!("NodeUnion Local TUI");
    println!("orchestrator: {}", base_url);
    println!("now epoch: {}", now);
    println!("ctrl+c to exit");
    println!();

    println!("status");
    println!(
        "  health: {}",
        if snapshot.healthy { "UP" } else { "DOWN" }
    );
    println!("  networks: {}", snapshot.networks.len());
    println!("  nodes: {}", snapshot.nodes.len());
    println!("  jobs: {}", snapshot.jobs.len());
    println!();

    println!("nodes (top 5)");
    for node in snapshot.nodes.iter().take(5) {
        println!(
            "  {} | net={} | status={:?} | idle={} | cpu={:.1}% | ram={}MB | chunks={}",
            node.node_id,
            node.network_id,
            node.status,
            node.is_idle,
            node.cpu_available_pct,
            node.ram_available_mb,
            node.running_chunks
        );
    }
    if snapshot.nodes.is_empty() {
        println!("  (none)");
    }
    println!();

    println!("jobs (top 10)");
    for job in snapshot.jobs.iter().take(10) {
        println!(
            "  {} | net={} | status={:?} | node={} | image={}",
            job.job_id,
            job.network_id,
            job.status,
            job.assigned_node_id.as_deref().unwrap_or("-") ,
            job.image
        );
    }
    if snapshot.jobs.is_empty() {
        println!("  (none)");
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
    let base_url = env::var("ORCHESTRATOR_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
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
