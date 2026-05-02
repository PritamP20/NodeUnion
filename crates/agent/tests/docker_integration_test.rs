// Docker integration tests for container lifecycle and app accessibility.
// These tests verify the daemon can actually start containers and access
// applications running inside them.
//
// REQUIREMENTS: Docker daemon must be running locally
// SETUP: Pull test image before running:  docker pull nginx:alpine

use nodeunion_agent::container_manager::{run_container_with_ports, stop_container};
use nodeunion_agent::models::RunJobRequest;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

/// Test: Start a simple nginx container, verify it's accessible on localhost
#[tokio::test]
#[ignore = "requires Docker daemon and docker pull nginx:alpine"]
async fn test_run_nginx_container_and_access_app() {
    let docker = bollard::Docker::connect_with_socket_defaults()
        .expect("Docker daemon must be running");

    // Create request to run nginx
    let req = RunJobRequest {
        job_id: "test-job-nginx".to_string(),
        chunk_id: "chunk-001".to_string(),
        image: "nginx:alpine".to_string(),
        cpu_limit: 0.5,
        ram_limit_mb: 128,
        exposed_port: None,
        input_path: None,
        command: None, // nginx starts by default
        env: None,
    };

    // Bind nginx port 80 to host port 8765 for testing
    let mut port_bindings = HashMap::new();
    port_bindings.insert("80/tcp".to_string(), 8765);

    // Start the container
    let deployment = run_container_with_ports(&docker, &req, Some(port_bindings))
        .await
        .expect("failed to start nginx container");
    let container_id = deployment.container_id;

    println!("✓ Started nginx container: {}", container_id);

    // Wait for nginx to start
    sleep(Duration::from_millis(500)).await;

    // Try to access the app
    let client = reqwest::Client::new();
    let url = "http://127.0.0.1:8765/";
    
    // Give it a few tries in case nginx is still starting
    let mut response = None;
    for attempt in 0..5 {
        match tokio::time::timeout(
            Duration::from_secs(2),
            client.get(url).send()
        ).await {
            Ok(Ok(resp)) => {
                response = Some(resp);
                break;
            }
            Ok(Err(e)) => {
                println!("Attempt {}: Connection error: {}", attempt + 1, e);
                sleep(Duration::from_millis(200)).await;
            }
            Err(_) => {
                println!("Attempt {}: Timeout connecting to {}", attempt + 1, url);
                sleep(Duration::from_millis(200)).await;
            }
        }
    }

    let resp = response.expect("Failed to connect to nginx after 5 attempts");
    assert_eq!(resp.status(), 200, "Expected 200 OK from nginx");

    let body = resp.text().await.expect("failed to read response body");
    assert!(body.contains("nginx"), "Response should contain 'nginx' welcome page");
    
    println!("✓ Successfully accessed nginx at http://127.0.0.1:8765/");
    println!("  Response body (first 100 chars): {}", &body[..body.len().min(100)]);

    // Stop and clean up the container
    stop_container(&docker, &container_id)
        .await
        .expect("failed to stop container");

    println!("✓ Stopped and removed container");
}

/// Test: Start a simple Python HTTP server, verify it responds
#[tokio::test]
#[ignore = "requires Docker daemon and docker pull python:3.11-alpine"]
async fn test_run_python_http_server_and_access() {
    let docker = bollard::Docker::connect_with_socket_defaults()
        .expect("Docker daemon must be running");

    // Create request to run Python HTTP server
    let req = RunJobRequest {
        job_id: "test-job-python".to_string(),
        chunk_id: "chunk-002".to_string(),
        image: "python:3.11-alpine".to_string(),
        cpu_limit: 0.5,
        ram_limit_mb: 256,
        exposed_port: None,
        input_path: None,
        command: Some(vec![
            "python".to_string(),
            "-m".to_string(),
            "http.server".to_string(),
            "8000".to_string(),
        ]),
        env: None,
    };

    // Bind Python server port 8000 to host port 8766
    let mut port_bindings = HashMap::new();
    port_bindings.insert("8000/tcp".to_string(), 8766);

    // Start the container
    let deployment = run_container_with_ports(&docker, &req, Some(port_bindings))
        .await
        .expect("failed to start Python HTTP server");
    let container_id = deployment.container_id;

    println!("✓ Started Python HTTP server container: {}", container_id);

    // Wait for server to start
    sleep(Duration::from_millis(500)).await;

    // Try to access the server
    let client = reqwest::Client::new();
    let url = "http://127.0.0.1:8766/";

    let mut response = None;
    for attempt in 0..5 {
        match tokio::time::timeout(
            Duration::from_secs(2),
            client.get(url).send()
        ).await {
            Ok(Ok(resp)) => {
                response = Some(resp);
                break;
            }
            Ok(Err(e)) => {
                println!("Attempt {}: Connection error: {}", attempt + 1, e);
                sleep(Duration::from_millis(200)).await;
            }
            Err(_) => {
                println!("Attempt {}: Timeout connecting to {}", attempt + 1, url);
                sleep(Duration::from_millis(200)).await;
            }
        }
    }

    let resp = response.expect("Failed to connect to Python server after 5 attempts");
    assert_eq!(resp.status(), 200, "Expected 200 OK from Python server");

    let body = resp.text().await.expect("failed to read response body");
    println!("✓ Successfully accessed Python HTTP server at http://127.0.0.1:8766/");
    assert!(!body.is_empty(), "Expected Python server response body to be non-empty");

    // Stop and clean up the container
    stop_container(&docker, &container_id)
        .await
        .expect("failed to stop container");

    println!("✓ Stopped and removed container");
}

