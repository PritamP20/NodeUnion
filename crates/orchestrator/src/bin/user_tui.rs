use nodeunion_orchestrator::model::{NetworkRecord, NodeRecord, SubmitJobRequest};
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

fn prompt(label: &str, default: &str) -> String {
    print!("{} [{}]: ", label, default);
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return default.to_string();
    }

    let trimmed = input.trim();
    if trimmed.is_empty() {
        default.to_string()
    } else {
        trimmed.to_string()
    }
}

fn prompt_required(label: &str) -> String {
    loop {
        print!("{}: ", label);
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            let trimmed = input.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }

        println!("This field is required.");
    }
}

fn prompt_yes_no(label: &str, default_yes: bool) -> bool {
    let default_label = if default_yes { "Y/n" } else { "y/N" };
    print!("{} [{}]: ", label, default_label);
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return default_yes;
    }

    let trimmed = input.trim().to_ascii_lowercase();
    if trimmed.is_empty() {
        return default_yes;
    }

    matches!(trimmed.as_str(), "y" | "yes")
}

async fn fetch_json<T: DeserializeOwned>(client: &Client, url: &str) -> anyhow::Result<T> {
    let response = client.get(url).send().await?;
    let status = response.status();

    if !status.is_success() {
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<unreadable>".to_string());
        anyhow::bail!("GET {} failed with {}: {}", url, status, body);
    }

    let data = response.json::<T>().await?;
    Ok(data)
}

fn parse_command(input: &str) -> Option<Vec<String>> {
    let parts = input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    if parts.is_empty() {
        None
    } else {
        Some(parts)
    }
}

fn choose_network(networks: &[NetworkRecord], counts: &HashMap<String, usize>) -> anyhow::Result<NetworkRecord> {
    println!();
    println!("Available networks:");
    for (idx, network) in networks.iter().enumerate() {
        let node_count = counts.get(&network.network_id).copied().unwrap_or(0);
        println!(
            "  {}. {} ({}) - nodes: {}",
            idx + 1,
            network.name,
            network.network_id,
            node_count
        );
    }

    loop {
        let selected = prompt_required("Choose network number");
        if let Ok(index) = selected.parse::<usize>() {
            if index >= 1 && index <= networks.len() {
                return Ok(networks[index - 1].clone());
            }
        }
        println!("Please enter a valid number from the list.");
    }
}

fn docker_build(dockerfile_path: &str, context_path: &str, image_tag: &str) -> anyhow::Result<()> {
    println!();
    println!("Building docker image {} ...", image_tag);

    let status = Command::new("docker")
        .arg("build")
        .arg("-f")
        .arg(dockerfile_path)
        .arg("-t")
        .arg(image_tag)
        .arg(context_path)
        .status()?;

    if !status.success() {
        anyhow::bail!("docker build failed with status {}", status);
    }

    Ok(())
}

fn docker_push(image_tag: &str) -> anyhow::Result<()> {
    println!();
    println!("Pushing docker image {} ...", image_tag);

    let status = Command::new("docker").arg("push").arg(image_tag).status()?;
    if !status.success() {
        anyhow::bail!("docker push failed with status {}", status);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("NodeUnion User Deploy TUI");
    println!("This wizard builds your Dockerfile and submits a job to orchestrator.");

    let orchestrator_url = prompt("ORCHESTRATOR_URL", "http://127.0.0.1:8080").trim_end_matches('/').to_string();
    let client = Client::new();

    let networks_url = format!("{}/networks", orchestrator_url);
    let nodes_url = format!("{}/nodes", orchestrator_url);

    let networks: Vec<NetworkRecord> = fetch_json(&client, &networks_url).await?;
    let nodes: Vec<NodeRecord> = fetch_json(&client, &nodes_url).await?;

    if networks.is_empty() {
        anyhow::bail!("No networks available. Ask orchestrator admin to create one first.");
    }

    let mut node_counts: HashMap<String, usize> = HashMap::new();
    for node in &nodes {
        *node_counts.entry(node.network_id.clone()).or_insert(0) += 1;
    }

    let selected_network = choose_network(&networks, &node_counts)?;
    let selected_count = node_counts
        .get(&selected_network.network_id)
        .copied()
        .unwrap_or(0);

    println!();
    println!(
        "Selected network: {} ({}) with {} node(s)",
        selected_network.name, selected_network.network_id, selected_count
    );

    let user_wallet = prompt_required("Your wallet address");
    let dockerfile_path = prompt_required("Dockerfile path (for example: /path/to/Dockerfile)");

    if !Path::new(&dockerfile_path).exists() {
        anyhow::bail!("Dockerfile path does not exist: {}", dockerfile_path);
    }

    let default_context = Path::new(&dockerfile_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());

    let context_path = prompt("Docker build context path", &default_context);
    let image_tag = prompt("Image tag to build and submit", "nodeunion-user-job:latest");
    let command_raw = prompt("Command (comma separated, blank for image default)", "");
    let cpu_limit_raw = prompt("CPU limit", "0.25");
    let ram_limit_raw = prompt("RAM limit MB", "128");

    let cpu_limit = cpu_limit_raw.parse::<f64>().unwrap_or(0.25);
    let ram_limit_mb = ram_limit_raw.parse::<u64>().unwrap_or(128);
    let command = parse_command(&command_raw);

    docker_build(&dockerfile_path, &context_path, &image_tag)?;

    let should_push = prompt_yes_no(
        "Push image to registry now? (required for remote provider nodes)",
        true,
    );
    if should_push {
        docker_push(&image_tag)?;
    } else {
        println!(
            "Skipping push. Remote providers may fail to pull this image unless it already exists in a registry."
        );
    }

    let submit_payload = SubmitJobRequest {
        network_id: selected_network.network_id,
        user_wallet,
        image: image_tag,
        command,
        cpu_limit,
        ram_limit_mb,
    };

    let submit_url = format!("{}/jobs/submit", orchestrator_url);
    let response = client.post(&submit_url).json(&submit_payload).send().await?;
    let status = response.status();
    let body = response.text().await.unwrap_or_else(|_| "<unreadable>".to_string());

    println!();
    println!("Submit status: {}", status);
    println!("Response: {}", body);

    Ok(())
}
