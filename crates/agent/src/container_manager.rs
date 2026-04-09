use anyhow::{Context, Result};
use bollard::container::{
    Config as ContainerConfig, CreateContainerOptions, RemoveContainerOptions,
    StartContainerOptions, StopContainerOptions,
};
use std::collections::HashMap;
use bollard::models::{HostConfig, PortBinding};
use bollard::Docker;
use crate::models::RunJobRequest;

/// Run a container with the specified configuration.
/// If port_bindings is provided, maps container ports to host ports.
/// Format for port_bindings: "8080/tcp" -> 8000 (container port -> host port)
pub async fn run_container(docker: &Docker, req: &RunJobRequest) -> Result<String> {
    run_container_with_ports(docker, req, None).await
}

/// Run a container with custom port bindings for testing and development.
/// port_bindings: HashMap<"port/protocol", host_port>
/// Example: ("8080/tcp", 8000) maps container:8080 -> host:8000
pub async fn run_container_with_ports(
    docker: &Docker,
    req: &RunJobRequest,
    port_bindings: Option<HashMap<String, u16>>,
) -> Result<String> {
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

    Ok(create_result.id)
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