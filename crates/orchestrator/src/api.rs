use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use crate::db::{
    entitlements_repo, jobs_repo, networks_repo, node_repo, settlements_repo,
    schema::{JobRow, NetworkRow, NodeRow, SettlementRow, UserEntitlementRow}, DbPool
};
use reqwest::Client;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};

use crate::model::{
    ChunkStatusUpdate, CreateNetworkRequest, CreateNetworkResponse, HeartbeatPayload, JobRecord,
    JobStatus, NetworkRecord, NetworkStatus, NodeRecord, NodeStatus, RegisterNodeRequest,
    RegisterNodeResponse, RunJobRequest, RunJobResponse, StopJobRequest, StopJobResponse,
    SubmitJobRequest, SubmitJobResponse,
};
use crate::state::SharedState;

use crate::solana_client::SolanaClient;

#[derive(Clone)]
pub struct AppState {
    pub state: SharedState,
    pub http: Client,
    pub db: DbPool,
    pub solana: SolanaClient,
    pub managed_network_id: Option<String>,
    pub orchestrator_public_url: Option<String>,
}

fn managed_network_mismatch(
    app: &AppState,
    network_id: &str,
) -> Option<(StatusCode, Json<serde_json::Value>)> {
    if let Some(managed) = &app.managed_network_id {
        if network_id != managed {
            return Some((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "error": format!(
                        "this orchestrator manages only network '{}' (requested '{}')",
                        managed, network_id
                    )
                })),
            ));
        }
    }

    None
}

pub fn build_router(app: AppState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/networks/create", post(create_network_handler))
        .route("/networks", get(list_networks_handler))
        .route("/nodes/register", post(register_node_handler))
        .route("/nodes", get(list_nodes_handler))
        .route("/jobs/submit", post(submit_job_handler))
        .route("/jobs/:job_id/stop", post(stop_job_handler))
        .route("/jobs", get(list_jobs_handler))
    .route("/users/:wallet/entitlements", get(list_user_entitlements_handler))
    .route("/users/:wallet/settlements", get(list_user_settlements_handler))
        .route("/agent/heartbeat", post(agent_heartbeat_handler))
        .route("/agent/chunk-status", post(agent_chunk_status_handler))
        .with_state(app)
}

async fn stop_job_handler(
    Path(job_id): Path<String>,
    State(app): State<AppState>,
    Json(payload): Json<StopJobRequest>,
) -> Result<Json<StopJobResponse>, (StatusCode, Json<serde_json::Value>)> {
    let (job_snapshot, node_snapshot, chunk_id) = {
        let guard = app.state.read().await;
        let job = guard.jobs.get(&job_id).cloned().ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": format!("job '{}' not found", job_id)})),
            )
        })?;

        let node = job
            .assigned_node_id
            .as_ref()
            .and_then(|id| guard.nodes.get(id))
            .cloned();
        let chunk_id = guard.job_chunk_ids.get(&job_id).cloned();

        (job, node, chunk_id)
    };

    if matches!(
        job_snapshot.status,
        JobStatus::Done | JobStatus::Failed | JobStatus::Stopped | JobStatus::Preempted
    ) {
        return Ok(Json(StopJobResponse {
            stopped: false,
            job_id,
            status: job_snapshot.status,
            message: "job is already in terminal state".to_string(),
        }));
    }

    if let (Some(node), Some(chunk)) = (node_snapshot.clone(), chunk_id.clone()) {
        let stop_url = format!("{}/stop", node.agent_url.trim_end_matches('/'));
        let agent_resp = app
            .http
            .post(stop_url)
            .json(&serde_json::json!({
                "chunk_id": chunk,
                "reason": payload.reason.clone(),
            }))
            .send()
            .await;

        match agent_resp {
            Ok(resp) if resp.status().is_success() => {}
            Ok(resp) => {
                return Err((
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({"error": format!("agent stop failed with status {}", resp.status())})),
                ));
            }
            Err(err) => {
                return Err((
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({"error": format!("agent stop request failed: {}", err)})),
                ));
            }
        }
    }

    let stop_message = payload
        .reason
        .clone()
        .unwrap_or_else(|| "stopped by user".to_string());

    let (job_state, node_state) = {
        let mut guard = app.state.write().await;

        if let Some(job_mut) = guard.jobs.get_mut(&job_id) {
            job_mut.status = JobStatus::Stopped;
            job_mut.error_detail = Some(stop_message.clone());
        }

        let node_state = if let Some(node_id) = job_snapshot.assigned_node_id.as_ref() {
            if let Some(node_mut) = guard.nodes.get_mut(node_id) {
                node_mut.is_idle = true;
                node_mut.status = NodeStatus::Idle;
            }
            guard.nodes.get(node_id).cloned()
        } else {
            None
        };

        guard.job_exposed_ports.remove(&job_id);
        guard.job_deploy_urls.remove(&job_id);
        guard.job_chunk_ids.remove(&job_id);

        (guard.jobs.get(&job_id).cloned(), node_state)
    };

    if let Some(job) = &job_state {
        let _ = persist_job_state(&app, job).await;
    }
    if let Some(node) = &node_state {
        let _ = node_repo::register_node(&app.db, &node_record_to_row(node)).await;
    }

    Ok(Json(StopJobResponse {
        stopped: true,
        job_id,
        status: JobStatus::Stopped,
        message: stop_message,
    }))
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

