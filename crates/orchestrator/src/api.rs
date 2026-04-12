use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use reqwest::Client;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};

use crate::model::{
    ChunkStatusUpdate, HeartbeatPayload, JobRecord, JobStatus, NodeRecord, NodeStatus,
    RegisterNodeRequest, RegisterNodeResponse, RunJobRequest, RunJobResponse, SubmitJobRequest,
    SubmitJobResponse,
};
use crate::state::SharedState;

#[derive(Clone)]
pub struct AppState {
    pub state: SharedState,
    pub http: Client,
}

pub fn build_router(app: AppState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/nodes/register", post(register_node_handler))
        .route("/nodes", get(list_nodes_handler))
        .route("/jobs/submit", post(submit_job_handler))
        .route("/jobs", get(list_jobs_handler))
        .route("/agent/heartbeat", post(agent_heartbeat_handler))
        .route("/agent/chunk-status", post(agent_chunk_status_handler))
        .with_state(app)
}

pub async fn run_pending_scheduler_loop(app: AppState, interval_secs: u64) {
    loop {
        let pending_job_ids = {
            let guard = app.state.read().await;
            guard
                .jobs
                .values()
                .filter(|job| job.status == JobStatus::Pending)
                .map(|job| job.job_id.clone())
                .collect::<Vec<_>>()
        };

        for job_id in pending_job_ids {
            let _ = dispatch_job_to_idle_node(&app, &job_id).await;
        }

        sleep(Duration::from_secs(interval_secs)).await;
    }
}

async fn health_handler() -> StatusCode {
    StatusCode::OK
}

async fn register_node_handler(
    State(app): State<AppState>,
    Json(payload): Json<RegisterNodeRequest>,
) -> Result<Json<RegisterNodeResponse>, (StatusCode, Json<serde_json::Value>)> {
    if payload.node_id.trim().is_empty() || payload.agent_url.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "node_id and agent_url are required"})),
        ));
    }

    let now = now_epoch_secs();
    let mut guard = app.state.write().await;

    let node = NodeRecord {
        node_id: payload.node_id.clone(),
        agent_url: payload.agent_url,
        region: payload.region,
        labels: payload.labels.unwrap_or_default(),
        status: NodeStatus::Idle,
        is_idle: true,
        cpu_available_pct: 0.0,
        ram_available_mb: 0,
        disk_available_gb: 0,
        running_chunks: 0,
        last_seen_epoch_secs: now,
    };

    guard.nodes.insert(payload.node_id, node);

    Ok(Json(RegisterNodeResponse {
        registered: true,
        message: "node registered".to_string(),
    }))
}

async fn list_nodes_handler(State(app): State<AppState>) -> Json<Vec<NodeRecord>> {
    let guard = app.state.read().await;
    Json(guard.nodes.values().cloned().collect())
}

async fn submit_job_handler(
    State(app): State<AppState>,
    Json(payload): Json<SubmitJobRequest>,
) -> Result<Json<SubmitJobResponse>, (StatusCode, Json<serde_json::Value>)> {
    if payload.image.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "image is required"})),
        ));
    }

    let job_id = {
        let mut guard = app.state.write().await;
        guard.next_job_seq += 1;
        let job_id = format!("job-{}", guard.next_job_seq);

        let job = JobRecord {
            job_id: job_id.clone(),
            image: payload.image.clone(),
            command: payload.command.clone(),
            cpu_limit: payload.cpu_limit,
            ram_limit_mb: payload.ram_limit_mb,
            status: JobStatus::Pending,
            assigned_node_id: None,
            created_at_epoch_secs: now_epoch_secs(),
        };

        guard.jobs.insert(job_id.clone(), job);

        job_id
    };

    // Try immediate dispatch once; scheduler loop will retry pending jobs later.
    let _ = dispatch_job_to_idle_node(&app, &job_id).await;

    let (status, assigned_node_id, message) = {
        let guard = app.state.read().await;
        let job = guard.jobs.get(&job_id).ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "job missing after submit"})),
            )
        })?;

        let message = match job.status {
            JobStatus::Pending => "job accepted, waiting for idle node".to_string(),
            JobStatus::Scheduled | JobStatus::Running => "job dispatched to node".to_string(),
            _ => "job accepted".to_string(),
        };

        (job.status.clone(), job.assigned_node_id.clone(), message)
    };

    Ok(Json(SubmitJobResponse {
        accepted: true,
        job_id,
        status,
        assigned_node_id,
        message,
    }))
}

