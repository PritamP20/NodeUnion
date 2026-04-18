use std::env;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::net::TcpListener;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct NetworkOption {
    network_id: String,
    name: String,
    status: String,
}

fn prompt(label: &str, env_key: &str, default: &str) -> String {
    let current = env::var(env_key).unwrap_or_else(|_| default.to_string());
    print!("{} [{}]: ", label, current);
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return current;
    }

    let trimmed = input.trim();
    if trimmed.is_empty() {
        current
    } else {
        trimmed.to_string()
    }
}

fn prompt_required(label: &str, env_key: &str) -> String {
    loop {
        let current = env::var(env_key).unwrap_or_default();
        if current.is_empty() {
            print!("{}: ", label);
        } else {
            print!("{} [{}]: ", label, current);
        }
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            if !current.is_empty() {
                return current;
            }
            continue;
        }

        let trimmed = input.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
        if !current.is_empty() {
            return current;
        }

        println!("This field is required.");
    }
}

fn is_bind_available(bind_addr: &str) -> bool {
    TcpListener::bind(bind_addr).is_ok()
}

fn choose_bind_addr(default_addr: &str, env_key: &str) -> String {
    let current = env::var(env_key).unwrap_or_else(|_| default_addr.to_string());

    loop {
        let candidate = prompt("AGENT_BIND_ADDR", env_key, &current);
        if is_bind_available(&candidate) {
            return candidate;
        }

        println!("{} is already in use.", candidate);
        println!("Press Enter to try the next free port or type a new bind address.");

        for port in 8090u16..8110u16 {
            let auto_candidate = format!("0.0.0.0:{}", port);
            if is_bind_available(&auto_candidate) {
                println!("Suggested free bind address: {}", auto_candidate);
                break;
            }
        }
    }
}

fn prompt_network_choice(networks: &[NetworkOption], env_key: &str) -> String {
    let current = env::var(env_key).unwrap_or_default();

    loop {
        println!();
        println!("Available networks from orchestrator:");
        for (index, network) in networks.iter().enumerate() {
            println!(
                "  [{}] {} - {} ({})",
                index + 1,
                network.network_id,
                network.name,
                network.status,
            );
        }

        if current.is_empty() {
            print!("Select network by number or network id: ");
        } else {
            print!("Select network by number or network id [{}]: ", current);
        }
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            if !current.is_empty() {
                return current;
            }
            continue;
        }

        let trimmed = input.trim();
        let selection = if trimmed.is_empty() { current.clone() } else { trimmed.to_string() };

        if selection.is_empty() {
            println!("This field is required.");
            continue;
        }

        if let Ok(index) = selection.parse::<usize>() {
            if index >= 1 && index <= networks.len() {
                return networks[index - 1].network_id.clone();
            }
        }

        if let Some(network) = networks.iter().find(|network| network.network_id == selection) {
            return network.network_id.clone();
        }

        println!("Unknown network selection. Choose one of the listed options.");
    }
}

async fn fetch_networks(orchestrator_base_url: &str) -> anyhow::Result<Vec<NetworkOption>> {
    let client = reqwest::Client::new();
    let url = format!("{}/networks", orchestrator_base_url.trim_end_matches('/'));
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("network fetch returned status {}", response.status()));
    }

    let networks = response.json::<Vec<NetworkOption>>().await?;
    Ok(networks)
}

fn run_agent(vars: &[(&str, String)]) -> anyhow::Result<()> {
    let mut cmd = Command::new("nodeunion-agent");
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    for (k, v) in vars {
        cmd.env(k, v);
    }

    match cmd.status() {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                Err(anyhow::anyhow!("nodeunion-agent exited with status {}", status))
            }
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            let mut fallback = Command::new("cargo");
            fallback
                .arg("run")
                .arg("-p")
                .arg("nodeunion-agent")
                .arg("--bin")
                .arg("nodeunion-agent")
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());

            for (k, v) in vars {
                fallback.env(k, v);
            }

            let status = fallback.status()?;
            if status.success() {
                Ok(())
            } else {
                Err(anyhow::anyhow!("fallback cargo run exited with status {}", status))
            }
        }
        Err(err) => Err(err.into()),
    }
}

fn main() -> anyhow::Result<()> {
    println!("NodeUnion Agent Launch Form");
    println!("Fill values and press enter to keep defaults where available.");
    println!();

    let node_id = prompt_required("NODE_ID", "NODE_ID");
    let orchestrator_base_url = prompt_required("ORCHESTRATOR_BASE_URL", "ORCHESTRATOR_BASE_URL");
    let networks = tokio::runtime::Runtime::new()?.block_on(fetch_networks(&orchestrator_base_url));
    let network_id = match networks {
        Ok(networks) if !networks.is_empty() => prompt_network_choice(&networks, "NETWORK_ID"),
        Ok(_) => {
            println!("No networks were returned by the orchestrator. Falling back to manual entry.");
            prompt_required("NETWORK_ID (which network this node joins)", "NETWORK_ID")
        }
        Err(err) => {
            println!("Could not load available networks: {}", err);
            prompt_required("NETWORK_ID (which network this node joins)", "NETWORK_ID")
        }
    };
    let bind_addr = choose_bind_addr("0.0.0.0:8090", "AGENT_BIND_ADDR");
    let heartbeat_interval_secs = prompt("HEARTBEAT_INTERVAL_SECS", "HEARTBEAT_INTERVAL_SECS", "60");
    let metrics_poll_interval_secs = prompt("METRICS_POLL_INTERVAL_SECS", "METRICS_POLL_INTERVAL_SECS", "30");
    let idle_cpu_threshold_pct = prompt("IDLE_CPU_THRESHOLD_PCT", "IDLE_CPU_THRESHOLD_PCT", "15.0");
    let preempt_cpu_threshold_pct = prompt("PREEMPT_CPU_THRESHOLD_PCT", "PREEMPT_CPU_THRESHOLD_PCT", "60.0");
    let idle_window_samples = prompt("IDLE_WINDOW_SAMPLES", "IDLE_WINDOW_SAMPLES", "10");
    let request_timeout_secs = prompt("REQUEST_TIMEOUT_SECS", "REQUEST_TIMEOUT_SECS", "10");

    let vars = vec![
        ("NODE_ID", node_id),
        ("NETWORK_ID", network_id),
        ("AGENT_BIND_ADDR", bind_addr),
        ("ORCHESTRATOR_BASE_URL", orchestrator_base_url),
        ("HEARTBEAT_INTERVAL_SECS", heartbeat_interval_secs),
        ("METRICS_POLL_INTERVAL_SECS", metrics_poll_interval_secs),
        ("IDLE_CPU_THRESHOLD_PCT", idle_cpu_threshold_pct),
        ("PREEMPT_CPU_THRESHOLD_PCT", preempt_cpu_threshold_pct),
        ("IDLE_WINDOW_SAMPLES", idle_window_samples),
        ("REQUEST_TIMEOUT_SECS", request_timeout_secs),
    ];

    println!();
    println!("Starting agent...");
    run_agent(&vars)
}
