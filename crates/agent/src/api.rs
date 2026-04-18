use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use crate::app_state::SharedAppState;
use crate::container_manager::{run_container, stop_container};
use crate::errors::AppError;
use crate::models::{
    AgentStateResponse, JobStatus, RunJobRequest, RunJobResponse, RunningChunkView, StopJobRequest,
    StopJobResponse,
};
use crate::{config::Config, orchestrator_client::OrchestratorClient};
use bollard::Docker;

#[derive(Clone)]
pub struct AppApiState {
    pub state: SharedAppState,
    pub config: Config,
    pub orchestrator_client: OrchestratorClient,
    pub docker: Docker,
}

pub fn build_router(app_state: AppApiState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
    .route("/state", get(state_handler))
        .route("/run", post(run_handler))
        .route("/stop", post(stop_handler))
        .with_state(app_state)
}


async fn health_handler() -> StatusCode {
    StatusCode::OK
}

async fn state_handler(
    State(app): State<AppApiState>,
) -> Result<Json<AgentStateResponse>, AppError> {
    let guard = app.state.read().await;

    let active_chunks = guard
        .running_chunks
        .values()
        .map(|chunk| RunningChunkView {
            job_id: chunk.job_id.clone(),
            chunk_id: chunk.chunk_id.clone(),
            container_id: chunk.container_id.clone(),
            status: chunk.status.clone(),
        })
        .collect::<Vec<_>>();

    let response = AgentStateResponse {
        node_id: guard.node_id.clone(),
        status: guard.node_status.clone(),
        is_idle: guard.is_idle,
        running_chunks: guard.running_chunks_count(),
        consecutive_preempt_spikes: guard.consecutive_preempt_spikes,
        avg_cpu_window_pct: guard.avg_cpu_window(),
        cpu_usage_pct: guard.metrics.cpu_usage_pct,
        cpu_available_pct: guard.metrics.cpu_available_pct,
        ram_total_mb: guard.metrics.ram_total_mb,
        ram_available_mb: guard.metrics.ram_available_mb,
        disk_available_gb: guard.metrics.disk_available_gb,
        active_chunks,
    };

    Ok(Json(response))
}

async fn run_handler(
    State(app): State<AppApiState>,
    Json(payload): Json<RunJobRequest>,
) -> Result<Json<RunJobResponse>, AppError> {
    if payload.image.trim().is_empty() {
        return Err(AppError::bad_request("image cannot be empty"));
    }

    let container_id = run_container(&app.docker, &payload)
        .await
        .map_err(AppError::from)?;

    {
        let mut guard = app.state.write().await;

        guard.running_chunks.insert(
            payload.chunk_id.clone(),
            crate::app_state::RunningChunk {
                job_id: payload.job_id.clone(),
                chunk_id: payload.chunk_id.clone(),
                container_id: container_id.clone(),
                status: crate::models::JobStatus::Running,
            },
        );
    }

    Ok(Json(RunJobResponse {
        accepted: true,
        message: "container started".to_string(),
        container_id: Some(container_id),
        status: JobStatus::Running,
    }))
}

async fn stop_handler(
    State(app): State<AppApiState>,
    Json(payload): Json<StopJobRequest>,
) -> Result<Json<StopJobResponse>, AppError> {
    let container_id = {
        let guard = app.state.read().await;
        match guard.running_chunks.get(&payload.chunk_id) {
            Some(running) => running.container_id.clone(),
            None => return Err(AppError::not_found("chunk not found")),
        }
    };

    stop_container(&app.docker, &container_id)
        .await
        .map_err(AppError::from)?;

    {
        let mut guard = app.state.write().await;
        guard.running_chunks.remove(&payload.chunk_id);
    }

    Ok(Json(StopJobResponse {
        stopped: true,
        message: "container stopped".to_string(),
        status: JobStatus::Stopped,
    }))
}