pub async fn run_status_maintenance_loop(app: AppState, interval_secs: u64, heartbeat_ttl_secs: u64) {
    loop {
        let now = now_epoch_secs();
        let mut stale_node_ids = Vec::new();
        let mut network_status_snapshots: Vec<(String, NetworkStatus)> = Vec::new();

        {
            let mut guard = app.state.write().await;

            let mut network_has_active_node: HashMap<String, bool> = HashMap::new();

            for node in guard.nodes.values() {
                let stale = now.saturating_sub(node.last_seen_epoch_secs) > heartbeat_ttl_secs;
                if stale {
                    stale_node_ids.push(node.node_id.clone());
                    continue;
                }

                let is_active = matches!(node.status, NodeStatus::Idle | NodeStatus::Busy | NodeStatus::Draining | NodeStatus::Preempting)
                    && !stale;

                if is_active {
                    network_has_active_node.insert(node.network_id.clone(), true);
                }
            }

            for network in guard.networks.values_mut() {
                let active = network_has_active_node
                    .get(&network.network_id)
                    .copied()
                    .unwrap_or(false);

                network.status = if active {
                    NetworkStatus::Active
                } else {
                    NetworkStatus::Inactive
                };

                network_status_snapshots.push((network.network_id.clone(), network.status.clone()));
            }

            for node_id in &stale_node_ids {
                guard.nodes.remove(node_id);
            }
        }

        for node_id in stale_node_ids {
            let _ = node_repo::delete_node(&app.db, &node_id).await;
        }

        for (network_id, status) in network_status_snapshots {
            let _ = networks_repo::set_network_status(
                &app.db,
                &network_id,
                network_status_to_string(&status),
            )
            .await;
        }

        sleep(Duration::from_secs(interval_secs)).await;
    }
}

pub async fn hydrate_runtime_state_from_db(app: &AppState) {
    let networks = networks_repo::list_networks(&app.db).await.unwrap_or_default();
    let nodes = node_repo::list_all_nodes(&app.db).await.unwrap_or_default();
    let jobs = jobs_repo::list_all_jobs(&app.db).await.unwrap_or_default();

    let mut guard = app.state.write().await;
    guard.networks.clear();
    guard.nodes.clear();
    guard.jobs.clear();

    for network in networks {
        let record = network_row_to_record(network);
        if let Some(managed) = &app.managed_network_id {
            if record.network_id != *managed {
                continue;
            }
        }
        guard.networks.insert(record.network_id.clone(), record);
    }

    for node in nodes {
        let record = node_row_to_record(node);
        if let Some(managed) = &app.managed_network_id {
            if record.network_id != *managed {
                continue;
            }
        }
        guard.nodes.insert(record.node_id.clone(), record);
    }

    for job in jobs {
        let record = job_row_to_record(job);
        if let Some(managed) = &app.managed_network_id {
            if record.network_id != *managed {
                continue;
            }
        }
        guard.jobs.insert(record.job_id.clone(), record);
    }
}

