use std::env;
use std::io::{self, Write};
use std::net::{IpAddr, UdpSocket};
use std::process::{Command, Stdio};

use nodeunion_orchestrator::dashboard;

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

fn prompt_yes_no(label: &str, env_key: &str, default: bool) -> bool {
    let default_label = if default { "Y/n" } else { "y/N" };
    let current = env::var(env_key).ok();

    loop {
        if let Some(value) = &current {
            print!("{} [{}]: ", label, value);
        } else {
            print!("{} [{}]: ", label, default_label);
        }
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return current
                .as_deref()
                .map(|value| matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "y" | "on"))
                .unwrap_or(default);
        }

        let trimmed = input.trim();
        let selection = if trimmed.is_empty() {
            current.clone().unwrap_or_else(|| default.to_string())
        } else {
            trimmed.to_string()
        };

        match selection.to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "y" | "on" => return true,
            "0" | "false" | "no" | "n" | "off" => return false,
            _ => println!("Please answer yes or no."),
        }
    }
}

fn detect_local_ip() -> Option<IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let addr = socket.local_addr().ok()?;
    Some(addr.ip())
}

fn advertised_url(bind_addr: &str) -> Option<String> {
    let (host, port) = bind_addr.rsplit_once(':')?;

    let public_host = if host == "0.0.0.0" {
        detect_local_ip()?.to_string()
    } else {
        host.to_string()
    };

    Some(format!("http://{}:{}", public_host, port))
}

fn spawn_orchestrator(vars: &[(&str, String)]) -> anyhow::Result<std::process::Child> {
    let mut cmd = Command::new("nodeunion-orchestrator");
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    for (k, v) in vars {
        cmd.env(k, v);
    }

    match cmd.spawn() {
        Ok(child) => Ok(child),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            let mut fallback = Command::new("cargo");
            fallback
                .arg("run")
                .arg("-p")
                .arg("nodeunion-orchestrator")
                .arg("--bin")
                .arg("nodeunion-orchestrator")
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());

            for (k, v) in vars {
                fallback.env(k, v);
            }

            Ok(fallback.spawn()?)
        }
        Err(err) => Err(err.into()),
    }
}

fn run_orchestrator(vars: &[(&str, String)]) -> anyhow::Result<()> {
    let mut child = spawn_orchestrator(vars)?;
    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("nodeunion-orchestrator exited with status {}", status))
    }
}

fn main() -> anyhow::Result<()> {
    println!("NodeUnion Orchestrator Launch Form");
    println!("Fill values and press enter to keep defaults where available.");
    println!();

    let database_url = prompt_required("DATABASE_URL", "DATABASE_URL");
    let solana_rpc_url = prompt(
        "SOLANA_RPC_URL",
        "SOLANA_RPC_URL",
        "https://api.devnet.solana.com",
    );
    let solana_payer_keypair = prompt_required("SOLANA_PAYER_KEYPAIR", "SOLANA_PAYER_KEYPAIR");
    let solana_program_id = prompt_required("SOLANA_PROGRAM_ID", "SOLANA_PROGRAM_ID");
    let bind_addr = prompt("ORCHESTRATOR_BIND_ADDR", "ORCHESTRATOR_BIND_ADDR", "0.0.0.0:8080");
    let managed_network_id = prompt(
        "ORCHESTRATOR_NETWORK_ID (required for single-network mode)",
        "ORCHESTRATOR_NETWORK_ID",
        "",
    );
    let managed_network_name = prompt(
        "ORCHESTRATOR_NETWORK_NAME",
        "ORCHESTRATOR_NETWORK_NAME",
        "",
    );
    let managed_network_description = prompt(
        "ORCHESTRATOR_NETWORK_DESCRIPTION",
        "ORCHESTRATOR_NETWORK_DESCRIPTION",
        "",
    );
    let default_public_url = advertised_url(&bind_addr).unwrap_or_default();
    let orchestrator_public_url = prompt(
        "ORCHESTRATOR_PUBLIC_URL (what users/providers should call)",
        "ORCHESTRATOR_PUBLIC_URL",
        &default_public_url,
    );
    let orchestrator_dashboard_url = orchestrator_public_url.clone();
    let open_dashboard_after_start = prompt_yes_no(
        "Open live dashboard after startup",
        "OPEN_DASHBOARD_AFTER_START",
        true,
    );

    let vars = vec![
        ("DATABASE_URL", database_url),
        ("SOLANA_RPC_URL", solana_rpc_url),
        ("SOLANA_PAYER_KEYPAIR", solana_payer_keypair),
        ("SOLANA_PROGRAM_ID", solana_program_id),
        ("ORCHESTRATOR_BIND_ADDR", bind_addr.clone()),
        ("ORCHESTRATOR_NETWORK_ID", managed_network_id),
        ("ORCHESTRATOR_NETWORK_NAME", managed_network_name),
        ("ORCHESTRATOR_NETWORK_DESCRIPTION", managed_network_description),
        ("ORCHESTRATOR_PUBLIC_URL", orchestrator_public_url),
    ];

    println!();
    if let Some(url) = advertised_url(&bind_addr) {
        println!("Detected orchestrator URL to share: {}", url);
        println!("Health check after start: {}/health", url);
        println!();
    }
    println!("Starting orchestrator...");

    if open_dashboard_after_start {
        let mut child = spawn_orchestrator(&vars)?;
        let dashboard_result = tokio::runtime::Runtime::new()?
            .block_on(dashboard::run(orchestrator_dashboard_url))
            .map_err(anyhow::Error::from);
        let _ = child.kill();
        let _ = child.wait();
        dashboard_result
    } else {
        run_orchestrator(&vars)
    }
}
