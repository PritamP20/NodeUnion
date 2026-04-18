use std::env;
use std::io::{self, Write};
use std::net::{IpAddr, UdpSocket};
use std::process::{Command, Stdio};

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

fn run_orchestrator(vars: &[(&str, String)]) -> anyhow::Result<()> {
    let mut cmd = Command::new("nodeunion-orchestrator");
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
                Err(anyhow::anyhow!("nodeunion-orchestrator exited with status {}", status))
            }
        }
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
    run_orchestrator(&vars)
}