async fn health_handler() -> StatusCode {
    StatusCode::OK
}

async fn create_network_handler(
    State(app): State<AppState>,
    Json(payload): Json<CreateNetworkRequest>,
) -> Result<Json<CreateNetworkResponse>, (StatusCode, Json<serde_json::Value>)> {
    if payload.network_id.trim().is_empty() || payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "network_id and name are required"})),
        ));
    }

    if let Some(err) = managed_network_mismatch(&app, &payload.network_id) {
        return Err(err);
    }

    let now = now_epoch_secs() as i64;
    let record = NetworkRecord {
        network_id: payload.network_id.clone(),
        name: payload.name.clone(),
        description: payload.description.clone(),
        orchestrator_url: app.orchestrator_public_url.clone(),
        status: NetworkStatus::Active,
        created_at_epoch_secs: now as u64,
    };

    let chain_message = match app
        .solana
        .register_network_on_chain(&payload.network_id, &payload.name)
        .await
    {
        Ok(tx_hash) => format!("network created on-chain: {tx_hash}"),
        Err(err) => {
            eprintln!(
                "[WARN] register_network_on_chain failed for network '{}': {}",
                payload.network_id, err
            );
            format!(
                "network saved in db; on-chain registration failed: {}",
                err
            )
        }
    };

    networks_repo::create_network(&app.db, &network_record_to_row(&record))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("db create network failed: {e}")})),
            )
        })?;

    let mut guard = app.state.write().await;
    guard.networks.insert(record.network_id.clone(), record);

    Ok(Json(CreateNetworkResponse {
        created: true,
        message: chain_message,
    }))
}

async fn list_networks_handler(State(app): State<AppState>) -> Json<Vec<NetworkRecord>> {
    match networks_repo::list_networks(&app.db).await {
        Ok(rows) => {
            let mut networks = rows.into_iter().map(network_row_to_record).collect::<Vec<_>>();
            if let Some(managed) = &app.managed_network_id {
                networks.retain(|n| n.network_id == *managed);
            }
            Json(networks)
        }
        Err(_) => {
            let guard = app.state.read().await;
            let mut networks = guard.networks.values().cloned().collect::<Vec<_>>();
            if let Some(managed) = &app.managed_network_id {
                networks.retain(|n| n.network_id == *managed);
            }
            Json(networks)
        }
    }
}

async fn register_node_handler(
    State(app): State<AppState>,
    Json(payload): Json<RegisterNodeRequest>,
) -> Result<Json<RegisterNodeResponse>, (StatusCode, Json<serde_json::Value>)> {
    if payload.node_id.trim().is_empty()
        || payload.agent_url.trim().is_empty()
        || payload.network_id.trim().is_empty()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "node_id, network_id and agent_url are required"})),
        ));
    }

    if let Some(err) = managed_network_mismatch(&app, &payload.network_id) {
        return Err(err);
    }

    let network_exists = networks_repo::get_network(&app.db, &payload.network_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("db read network failed: {e}")})),
            )
        })?
        .is_some();

    if !network_exists {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "network does not exist"})),
        ));
    }

    // Register provider on-chain if wallet is provided
    let mut chain_message = "node registered".to_string();
    if let Some(provider_wallet) = &payload.provider_wallet {
        let provider_tx = app.solana
            .register_provider_on_chain(&payload.network_id, &payload.node_id, provider_wallet)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("solana register provider failed: {e}")})),
                )
            })?;
        chain_message = format!("node registered on-chain: {}", provider_tx);
    }

    let now = now_epoch_secs() as i64;
    let mut guard = app.state.write().await;

    let node = NodeRecord {
        node_id: payload.node_id.clone(),
        network_id: payload.network_id,
        agent_url: payload.agent_url,
        provider_wallet: payload.provider_wallet,
        region: payload.region,
        labels: payload.labels.unwrap_or_default(),
        status: NodeStatus::Idle,
        is_idle: true,
        cpu_available_pct: 0.0,
        ram_available_mb: 0,
        disk_available_gb: 0,
        running_chunks: 0,
        last_seen_epoch_secs: now as u64,
    };

    guard.nodes.insert(payload.node_id, node.clone());

    node_repo::register_node(&app.db, &node_record_to_row(&node)).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("db register node failed: {e}")})),
        )
    })?;

    Ok(Json(RegisterNodeResponse {
        registered: true,
        message: chain_message,
    }))
}

