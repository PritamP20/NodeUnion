use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Running,
    Done,
    Failed,
    Preempted,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Idle,
    Busy,
    Draining,
    Preempting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunJobRequest {
    pub job_id: String,
    pub chunk_id: String,
    pub image: String,
    pub cpu_limit: f64,
    pub ram_limit_mb: u64,
    pub input_path: Option<String>,
    pub command: Option<Vec<String>>,
    pub env: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunJobResponse {
    pub accepted: bool,
    pub message: String,
    pub container_id: Option<String>,
    pub status: JobStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopJobRequest {
    pub chunk_id: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopJobResponse {
    pub stopped: bool,
    pub message: String,
    pub status: JobStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatPayload {
    pub node_id: String,
    pub cpu_available_pct: f32,
    pub ram_available_mb: u64,
    pub disk_available_gb: u64,
    pub idle_until_epoch_secs: Option<u64>,
    pub running_chunks: usize,
    pub is_idle: bool,
    pub status: NodeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkStatusUpdate {
    pub node_id: String,
    pub job_id: String,
    pub chunk_id: String,
    pub status: JobStatus,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningChunkView {
    pub job_id: String,
    pub chunk_id: String,
    pub container_id: String,
    pub status: JobStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStateResponse {
    pub node_id: String,
    pub status: NodeStatus,
    pub is_idle: bool,
    pub running_chunks: usize,
    pub consecutive_preempt_spikes: usize,
    pub avg_cpu_window_pct: Option<f32>,
    pub cpu_usage_pct: f32,
    pub cpu_available_pct: f32,
    pub ram_total_mb: u64,
    pub ram_available_mb: u64,
    pub disk_available_gb: u64,
    pub active_chunks: Vec<RunningChunkView>,
}