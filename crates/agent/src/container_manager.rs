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
use serde::Deserialize;
use std::process::{Command, Stdio};
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone)]
pub struct DeploymentResult {
    pub container_id: String,
    pub deploy_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NgrokTunnelsResponse {
    tunnels: Vec<NgrokTunnel>,
}

#[derive(Debug, Deserialize)]
struct NgrokTunnel {
    public_url: String,
    config: NgrokTunnelConfig,
}

#[derive(Debug, Deserialize)]
struct NgrokTunnelConfig {
    addr: String,
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

    if let Some(bindings) = port_bindings {
        for (container_port, host_port) in bindings {
            // Add to exposed ports
            exposed_ports.insert(container_port.clone(), HashMap::new());
            
            // Add to port bindings - format: "8080/tcp" -> [PortBinding { host_ip: "127.0.0.1", host_port: "8000" }]
            let binding = vec![PortBinding {
                host_ip: Some("127.0.0.1".to_string()),
                host_port: Some(host_port.to_string()),
            }];
            port_bindings_map.insert(container_port, Some(binding));
        }
    } else if let Some(exposed_port) = req.exposed_port {
        let port_key = format!("{}/tcp", exposed_port);
        exposed_ports.insert(port_key.clone(), HashMap::new());
        port_bindings_map.insert(
            port_key,
            Some(vec![PortBinding {
                host_ip: Some("127.0.0.1".to_string()),
                host_port: Some(exposed_port.to_string()),
            }]),
        );
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

    let deploy_url = if let Some(exposed_port) = req.exposed_port {
        maybe_expose_with_ngrok(exposed_port).await
    } else {
        None
    };

    Ok(DeploymentResult {
        container_id: create_result.id,
        deploy_url,
    })
}

fn should_auto_expose_with_ngrok() -> bool {
    std::env::var("AUTO_NGROK_EXPOSE")
        .ok()
        .map(|value| matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(true)
}

async fn maybe_expose_with_ngrok(port: u16) -> Option<String> {
    if !should_auto_expose_with_ngrok() {
        return None;
    }

    if !Command::new("ngrok")
        .arg("version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
    {
        return None;
    }

    if let Some(existing) = find_ngrok_tunnel(port).await {
        return Some(existing);
    }

    let _ = Command::new("ngrok")
        .arg("http")
        .arg(port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    for _ in 0..12 {
        if let Some(url) = find_ngrok_tunnel(port).await {
            return Some(url);
        }
        sleep(Duration::from_millis(500)).await;
    }

    None
}

async fn find_ngrok_tunnel(port: u16) -> Option<String> {
    let url = std::env::var("NGROK_API_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "http://127.0.0.1:4040/api/tunnels".to_string());

    let response = reqwest::get(url).await.ok()?;
    let body = response.json::<NgrokTunnelsResponse>().await.ok()?;
    let needle = format!(":{}", port);

    body.tunnels
        .into_iter()
        .find(|tunnel| tunnel.config.addr.ends_with(&needle) || tunnel.config.addr == port.to_string())
        .map(|tunnel| tunnel.public_url)
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