async fn list_nodes_handler(State(app): State<AppState>) -> Json<Vec<NodeRecord>> {
    match node_repo::list_all_nodes(&app.db).await {
        Ok(rows) => {
            let mut nodes = rows.into_iter().map(node_row_to_record).collect::<Vec<_>>();
            if let Some(managed) = &app.managed_network_id {
                nodes.retain(|n| n.network_id == *managed);
            }
            Json(nodes)
        }
        Err(_) => {
            let guard = app.state.read().await;
            let mut nodes = guard.nodes.values().cloned().collect::<Vec<_>>();
            if let Some(managed) = &app.managed_network_id {
                nodes.retain(|n| n.network_id == *managed);
            }
            Json(nodes)
        }
    }
}

async fn submit_job_handler(
    State(app): State<AppState>,
    Json(payload): Json<SubmitJobRequest>,
) -> Result<Json<SubmitJobResponse>, (StatusCode, Json<serde_json::Value>)> {
    if payload.network_id.trim().is_empty() || payload.image.trim().is_empty() || payload.user_wallet.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "network_id, image, and user_wallet are required"})),
        ));
    }

    if let Some(err) = managed_network_mismatch(&app, &payload.network_id) {
        return Err(err);
    }

    let network_exists = networks_repo::get_network(&app.db, &payload.network_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("db read network failed: {e}")})),
            )
        })?
        .is_some();

    if !network_exists {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "network does not exist"})),
        ));
    }

    // Check user quota in network (can be disabled for local testing)
    let disable_billing = std::env::var("DISABLE_BILLING_CHECK").is_ok();
    if !disable_billing {
        let estimated_units = (payload.cpu_limit * 10000.0) as i64 + (payload.ram_limit_mb as i64 / 100);
        let has_quota = entitlements_repo::check_quota(&app.db, &payload.user_wallet, &payload.network_id, estimated_units)
            .await
            .unwrap_or(false);

        if !has_quota {
            return Err((
                StatusCode::PAYMENT_REQUIRED,
                Json(serde_json::json!({"error": "insufficient compute credits in network"})),
            ));
        }
    }

    let (job_id, inserted_job) = {
        let mut guard = app.state.write().await;
        guard.next_job_seq += 1;
        let job_id = format!("job-{}-{}", now_epoch_millis(), guard.next_job_seq);

        let job = JobRecord {
            job_id: job_id.clone(),
            network_id: payload.network_id.clone(),
            user_wallet: Some(payload.user_wallet.clone()),
            image: payload.image.clone(),
            command: payload.command.clone(),
            cpu_limit: payload.cpu_limit,
            ram_limit_mb: payload.ram_limit_mb,
            exposed_port: payload.exposed_port,
            status: JobStatus::Pending,
            assigned_node_id: None,
            created_at_epoch_secs: now_epoch_secs(),
            error_detail: None,
            deploy_url: None,
        };

        guard.jobs.insert(job_id.clone(), job);
        if let Some(port) = payload.exposed_port {
            guard.job_exposed_ports.insert(job_id.clone(), port);
        }

        (job_id.clone(), guard.jobs.get(&job_id).cloned().expect("job inserted"))
    };

    jobs_repo::create_job(&app.db, &job_record_to_row(&inserted_job))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("db create job failed: {e}")})),
            )
        })?;

    // Try immediate dispatch once; scheduler loop will retry pending jobs later.
    let _ = dispatch_job_to_idle_node(&app, &job_id).await;

    let (status, assigned_node_id, deploy_url, message, job_snapshot_for_persist) = {
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

        (
            job.status.clone(),
            job.assigned_node_id.clone(),
            guard.job_deploy_urls.get(&job_id).cloned(),
            message,
            job.clone(),
        )
    };

    let _ = persist_job_state(&app, &job_snapshot_for_persist).await;

    Ok(Json(SubmitJobResponse {
        accepted: true,
        job_id,
        status,
        assigned_node_id,
        deploy_url,
        message,
    }))
}

