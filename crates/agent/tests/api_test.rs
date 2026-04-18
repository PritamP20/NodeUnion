// Integration tests for the HTTP API endpoints.
// These tests start the Axum server and make HTTP requests to it.

// Integration tests for the HTTP API endpoints.
// These tests start a test Axum server and make HTTP requests to verify handlers work.

use nodeunion_agent::api::build_router;
use nodeunion_agent::app_state::{AppState, SharedAppState, NodeMetricsSnapshot};
use nodeunion_agent::config::Config;
use nodeunion_agent::models::{RunJobRequest, StopJobRequest, NodeStatus};
use nodeunion_agent::orchestrator_client::OrchestratorClient;
use axum::http::StatusCode;
use axum::body::Body;
use axum::http::Request;
use std::sync::Arc;
use std::collections::VecDeque;
use tokio::sync::RwLock;
use tower::ServiceExt;

// Helper to create a test AppState and AppApiState for testing without Docker
async fn create_test_app_state() -> (SharedAppState, nodeunion_agent::api::AppApiState) {
    let app_state = Arc::new(RwLock::new(AppState {
        node_id: "test-node-1".to_string(),
        node_status: NodeStatus::Idle,
        is_idle: true,
        consecutive_preempt_spikes: 0,
        idle_until_epoch_secs: None,
        cpu_window: VecDeque::new(),
        metrics: NodeMetricsSnapshot::default(),
        running_chunks: std::collections::HashMap::new(),
    }));

    let config = Config {
        node_id: "test-node-1".to_string(),
        bind_addr: "127.0.0.1:8090".to_string(),
        orchestrator_base_url: "http://127.0.0.1:8080".to_string(),
        heartbeat_interval_secs: 60,
        metrics_poll_interval_secs: 30,
        idle_cpu_threshold_pct: 15.0,
        preempt_cpu_threshold_pct: 60.0,
        idle_window_samples: 10,
        request_timeout_secs: 30,
    };

    let client = OrchestratorClient::new(&config);

    // Docker daemon must be running for integration tests
    let docker = bollard::Docker::connect_with_socket_defaults()
        .unwrap_or_else(|_| panic!("Docker daemon must be running for integration tests"));

    (app_state.clone(), nodeunion_agent::api::AppApiState {
        state: app_state.clone(),
        config,
        orchestrator_client: client,
        docker,
    })
}

#[tokio::test]
async fn health_endpoint_returns_200() {
    let (_state, app_api_state) = create_test_app_state().await;
    let router = build_router(app_api_state);

    let request = Request::builder().uri("/health").body(Body::empty()).unwrap();
    let response = router.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn run_endpoint_rejects_empty_image() {
    let (_state, app_api_state) = create_test_app_state().await;
    let router = build_router(app_api_state);

    let payload = RunJobRequest {
        job_id: "job-123".to_string(),
        chunk_id: "chunk-456".to_string(),
        image: "".to_string(), // Empty image should fail validation
        cpu_limit: 1.0,
        ram_limit_mb: 512,
        input_path: None,
        command: None,
        env: None,
    };

    let request = Request::builder()
        .method("POST")
        .uri("/run")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn stop_endpoint_returns_404_for_unknown_chunk() {
    let (_state, app_api_state) = create_test_app_state().await;
    let router = build_router(app_api_state);

    let payload = StopJobRequest {
        chunk_id: "nonexistent-chunk".to_string(),
        reason: None,
    };

    let request = Request::builder()
        .method("POST")
        .uri("/stop")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires Docker daemon and a real container to stop"]
async fn stop_endpoint_removes_chunk_from_state() {
    let (state, app_api_state) = create_test_app_state().await;
    let router = build_router(app_api_state);

    // Start a real container through /run so /stop operates on an actual container id.
    let run_payload = RunJobRequest {
        job_id: "job-123".to_string(),
        chunk_id: "chunk-456".to_string(),
        image: "alpine:latest".to_string(),
        cpu_limit: 0.25,
        ram_limit_mb: 128,
        input_path: None,
        command: Some(vec!["sh".to_string(), "-c".to_string(), "sleep 30".to_string()]),
        env: None,
    };

    let run_request = Request::builder()
        .method("POST")
        .uri("/run")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&run_payload).unwrap()))
        .unwrap();

    let run_response = router.clone().oneshot(run_request).await.unwrap();
    assert_eq!(run_response.status(), StatusCode::OK);

    // Verify chunk is tracked in state before stop.
    {
        let guard = state.read().await;
        assert!(guard.running_chunks.contains_key("chunk-456"));
    }

    let payload = StopJobRequest {
        chunk_id: "chunk-456".to_string(),
        reason: Some("test stop".to_string()),
    };

    let request = Request::builder()
        .method("POST")
        .uri("/stop")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify chunk was removed from state
    {
        let guard = state.read().await;
        assert!(!guard.running_chunks.contains_key("chunk-456"));
    }
}
