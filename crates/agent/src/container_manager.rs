use anyhow::{Context, Result};
use bollard::container::{
    Config as ContainerConfig, CreateContainerOptions, RemoveContainerOptions,
    StartContainerOptions, StopContainerOptions,
};
use bollard::errors::Error as BollardError;
use bollard::image::CreateImageOptions;
use futures_util::TryStreamExt;
use std::collections::HashMap;
use bollard::models::{HostConfig, PortBinding};
use bollard::Docker;
use crate::models::RunJobRequest;
use std::process::{Command, Stdio, Child};
use std::sync::mpsc;
use std::thread::spawn as spawn_thread;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct DeploymentResult {
    pub container_id: String,
    pub deploy_url: Option<String>,
}

async fn ensure_image_available(docker: &Docker, image: &str) -> Result<()> {
    match docker.inspect_image(image).await {
        Ok(_) => Ok(()),
        Err(BollardError::DockerResponseServerError { status_code: 404, .. }) => {
            let options = Some(CreateImageOptions {
                from_image: image,
                ..Default::default()
            });

            docker
                .create_image(options, None, None)
                .try_collect::<Vec<_>>()
                .await
                .with_context(|| format!("failed to pull missing image {}", image))?;

            Ok(())
        }
        Err(err) => Err(err).with_context(|| format!("failed to inspect image {}", image)),
    }
}

/// Run a container with the specified configuration.
/// If port_bindings is provided, maps container ports to host ports.
/// Format for port_bindings: "8080/tcp" -> 8000 (container port -> host port)
pub async fn run_container(docker: &Docker, req: &RunJobRequest) -> Result<DeploymentResult> {
    run_container_with_ports(docker, req, None).await
}

/// Run a container with custom port bindings for testing and development.
/// port_bindings: HashMap<"port/protocol", host_port>
/// Example: ("8080/tcp", 8000) maps container:8080 -> host:8000
pub async fn run_container_with_ports(
    docker: &Docker,
    req: &RunJobRequest,
    port_bindings: Option<HashMap<String, u16>>,
) -> Result<DeploymentResult> {
    ensure_image_available(docker, &req.image).await?;

    let name = format!("job-{}-{}", req.job_id, req.chunk_id);
    let nano_cpus = (req.cpu_limit * 1_000_000_000.0) as i64;
    let memory_bytes = (req.ram_limit_mb as i64) * 1024 * 1024;

    // Build port bindings if provided
    let mut exposed_ports = HashMap::new();
    let mut port_bindings_map: HashMap<String, Option<Vec<PortBinding>>> = HashMap::new();
    let mut public_probe_port: Option<u16> = None;

    if let Some(bindings) = port_bindings {
        for (container_port, host_port) in bindings {
            // Add to exposed ports
            exposed_ports.insert(container_port.clone(), HashMap::new());
            if public_probe_port.is_none() {
                public_probe_port = Some(host_port);
            }
            
            // Add to port bindings - format: "8080/tcp" -> [PortBinding { host_ip: "127.0.0.1", host_port: "8000" }]
            let binding = vec![PortBinding {
                host_ip: Some("127.0.0.1".to_string()),
                host_port: Some(host_port.to_string()),
            }];
            port_bindings_map.insert(container_port, Some(binding));
        }
    } else if let Some(exposed_port) = req.exposed_port {
        let host_port = find_free_local_port().with_context(|| {
            format!(
                "failed to allocate host port for container exposed port {}",
                exposed_port
            )
        })?;
        let port_key = format!("{}/tcp", exposed_port);
        exposed_ports.insert(port_key.clone(), HashMap::new());
        port_bindings_map.insert(
            port_key,
            Some(vec![PortBinding {
                host_ip: Some("127.0.0.1".to_string()),
                host_port: Some(host_port.to_string()),
            }]),
        );
        public_probe_port = Some(host_port);
    }

    let host_config = HostConfig {
        nano_cpus: Some(nano_cpus),
        memory: Some(memory_bytes),
        port_bindings: if port_bindings_map.is_empty() { None } else { Some(port_bindings_map) },
        ..Default::default()
    };

    let container_config = ContainerConfig {
        image: Some(req.image.clone()),
        cmd: req.command.clone(),
        host_config: Some(host_config),
        tty: Some(false),
        exposed_ports: if exposed_ports.is_empty() { None } else { Some(exposed_ports) },
        ..Default::default()
    };

    let create_result = docker
        .create_container(
            Some(CreateContainerOptions {
                name: name.clone(),
                platform: None
            }),
            container_config
        )
        .await
        .with_context(|| format!("failed to create container {}", name))?;

    docker
        .start_container(&create_result.id, None::<StartContainerOptions<String>>)
        .await
        .with_context(|| format!("failed to start container {}", create_result.id))?;

    if let Err(err) = wait_for_container_ready(docker, &create_result.id, public_probe_port).await {
        let _ = docker
            .remove_container(
                &create_result.id,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await;
        return Err(err).with_context(|| format!("container {} failed readiness checks", create_result.id));
    }

    let deploy_url = if let Some(host_port) = public_probe_port {
        maybe_expose_with_cloudflare(host_port).await
    } else {
        None
    };

    Ok(DeploymentResult {
        container_id: create_result.id,
        deploy_url,
    })
}

fn find_free_local_port() -> Result<u16> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .context("failed to bind an ephemeral local port")?;
    let port = listener
        .local_addr()
        .context("failed to read ephemeral local port")?
        .port();
    Ok(port)
}