async fn list_jobs_handler(State(app): State<AppState>) -> Json<Vec<JobRecord>> {
    let guard = app.state.read().await;
    Json(guard.jobs.values().cloned().collect())
}

async fn agent_heartbeat_handler(
    State(app): State<AppState>,
    Json(payload): Json<HeartbeatPayload>,
) -> StatusCode {
    let mut guard = app.state.write().await;

    let entry = guard.nodes.entry(payload.node_id.clone()).or_insert(NodeRecord {
        node_id: payload.node_id.clone(),
        agent_url: "unknown".to_string(),
        region: None,
        labels: HashMap::new(),
        status: payload.status.clone(),
        is_idle: payload.is_idle,
        cpu_available_pct: payload.cpu_available_pct,
        ram_available_mb: payload.ram_available_mb,
        disk_available_gb: payload.disk_available_gb,
        running_chunks: payload.running_chunks,
        last_seen_epoch_secs: now_epoch_secs(),
    });

    entry.status = payload.status;
    entry.is_idle = payload.is_idle;
    entry.cpu_available_pct = payload.cpu_available_pct;
    entry.ram_available_mb = payload.ram_available_mb;
    entry.disk_available_gb = payload.disk_available_gb;
    entry.running_chunks = payload.running_chunks;
    entry.last_seen_epoch_secs = now_epoch_secs();

    StatusCode::OK
}

async fn agent_chunk_status_handler(
    State(app): State<AppState>,
    Json(payload): Json<ChunkStatusUpdate>,
) -> StatusCode {
    let mut guard = app.state.write().await;

    if let Some(job) = guard.jobs.get_mut(&payload.job_id) {
        job.status = payload.status;
    }

    StatusCode::OK
}

async fn dispatch_job_to_idle_node(app: &AppState, job_id: &str) -> Result<(), ()> {
    let (node, run_payload) = {
        let mut guard = app.state.write().await;

        let job = match guard.jobs.get(job_id).cloned() {
            Some(j) => j,
            None => return Err(()),
        };

        if !matches!(job.status, JobStatus::Pending) {
            return Ok(());
        }

        let selected_node = match guard
            .nodes
            .values()
            .find(|n| n.is_idle && (n.agent_url.starts_with("http://") || n.agent_url.starts_with("https://")))
            .cloned()
        {
            Some(n) => n,
            None => return Ok(()),
        };

        // Optimistically reserve node and mark job as scheduled before network call.
        if let Some(node_mut) = guard.nodes.get_mut(&selected_node.node_id) {
            node_mut.is_idle = false;
            node_mut.status = NodeStatus::Busy;
        }

        if let Some(job_mut) = guard.jobs.get_mut(job_id) {
            job_mut.status = JobStatus::Scheduled;
            job_mut.assigned_node_id = Some(selected_node.node_id.clone());
        }

        let run_payload = RunJobRequest {
            job_id: job.job_id,
            // Use a unique chunk suffix per dispatch attempt so retries do not collide
            // with stale container names on the agent.
            chunk_id: format!("{}-chunk-{}", job_id, now_epoch_millis()),
            image: job.image,
            cpu_limit: job.cpu_limit,
            ram_limit_mb: job.ram_limit_mb,
            input_path: None,
            command: job.command,
            env: None,
        };

        (selected_node, run_payload)
    };

    let run_url = format!("{}/run", node.agent_url.trim_end_matches('/'));
    let send_result = app.http.post(run_url).json(&run_payload).send().await;

    match send_result {
        Ok(resp) if resp.status().is_success() => {
            let parsed = resp.json::<RunJobResponse>().await;
            if let Ok(agent_resp) = parsed {
                let mut guard = app.state.write().await;
                if let Some(job_mut) = guard.jobs.get_mut(job_id) {
                    job_mut.status = agent_resp.status;
                }
            }
            Ok(())
        }
        _ => {
            // Dispatch failed: make node available again and reset job to pending for retry.
            let mut guard = app.state.write().await;

            if let Some(node_mut) = guard.nodes.get_mut(&node.node_id) {
                node_mut.is_idle = true;
                node_mut.status = NodeStatus::Idle;
            }

            if let Some(job_mut) = guard.jobs.get_mut(job_id) {
                job_mut.status = JobStatus::Pending;
                job_mut.assigned_node_id = None;
            }
            Err(())
        }
    }
}

fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn now_epoch_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}