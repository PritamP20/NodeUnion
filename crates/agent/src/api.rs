use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use crate::app_state::SharedAppState;
use crate::container_manager::{run_container, stop_container};
use crate::errors::AppError;
use crate::models::{JobStatus, RunJobRequest, RunJobResponse, StopJobRequest, StopJobResponse};
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
        .route("/run", post(run_handler))
        .route("/stop", post(stop_handler))
        .with_state(app_state)
}


async fn health_handler() -> StatusCode {
    StatusCode::OK
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