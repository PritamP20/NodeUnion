use std::env;
use std::process::{Command, Stdio};

use anyhow::{anyhow, bail, Context, Result};

#[derive(Debug, Clone)]
struct QuickstartConfig {
    orchestrator_url: String,
    network_id: String,
    node_id: String,
    bind_addr: String,
    provider_wallet: Option<String>,
    agent_public_url: Option<String>,
    public_url_provider: String,
    auto_install_deps: bool,
}

fn usage() {
    eprintln!(
        "nodeunion-agent-quickstart\n\nUsage:\n  nodeunion-agent-quickstart --orchestrator-url <url> --network-id <id> [options]\n\nRequired:\n  --orchestrator-url URL\n  --network-id ID\n\nOptional:\n  --node-id ID\n  --bind-addr ADDR              (default: 0.0.0.0:8090)\n  --provider-wallet WALLET\n  --agent-public-url URL\n  --public-url-provider P       (cloudflare|none, default: cloudflare)\n  --no-auto-install\n  -h, --help\n\nExample:\n  nodeunion-agent-quickstart --orchestrator-url http://10.209.76.140:8080 --network-id 11"
    );
}

fn command_exists(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

fn run_ok(command: &str, args: &[&str]) -> bool {
    Command::new(command)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn normalize_url(raw: &str) -> String {
    let mut value = raw.trim().trim_end_matches('/').to_string();
    if value.starts_with("http://https://") {
        value = value.replacen("http://https://", "https://", 1);
    }
    if value.starts_with("https://http://") {
        value = value.replacen("https://http://", "http://", 1);
    }
    if !value.starts_with("http://") && !value.starts_with("https://") {
        value = format!("http://{}", value);
    }
    value
}

fn parse_args() -> Result<QuickstartConfig> {
    let mut orchestrator_url = env::var("ORCHESTRATOR_BASE_URL").unwrap_or_default();
    let mut network_id = env::var("NETWORK_ID").unwrap_or_default();
    let mut node_id = env::var("NODE_ID").unwrap_or_else(|_| format!("provider-{}", std::process::id()));
    let mut bind_addr = env::var("AGENT_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8090".to_string());
    let mut provider_wallet = env::var("PROVIDER_WALLET").ok();
    let mut agent_public_url = env::var("AGENT_PUBLIC_URL").ok();
    let mut public_url_provider =
        env::var("AGENT_PUBLIC_URL_PROVIDER").unwrap_or_else(|_| "cloudflare".to_string());
    let mut auto_install_deps = true;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--orchestrator-url" => {
                orchestrator_url = args
                    .next()
                    .ok_or_else(|| anyhow!("--orchestrator-url requires a value"))?;
            }
            "--network-id" => {
                network_id = args
                    .next()
                    .ok_or_else(|| anyhow!("--network-id requires a value"))?;
            }
            "--node-id" => {
                node_id = args
                    .next()
                    .ok_or_else(|| anyhow!("--node-id requires a value"))?;
            }
            "--bind-addr" => {
                bind_addr = args
                    .next()
                    .ok_or_else(|| anyhow!("--bind-addr requires a value"))?;
            }
            "--provider-wallet" => {
                provider_wallet = Some(
                    args.next()
                        .ok_or_else(|| anyhow!("--provider-wallet requires a value"))?,
                );
            }
            "--agent-public-url" => {
                agent_public_url = Some(
                    args.next()
                        .ok_or_else(|| anyhow!("--agent-public-url requires a value"))?,
                );
            }
            "--public-url-provider" => {
                public_url_provider = args
                    .next()
                    .ok_or_else(|| anyhow!("--public-url-provider requires a value"))?;
            }
            "--no-auto-install" => auto_install_deps = false,
            "-h" | "--help" => {
                usage();
                std::process::exit(0);
            }
            _ => bail!("unknown argument: {}", arg),
        }
    }

    if orchestrator_url.trim().is_empty() || network_id.trim().is_empty() {
        usage();
        bail!("--orchestrator-url and --network-id are required")
    }

    Ok(QuickstartConfig {
        orchestrator_url: normalize_url(&orchestrator_url),
        network_id,
        node_id,
        bind_addr,
        provider_wallet,
        agent_public_url: agent_public_url.map(|v| normalize_url(&v)),
        public_url_provider,
        auto_install_deps,
    })
}

fn ensure_docker() -> Result<()> {
    if !command_exists("docker") {
        bail!("Docker is required but not installed")
    }
    if !run_ok("docker", &["info"]) {
        bail!("Docker is installed but daemon is not running/reachable")
    }
    Ok(())
}

fn ensure_cloudflared(auto_install: bool) -> Result<()> {
    if command_exists("cloudflared") {
        return Ok(());
    }

    if !auto_install {
        bail!("cloudflared not found (pass --agent-public-url to skip auto tunnel)");
    }

    if command_exists("brew") {
        let status = Command::new("brew")
            .args(["install", "cloudflared"])
            .status()
            .context("failed to run brew install cloudflared")?;
        if status.success() {
            return Ok(());
        }
    }

    if command_exists("apt-get") {
        let update_ok = Command::new("sudo")
            .args(["apt-get", "update"])
            .status()
            .context("failed to run apt-get update")?
            .success();
        let install_ok = Command::new("sudo")
            .args(["apt-get", "install", "-y", "cloudflared"])
            .status()
            .context("failed to run apt-get install cloudflared")?
            .success();

        if update_ok && install_ok {
            return Ok(());
        }
    }

    bail!("could not auto-install cloudflared. Install it manually or pass --agent-public-url")
}

async fn ensure_orchestrator_health(base_url: &str) -> Result<()> {
    let health_url = format!("{}/health", base_url.trim_end_matches('/'));
    let response = reqwest::get(&health_url)
        .await
        .with_context(|| format!("failed to call {}", health_url))?;

    if !response.status().is_success() {
        bail!("orchestrator health check failed with status {}", response.status());
    }

    Ok(())
}

fn run_agent(config: &QuickstartConfig) -> Result<()> {
    let mut cmd = Command::new("nodeunion-agent");
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .env("NODE_ID", &config.node_id)
        .env("NETWORK_ID", &config.network_id)
        .env("AGENT_BIND_ADDR", &config.bind_addr)
        .env("ORCHESTRATOR_BASE_URL", &config.orchestrator_url)
        .env("AGENT_PUBLIC_URL_PROVIDER", &config.public_url_provider);

    if let Some(wallet) = &config.provider_wallet {
        cmd.env("PROVIDER_WALLET", wallet);
    }
    if let Some(public_url) = &config.agent_public_url {
        cmd.env("AGENT_PUBLIC_URL", public_url);
    }

    let status = cmd.status().context("failed to start nodeunion-agent")?;
    if !status.success() {
        bail!("nodeunion-agent exited with status {}", status);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = parse_args()?;

    ensure_docker()?;
    ensure_orchestrator_health(&config.orchestrator_url).await?;

    if config.agent_public_url.is_none() && config.public_url_provider.eq_ignore_ascii_case("cloudflare") {
        ensure_cloudflared(config.auto_install_deps)?;
    }

    eprintln!("Starting node provider with node_id={} network_id={}", config.node_id, config.network_id);
    run_agent(&config)
}