async fn list_jobs_handler(State(app): State<AppState>) -> Json<Vec<JobRecord>> {
    match jobs_repo::list_all_jobs(&app.db).await {
        Ok(rows) => {
            let mut jobs = rows.into_iter().map(job_row_to_record).collect::<Vec<_>>();
            if let Some(managed) = &app.managed_network_id {
                jobs.retain(|j| j.network_id == *managed);
            }
            Json(jobs)
        }
        Err(_) => {
            let guard = app.state.read().await;
            let mut jobs = guard.jobs.values().cloned().collect::<Vec<_>>();
            if let Some(managed) = &app.managed_network_id {
                jobs.retain(|j| j.network_id == *managed);
            }
            Json(jobs)
        }
    }
}

async fn list_user_entitlements_handler(
    Path(wallet): Path<String>,
    State(app): State<AppState>,
) -> Result<Json<Vec<UserEntitlementRow>>, (StatusCode, Json<serde_json::Value>)> {
    let rows = entitlements_repo::list_user_entitlements(&app.db, &wallet)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("db list user entitlements failed: {e}")})),
            )
        })?;

    Ok(Json(rows))
}

async fn list_user_settlements_handler(
    Path(wallet): Path<String>,
    State(app): State<AppState>,
) -> Result<Json<Vec<SettlementRow>>, (StatusCode, Json<serde_json::Value>)> {
    let rows = settlements_repo::list_user_settlements(&app.db, &wallet)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("db list user settlements failed: {e}")})),
            )
        })?;

    Ok(Json(rows))
}

