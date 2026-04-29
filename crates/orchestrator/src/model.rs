use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeStatus {
    Idle,
    Busy,
    Draining,
    Preempting,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetworkStatus {
    Active,
    Inactive,
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Scheduled,
    Running,
    Done,
    Failed,
    Preempted,
    Stopped
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterNodeRequest {
    pub node_id: String,
    pub network_id: String,
    pub agent_url: String,
    pub provider_wallet: Option<String>, // Provider's payout wallet address
    pub region: Option<String>,
    pub labels: Option<HashMap<String, String>>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterNodeResponse {
    pub registered: bool, // Whether node registration succeeded.
    pub message: String // Human-readable registration result message.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatPayload {
    pub node_id: String, // Node ID sending this heartbeat.
    pub network_id: String, // Network this node belongs to.
    pub cpu_available_pct: f32, // Currently available CPU percentage on node.
    pub ram_available_mb: u64, // Free RAM on node in MB.
    pub disk_available_gb: u64, // Free disk space on node in GB.
    pub idle_until_epoch_secs: Option<u64>, // Optional epoch timestamp until node expects to stay idle.
    pub running_chunks: usize, // Number of currently running chunks on node.
    pub is_idle: bool, // Quick idle/busy flag used for scheduling decisions.
    pub status: NodeStatus, // Detailed node status state machine value.
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitJobRequest {
    pub network_id: String,
    pub user_wallet: String, // User's wallet address for billing
    pub image: String,
    pub command: Option<Vec<String>>,
    pub cpu_limit: f64,
    pub ram_limit_mb: u64,
    pub exposed_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNetworkRequest {
    pub network_id: String, // Unique network identifier (e.g., clg-a).
    pub name: String, // Human-readable network name.
    pub description: Option<String>, // Optional details about this network.
    pub price_per_unit: Option<u64>, // Price per compute unit (defaults to 100 if not specified).
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNetworkResponse {
    pub created: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRecord {
    pub network_id: String,
    pub name: String,
    pub description: Option<String>,
    pub orchestrator_url: Option<String>,
    pub status: NetworkStatus,
    pub created_at_epoch_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitJobResponse {
    pub accepted: bool, // Whether orchestrator accepted the submitted job.
    pub job_id: String, // Generated job ID assigned by orchestrator.
    pub status: JobStatus, // Initial scheduling/execution status of the job.
    pub assigned_node_id: Option<String>, // Node chosen for the job, if already assigned.
    pub deploy_url: Option<String>, // Public deployment URL (for web services), when available.
    pub message: String // Human-readable scheduling result details.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopJobRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopJobResponse {
    pub stopped: bool,
    pub job_id: String,
    pub status: JobStatus,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunJobRequest {
    pub job_id: String, // Parent job ID this chunk belongs to.
    pub chunk_id: String, // Unique chunk ID to run on the agent.
    pub image: String, // Container image for this chunk.
    pub cpu_limit: f64, // CPU limit to enforce for container.
    pub ram_limit_mb: u64, // RAM limit in MB to enforce for container.
    pub input_path: Option<String>, // Optional input data path/object for worker.
    pub command: Option<Vec<String>>, // Optional command override for execution.
    pub env: Option<Vec<String>>, // Optional environment variables for container.
    pub exposed_port: Option<u16>, // Optional service port exposed by the workload.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunJobResponse {
    pub accepted: bool, // Whether agent accepted chunk launch.
    pub message: String, // Human-readable launch/validation result.
    pub container_id: Option<String>, // Created container ID when launch succeeds.
    pub deploy_url: Option<String>, // Public deployment URL (for web services), when available.
    pub status: JobStatus // Current chunk/job status after request handling.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkStatusUpdate {
    pub node_id: String, // Node sending chunk lifecycle update.
    pub job_id: String, // Parent job ID.
    pub chunk_id: String, // Chunk ID being updated.
    pub status: JobStatus, // New chunk/job status.
    pub detail: Option<String>, // Optional human-readable status detail.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRecord {
    pub node_id: String,
    pub network_id: String,
    pub agent_url: String,
    pub provider_wallet: Option<String>,
    pub region: Option<String>,
    pub labels: HashMap<String, String>,
    pub status: NodeStatus,
    pub is_idle: bool,
    pub cpu_available_pct: f32,
    pub ram_available_mb: u64,
    pub disk_available_gb: u64,
    pub running_chunks: usize,
    pub last_seen_epoch_secs: u64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRecord {
    pub job_id: String,
    pub network_id: String,
    pub user_wallet: Option<String>,
    pub image: String,
    pub command: Option<Vec<String>>,
    pub cpu_limit: f64,
    pub ram_limit_mb: u64,
    pub exposed_port: Option<u16>,
    pub status: JobStatus,
    pub assigned_node_id: Option<String>,
    pub created_at_epoch_secs: u64,
    pub error_detail: Option<String>,
    pub deploy_url: Option<String>,
}