/// Test: Verify container resource limits are applied
#[tokio::test]
#[ignore = "requires Docker daemon and docker pull alpine:latest"]
async fn test_container_resource_limits_applied() {
    let docker = bollard::Docker::connect_with_socket_defaults()
        .expect("Docker daemon must be running");

    let req = RunJobRequest {
        job_id: "test-job-limits".to_string(),
        chunk_id: "chunk-003".to_string(),
        image: "alpine:latest".to_string(),
        cpu_limit: 0.25, // 250m = 250_000_000 nano_cpus
        ram_limit_mb: 64, // 64 MB
        exposed_port: None,
        input_path: None,
        command: Some(vec!["sleep".to_string(), "5".to_string()]),
        env: None,
    };

    let deployment = run_container_with_ports(&docker, &req, None)
        .await
        .expect("failed to start container with resource limits");
    let container_id = deployment.container_id;

    println!("✓ Started container with resource limits: {}", container_id);

    // Inspect the container to verify limits were applied
    match docker.inspect_container(&container_id, None).await {
        Ok(info) => {
            if let Some(host_config) = info.host_config {
                // Verify CPU limit (250_000_000 nano_cpus = 0.25 CPU)
                assert_eq!(
                    host_config.nano_cpus,
                    Some(250_000_000),
                    "CPU limit should be 250_000_000 nano_cpus"
                );

                // Verify memory limit (64 MB = 67_108_864 bytes)
                assert_eq!(
                    host_config.memory,
                    Some(64 * 1024 * 1024),
                    "Memory limit should be {} bytes",
                    64 * 1024 * 1024
                );

                println!("✓ CPU limit correctly set to 0.25 CPU (250m)");
                println!("✓ Memory limit correctly set to 64 MB");
            }
        }
        Err(e) => panic!("Failed to inspect container: {}", e),
    }

    // Give container time to run then stop it
    sleep(Duration::from_millis(500)).await;
    stop_container(&docker, &container_id)
        .await
        .expect("failed to stop container");

    println!("✓ Stopped and removed container");
}

/// Test: Multiple containers can be started with different ports
#[tokio::test]
#[ignore = "requires Docker daemon and docker pull nginx:alpine"]
async fn test_multiple_containers_with_different_ports() {
    let docker = bollard::Docker::connect_with_socket_defaults()
        .expect("Docker daemon must be running");

    let mut containers = vec![];

    // Start 2 nginx containers on different ports
    for i in 0..2 {
        let req = RunJobRequest {
            job_id: format!("test-job-multi-{}", i),
            chunk_id: format!("chunk-multi-{:03}", i),
            image: "nginx:alpine".to_string(),
            cpu_limit: 0.5,
            ram_limit_mb: 128,
            exposed_port: None,
            input_path: None,
            command: None,
            env: None,
        };

        let host_port = 8767 + i as u16;
        let mut port_bindings = HashMap::new();
        port_bindings.insert("80/tcp".to_string(), host_port);

        let deployment = run_container_with_ports(&docker, &req, Some(port_bindings))
            .await
            .expect(&format!("failed to start container {}", i));
        let container_id = deployment.container_id;

        println!("✓ Started container {} on port {}: {}", i, host_port, container_id);
        containers.push((container_id, host_port));
    }

    sleep(Duration::from_millis(500)).await;

    // Verify both are accessible
    let client = reqwest::Client::new();
    for (_container_id, host_port) in &containers {
        let url = format!("http://127.0.0.1:{}/", host_port);
        let resp = tokio::time::timeout(
            Duration::from_secs(3),
            client.get(&url).send()
        )
        .await
        .expect(&format!("Timeout accessing container on port {}", host_port))
        .expect(&format!("Failed to connect to container on port {}", host_port));

        assert_eq!(resp.status(), 200);
        println!("✓ Successfully accessed container on port {}", host_port);
    }

    // Clean up
    for (container_id, _) in containers {
        stop_container(&docker, &container_id)
            .await
            .expect("failed to stop container");
    }

    println!("✓ Stopped all containers");
}

/// Test: Container stops and removes cleanly
#[tokio::test]
#[ignore = "requires Docker daemon and docker pull alpine:latest"]
async fn test_container_stops_and_removes() {
    let docker = bollard::Docker::connect_with_socket_defaults()
        .expect("Docker daemon must be running");

    let req = RunJobRequest {
        job_id: "test-job-stop".to_string(),
        chunk_id: "chunk-stop".to_string(),
        image: "alpine:latest".to_string(),
        cpu_limit: 0.5,
        ram_limit_mb: 128,
        exposed_port: None,
        input_path: None,
        command: Some(vec!["sleep".to_string(), "30".to_string()]),
        env: None,
    };

    let deployment = run_container_with_ports(&docker, &req, None)
        .await
        .expect("failed to start container");
    let container_id = deployment.container_id;

    println!("✓ Started container: {}", container_id);

    // Verify it's running
    sleep(Duration::from_millis(200)).await;

    match docker.inspect_container(&container_id, None).await {
        Ok(info) => {
            // Container should be running
            assert!(info.state.as_ref().map_or(false, |s| s.running.unwrap_or(false)),
                "Container should be running");
            println!("✓ Container is running");
        }
        Err(e) => panic!("Failed to inspect container: {}", e),
    }

    // Stop the container
    stop_container(&docker, &container_id)
        .await
        .expect("failed to stop container");

    println!("✓ Stopped container");

    // Verify it's removed (should fail on inspect)
    match docker.inspect_container(&container_id, None).await {
        Ok(_) => panic!("Container should have been removed!"),
        Err(_) => println!("✓ Container successfully removed"),
    }
}
