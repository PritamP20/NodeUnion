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

fn local_dashboard_url(bind_addr: &str) -> String {
    let (_, port) = bind_addr.rsplit_once(':').unwrap_or(("0.0.0.0", "8080"));
    format!("http://127.0.0.1:{}", port)
}

fn command_exists(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

fn try_install_cloudflared() -> anyhow::Result<bool> {
    if command_exists("cloudflared") {
        return Ok(true);
    }

    if cfg!(target_os = "windows") {
        if command_exists("winget") {
            println!("cloudflared not found. Attempting install via winget...");
            let status = Command::new("winget")
                .arg("install")
                .arg("--id")
                .arg("Cloudflare.cloudflared")
                .arg("-e")
                .arg("--accept-source-agreements")
                .arg("--accept-package-agreements")
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()?;

            if status.success() && command_exists("cloudflared") {
                return Ok(true);
            }
        }

        if command_exists("choco") {
            println!("cloudflared not found. Attempting install via choco...");
            let status = Command::new("choco")
                .arg("install")
                .arg("cloudflared")
                .arg("-y")
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()?;

            if status.success() && command_exists("cloudflared") {
                return Ok(true);
            }
        }
    }

    if command_exists("brew") {
        println!("cloudflared not found. Attempting install via brew...");
        let status = Command::new("brew")
            .arg("install")
            .arg("cloudflared")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;
        if status.success() && command_exists("cloudflared") {
            return Ok(true);
        }
    }

    if command_exists("apt-get") {
        println!("cloudflared not found. Attempting install via apt-get...");
        let update = Command::new("sudo")
            .arg("apt-get")
            .arg("update")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        let install = Command::new("sudo")
            .arg("apt-get")
            .arg("install")
            .arg("-y")
            .arg("cloudflared")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        if update.success() && install.success() && command_exists("cloudflared") {
            return Ok(true);
        }
    }

    Ok(false)
}

fn spawn_orchestrator(vars: &[(&str, String)], quiet_output: bool) -> anyhow::Result<std::process::Child> {
    let mut cmd = Command::new("nodeunion-orchestrator");
    cmd.stdin(Stdio::inherit());

    if quiet_output {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    } else {
        cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    }

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
                .stdin(Stdio::inherit());

            if quiet_output {
                fallback.stdout(Stdio::null()).stderr(Stdio::null());
            } else {
                fallback.stdout(Stdio::inherit()).stderr(Stdio::inherit());
            }

            for (k, v) in vars {
                fallback.env(k, v);
            }

            Ok(fallback.spawn()?)
        }
        Err(err) => Err(err.into()),
    }
}

fn run_orchestrator(vars: &[(&str, String)]) -> anyhow::Result<()> {
    let mut child = spawn_orchestrator(vars, false)?;
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
    let stellar_network = prompt("STELLAR_NETWORK", "STELLAR_NETWORK", "testnet");
    let stellar_source_account = prompt_required("STELLAR_SOURCE_ACCOUNT", "STELLAR_SOURCE_ACCOUNT");
    let stellar_contract_id = prompt_required("STELLAR_CONTRACT_ID", "STELLAR_CONTRACT_ID");
    let stellar_rate_per_unit = prompt("STELLAR_RATE_PER_UNIT", "STELLAR_RATE_PER_UNIT", "100");
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
    let managed_network_price_per_unit = prompt(
        "ORCHESTRATOR_NETWORK_PRICE_PER_UNIT (tokens per compute unit)",
        "ORCHESTRATOR_NETWORK_PRICE_PER_UNIT",
        "100",
    );
    let orchestrator_public_url = prompt(
        "ORCHESTRATOR_PUBLIC_URL (optional explicit public URL; leave blank for auto)",
        "ORCHESTRATOR_PUBLIC_URL",
        "",
    );
    let mut orchestrator_public_url_provider = prompt(
        "ORCHESTRATOR_PUBLIC_URL_PROVIDER (cloudflare or none)",
        "ORCHESTRATOR_PUBLIC_URL_PROVIDER",
        "cloudflare",
    );
    let orchestrator_dashboard_url = local_dashboard_url(&bind_addr);

    if orchestrator_public_url.trim().is_empty()
        && orchestrator_public_url_provider.trim().eq_ignore_ascii_case("cloudflare")
        && !command_exists("cloudflared")
    {
        if try_install_cloudflared()? {
            println!("cloudflared installed successfully.");
        } else {
            println!(
                "cloudflared is not available; falling back to ORCHESTRATOR_PUBLIC_URL_PROVIDER=none for this run."
            );
            orchestrator_public_url_provider = "none".to_string();
        }
    }
    let open_dashboard_after_start = prompt_yes_no(
        "Open live dashboard after startup",
        "OPEN_DASHBOARD_AFTER_START",
        true,
    );

    let vars = vec![
        ("DATABASE_URL", database_url),
        ("STELLAR_NETWORK", stellar_network),
        ("STELLAR_SOURCE_ACCOUNT", stellar_source_account),
        ("STELLAR_CONTRACT_ID", stellar_contract_id),
        ("STELLAR_RATE_PER_UNIT", stellar_rate_per_unit),
        ("ORCHESTRATOR_BIND_ADDR", bind_addr.clone()),
        ("ORCHESTRATOR_NETWORK_ID", managed_network_id),
        ("ORCHESTRATOR_NETWORK_NAME", managed_network_name),
        ("ORCHESTRATOR_NETWORK_DESCRIPTION", managed_network_description),
        ("ORCHESTRATOR_NETWORK_PRICE_PER_UNIT", managed_network_price_per_unit),
        ("ORCHESTRATOR_PUBLIC_URL", orchestrator_public_url),
        ("ORCHESTRATOR_PUBLIC_URL_PROVIDER", orchestrator_public_url_provider),
    ];

    println!();
    if let Some(url) = advertised_url(&bind_addr) {
        println!("Detected orchestrator URL to share: {}", url);
        println!("Health check after start: {}/health", url);
        println!("Local dashboard URL: {}", orchestrator_dashboard_url);
        println!();
    }
    println!("Starting orchestrator...");

    if open_dashboard_after_start {
        let mut child = spawn_orchestrator(&vars, true)?;
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