async fn agent_heartbeat_handler(
    State(app): State<AppState>,
    Json(payload): Json<HeartbeatPayload>,
) -> StatusCode {
    let mut guard = app.state.write().await;

    // Use network_id from payload, or fall back to managed network, or default
    let network_id = if !payload.network_id.is_empty() {
        payload.network_id.clone()
    } else {
        app.managed_network_id
            .clone()
            .unwrap_or_else(|| "default".to_string())
    };

    let entry = guard.nodes.entry(payload.node_id.clone()).or_insert(NodeRecord {
        node_id: payload.node_id.clone(),
        network_id: network_id.clone(),
        agent_url: "unknown".to_string(),
        provider_wallet: None,
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

    entry.network_id = network_id;
    entry.status = payload.status;
    entry.is_idle = payload.is_idle;
    entry.cpu_available_pct = payload.cpu_available_pct;
    entry.ram_available_mb = payload.ram_available_mb;
    entry.disk_available_gb = payload.disk_available_gb;
    entry.running_chunks = payload.running_chunks;
    entry.last_seen_epoch_secs = now_epoch_secs();

    let node_snapshot = entry.clone();
    drop(guard);

    let _ = node_repo::register_node(&app.db, &node_record_to_row(&node_snapshot)).await;

    StatusCode::OK
}

async fn agent_chunk_status_handler(
    State(app): State<AppState>,
    Json(payload): Json<ChunkStatusUpdate>,
) -> StatusCode {
    let updated_job = {
        let mut guard = app.state.write().await;

        if let Some(job) = guard.jobs.get_mut(&payload.job_id) {
            job.status = payload.status.clone();
            job.error_detail = payload.detail.clone();
        }

        if matches!(
            payload.status,
            JobStatus::Done | JobStatus::Failed | JobStatus::Preempted | JobStatus::Stopped
        ) {
            guard.job_exposed_ports.remove(&payload.job_id);
            // Keep deploy_urls so the discovered public link persists in the job record.
            guard.job_chunk_ids.remove(&payload.job_id);
        }

        guard.jobs.get(&payload.job_id).cloned()
    };

    if let Some(job) = updated_job {
        let _ = jobs_repo::update_job(&app.db, &job_record_to_row(&job)).await;
    }

    StatusCode::OK
}

async fn dispatch_job_to_idle_node(app: &AppState, job_id: &str) -> Result<(), ()> {
    let candidate_node_ids = {
        let guard = app.state.read().await;
        let job = match guard.jobs.get(job_id) {
            Some(j) => j,
            None => return Err(()),
        };

        if !matches!(job.status, JobStatus::Pending) {
            return Ok(());
        }

        guard
            .nodes
            .values()
            .filter(|n| {
                n.network_id == job.network_id
                    && n.is_idle
                    && matches!(n.status, NodeStatus::Idle)
                    && (n.agent_url.starts_with("http://") || n.agent_url.starts_with("https://"))
            })
            .map(|n| n.node_id.clone())
            .collect::<Vec<_>>()
    };

    if candidate_node_ids.is_empty() {
        return Ok(());
    }

    for candidate_node_id in candidate_node_ids {
        let dispatch_ctx = {
            let mut guard = app.state.write().await;

            let job = match guard.jobs.get(job_id).cloned() {
                Some(j) => j,
                None => return Err(()),
            };

            if !matches!(job.status, JobStatus::Pending) {
                return Ok(());
            }

            let selected_node = match guard.nodes.get(&candidate_node_id).cloned() {
                Some(n)
                    if n.network_id == job.network_id
                        && n.is_idle
                        && matches!(n.status, NodeStatus::Idle)
                        && (n.agent_url.starts_with("http://")
                            || n.agent_url.starts_with("https://")) =>
                {
                    n
                }
                _ => continue,
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

            let exposed_port = guard.job_exposed_ports.get(job_id).copied();

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
                exposed_port,
            };

            let selected_node_snapshot = guard.nodes.get(&selected_node.node_id).cloned();
            let scheduled_job_snapshot = guard.jobs.get(job_id).cloned();
            let user_wallet = job.user_wallet.clone();

            (
                selected_node,
                run_payload,
                selected_node_snapshot,
                scheduled_job_snapshot,
                user_wallet,
            )
        };

        let (node, run_payload, selected_node_snapshot, scheduled_job_snapshot, user_wallet) =
            dispatch_ctx;

        // Open escrow on-chain before dispatching to agent.
        if let Some(_provider_wallet) = &node.provider_wallet {
            let estimated_units = (node.cpu_available_pct as u64 * 100) + (node.ram_available_mb / 10);
            let escrow_wallet = user_wallet.unwrap_or_else(|| run_payload.job_id.clone());
            let _ = app.solana
                .open_escrow_on_chain(
                    job_id,
                    &node.network_id,
                    &node.node_id,
                    estimated_units,
                    estimated_units * 100, // rough token estimate
                    &escrow_wallet,
                )
                .await;
        }

        if let Some(node_state) = selected_node_snapshot {
            let _ = node_repo::register_node(&app.db, &node_record_to_row(&node_state)).await;
        }
        if let Some(job_state) = scheduled_job_snapshot {
            let _ = persist_job_state(app, &job_state).await;
        }

        let run_url = format!("{}/run", node.agent_url.trim_end_matches('/'));
        let send_result = app.http.post(run_url).json(&run_payload).send().await;

        let dispatch_succeeded = match send_result {
            Ok(resp) if resp.status().is_success() => match resp.json::<RunJobResponse>().await {
                Ok(agent_resp) => {
                    let mut guard = app.state.write().await;
                    if let Some(job_mut) = guard.jobs.get_mut(job_id) {
                        job_mut.status = agent_resp.status.clone();
                        if let Some(url) = agent_resp.deploy_url.clone() {
                            job_mut.deploy_url = Some(url.clone());
                        }
                    }
                    guard
                        .job_chunk_ids
                        .insert(job_id.to_string(), run_payload.chunk_id.clone());
                    if let Some(url) = agent_resp.deploy_url {
                        guard.job_deploy_urls.insert(job_id.to_string(), url);
                    }

                    let job_snapshot = guard.jobs.get(job_id).cloned();
                    drop(guard);

                    if let Some(job_state) = job_snapshot {
                        let _ = persist_job_state(app, &job_state).await;
                    }
                    true
                }
                Err(_) => false,
            },
            _ => false,
        };

        if dispatch_succeeded {
            return Ok(());
        }

        // Dispatch failed on this node: make it available and continue trying other idle nodes.
        let mut guard = app.state.write().await;

        if let Some(node_mut) = guard.nodes.get_mut(&node.node_id) {
            node_mut.is_idle = true;
            node_mut.status = NodeStatus::Idle;
        }

        if let Some(job_mut) = guard.jobs.get_mut(job_id) {
            job_mut.status = JobStatus::Pending;
            job_mut.assigned_node_id = None;
        }
        guard.job_chunk_ids.remove(job_id);

        let node_snapshot = guard.nodes.get(&node.node_id).cloned();
        let job_snapshot = guard.jobs.get(job_id).cloned();
        drop(guard);

        if let Some(node_state) = node_snapshot {
            let _ = node_repo::register_node(&app.db, &node_record_to_row(&node_state)).await;
        }

        if let Some(job_state) = job_snapshot {
            let _ = persist_job_state(app, &job_state).await;
        }
    }

    Ok(())
}

async fn persist_job_state(app: &AppState, job: &JobRecord) -> Result<(), ()> {
    jobs_repo::update_job(&app.db, &job_record_to_row(job))
        .await
        .map_err(|_| ())
}

fn node_record_to_row(node: &NodeRecord) -> NodeRow {
    NodeRow {
        node_id: node.node_id.clone(),
        network_id: node.network_id.clone(),
        agent_url: node.agent_url.clone(),
        provider_wallet: node.provider_wallet.clone(),
        region: node.region.clone(),
        labels: serde_json::to_string(&node.labels).unwrap_or_else(|_| "{}".to_string()),
        status: node_status_to_string(&node.status).to_string(),
        is_idle: node.is_idle,
        cpu_available_pct: node.cpu_available_pct,
        ram_available_mb: node.ram_available_mb as i64,
        disk_available_gb: node.disk_available_gb as i64,
        running_chunks: node.running_chunks as i32,
        last_seen_epoch_secs: node.last_seen_epoch_secs as i64,
    }
}

fn node_row_to_record(row: NodeRow) -> NodeRecord {
    NodeRecord {
        node_id: row.node_id,
        network_id: row.network_id,
        agent_url: row.agent_url,
        provider_wallet: row.provider_wallet,
        region: row.region,
        labels: serde_json::from_str(&row.labels).unwrap_or_default(),
        status: node_status_from_string(&row.status),
        is_idle: row.is_idle,
        cpu_available_pct: row.cpu_available_pct,
        ram_available_mb: row.ram_available_mb as u64,
        disk_available_gb: row.disk_available_gb as u64,
        running_chunks: row.running_chunks as usize,
        last_seen_epoch_secs: row.last_seen_epoch_secs as u64,
    }
}

fn job_record_to_row(job: &JobRecord) -> JobRow {
    JobRow {
        job_id: job.job_id.clone(),
        network_id: job.network_id.clone(),
        user_wallet: job.user_wallet.clone(),
        image: job.image.clone(),
        command: job
            .command
            .clone()
            .and_then(|cmd| serde_json::to_string(&cmd).ok()),
        cpu_limit: job.cpu_limit,
        ram_limit_mb: job.ram_limit_mb as i64,
        exposed_port: job.exposed_port.map(|port| port as i64),
        status: job_status_to_string(&job.status).to_string(),
        assigned_node_id: job.assigned_node_id.clone(),
        created_at_epoch_secs: job.created_at_epoch_secs as i64,
        error_detail: job.error_detail.clone(),
        deploy_url: job.deploy_url.clone(),
    }
}

fn job_row_to_record(row: JobRow) -> JobRecord {
    JobRecord {
        job_id: row.job_id,
        network_id: row.network_id,
        user_wallet: row.user_wallet,
        image: row.image,
        command: row
            .command
            .and_then(|cmd_json| serde_json::from_str::<Vec<String>>(&cmd_json).ok()),
        cpu_limit: row.cpu_limit,
        ram_limit_mb: row.ram_limit_mb as u64,
        exposed_port: row.exposed_port.map(|port| port as u16),
        status: job_status_from_string(&row.status),
        assigned_node_id: row.assigned_node_id,
        created_at_epoch_secs: row.created_at_epoch_secs as u64,
        error_detail: row.error_detail,
        deploy_url: row.deploy_url,
    }
}

fn network_record_to_row(network: &NetworkRecord) -> NetworkRow {
    NetworkRow {
        network_id: network.network_id.clone(),
        name: network.name.clone(),
        description: network.description.clone(),
        orchestrator_url: network.orchestrator_url.clone(),
        status: network_status_to_string(&network.status).to_string(),
        created_at_epoch_secs: network.created_at_epoch_secs as i64,
    }
}

fn network_row_to_record(row: NetworkRow) -> NetworkRecord {
    NetworkRecord {
        network_id: row.network_id,
        name: row.name,
        description: row.description,
        orchestrator_url: row.orchestrator_url,
        status: network_status_from_string(&row.status),
        created_at_epoch_secs: row.created_at_epoch_secs as u64,
    }
}

fn node_status_to_string(status: &NodeStatus) -> &'static str {
    match status {
        NodeStatus::Idle => "Idle",
        NodeStatus::Busy => "Busy",
        NodeStatus::Draining => "Draining",
        NodeStatus::Preempting => "Preempting",
        NodeStatus::Offline => "Offline",
    }
}

fn node_status_from_string(status: &str) -> NodeStatus {
    match status {
        "Busy" => NodeStatus::Busy,
        "Draining" => NodeStatus::Draining,
        "Preempting" => NodeStatus::Preempting,
        "Offline" => NodeStatus::Offline,
        _ => NodeStatus::Idle,
    }
}

fn network_status_to_string(status: &NetworkStatus) -> &'static str {
    match status {
        NetworkStatus::Active => "Active",
        NetworkStatus::Inactive => "Inactive",
        NetworkStatus::Removed => "Removed",
    }
}

fn network_status_from_string(status: &str) -> NetworkStatus {
    match status {
        "Inactive" => NetworkStatus::Inactive,
        "Removed" => NetworkStatus::Removed,
        _ => NetworkStatus::Active,
    }
}

fn job_status_to_string(status: &JobStatus) -> &'static str {
    match status {
        JobStatus::Pending => "Pending",
        JobStatus::Scheduled => "Scheduled",
        JobStatus::Running => "Running",
        JobStatus::Done => "Done",
        JobStatus::Failed => "Failed",
        JobStatus::Preempted => "Preempted",
        JobStatus::Stopped => "Stopped",
    }
}

fn job_status_from_string(status: &str) -> JobStatus {
    match status {
        "Scheduled" => JobStatus::Scheduled,
        "Running" => JobStatus::Running,
        "Done" => JobStatus::Done,
        "Failed" => JobStatus::Failed,
        "Preempted" => JobStatus::Preempted,
        "Stopped" => JobStatus::Stopped,
        _ => JobStatus::Pending,
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