async fn wait_for_container_ready(
    docker: &Docker,
    container_id: &str,
    host_port: Option<u16>,
) -> Result<()> {
    let deadline = std::time::Instant::now() + Duration::from_secs(20);

    loop {
        if std::time::Instant::now() >= deadline {
            return Err(anyhow::anyhow!(
                "timed out waiting for container {} to become ready",
                container_id
            ));
        }

        let info = docker
            .inspect_container(container_id, None)
            .await
            .with_context(|| format!("failed to inspect container {}", container_id))?;

        if let Some(state) = info.state {
            if state.running == Some(false) {
                let exit = state.exit_code.unwrap_or(-1);
                return Err(anyhow::anyhow!(
                    "container exited before readiness with exit code {}",
                    exit
                ));
            }

            if state.running == Some(true) {
                if let Some(port) = host_port {
                    if TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
                        return Ok(());
                    }
                } else {
                    return Ok(());
                }
            }
        }

        sleep(Duration::from_millis(500)).await;
    }
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
    spawn_thread(move || {
        use std::io::{BufRead, BufReader};
        let reader = BufReader::new(reader);
        for line in reader.lines() {
            if let Ok(line) = line {
                let _ = tx.send(line);
            }
        }
    });
}

fn start_cloudflare_tunnel_for_port(port: u16) -> Result<(Child, String)> {
    let local_url = format!("http://127.0.0.1:{}", port);
    let mut child = Command::new("cloudflared")
        .args(["tunnel", "--url", &local_url, "--no-autoupdate"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to start cloudflared for container port")?;

    let stdout = child
        .stdout
        .take()
        .context("cloudflared stdout unavailable")?;
    let stderr = child
        .stderr
        .take()
        .context("cloudflared stderr unavailable")?;

    let (tx, rx) = mpsc::channel();
    spawn_line_reader(stdout, tx.clone());
    spawn_line_reader(stderr, tx);

    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    let mut first_https: Option<String> = None;

    loop {
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            return Err(anyhow::anyhow!("timed out waiting for cloudflared container tunnel"));
        }

        if let Ok(Some(status)) = child.try_wait() {
            let _ = child.kill();
            return Err(anyhow::anyhow!("cloudflared exited early with status {}", status));
        }

        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(line) => {
                if let Some(url) = extract_https_url(&line) {
                    if url.contains("trycloudflare.com") {
                        return Ok((child, url));
                    }
                    if first_https.is_none() {
                        first_https = Some(url);
                    }
                }
            }
            Err(_) => {}
        }
    }
}

async fn maybe_expose_with_cloudflare(port: u16) -> Option<String> {
    match start_cloudflare_tunnel_for_port(port) {
        Ok((_child, url)) => Some(url),
        Err(_) => None,
    }
}

pub async fn stop_container(docker: &Docker, container_id: &str) -> Result<()> {
    docker
        .stop_container(container_id, Some(StopContainerOptions { t: 10 }))
        .await
        .with_context(|| format!("failed to stop container {}", container_id))?;

    docker
        .remove_container(
            container_id,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        )
        .await
        .with_context(|| format!("failed to remove container {}", container_id))?;

    Ok(())
}