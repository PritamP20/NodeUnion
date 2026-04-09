use serde::{Deserialize, Serialize}
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Idle,
    Busy,
    Draining,
    Preempting
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub node_id: String, // Unique identifier of the node registering with orchestrator.
    pub agent_url: String, // Base URL where this node's agent API is reachable.
    pub region: Option<String>, // Optional deployment region (for locality-aware scheduling).
    pub labels: Option<HashMap<String, String>> // Optional key/value metadata (gpu=true, tier=dev, etc.).
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterNodeResponse {
    pub registered: bool, // Whether node registration succeeded.
    pub message: String // Human-readable registration result message.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatPayload {
    pub node_id: String, // Node ID sending this heartbeat.
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
    pub image: String, // Container image to run for this job.
    pub command: Option<Vec<String>>, // Optional command override for container entrypoint.
    pub cpu_limit: f64, // CPU allocation requested for job/chunk.
    pub ram_limit_mb: u64 // RAM allocation requested in MB.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitJobResponse {
    pub accepted: bool, // Whether orchestrator accepted the submitted job.
    pub job_id: String, // Generated job ID assigned by orchestrator.
    pub status: JobStatus, // Initial scheduling/execution status of the job.
    pub assigned_node_id: Option<String>, // Node chosen for the job, if already assigned.
    pub message: String // Human-readable scheduling result details.
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
    pub env: Option<Vec<String>> // Optional environment variables for container.
}

#[derive(Derive, Clone, Serialize, Deserialize)]
pub struct RunJobResponse {
    pub accepted: bool, // Whether agent accepted chunk launch.
    pub message: String, // Human-readable launch/validation result.
    pub container_id: Option<String>, // Created container ID when launch succeeds.
    pub status: JobStatus // Current chunk/job status after request handling.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRecord {
    pub node_id: String, // Unique node identifier key.
    pub agent_url: String, // Agent base URL used by orchestrator to call /run and /stop.
    pub region: Option<String>, // Optional node region for placement strategy.
    pub labels: HashMap<String, String>, // Normalized node metadata labels.
    pub status: NodeStatus, // Latest reported detailed node status.
    pub is_idle: bool, // Latest reported idle flag.
    pub cpu_available_pct: f32, // Latest available CPU percentage.
    pub ram_available_mb: u64, // Latest available RAM in MB.
    pub disk_available_gb: u64, // Latest available disk in GB.
    pub running_chunks: usize, // Latest count of running chunks.
    pub last_seen_epoch_secs: u64 // Last heartbeat timestamp (epoch seconds).
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRecord {
    pub job_id: String, // Unique orchestrator-assigned job ID.
    pub image: String, // Requested container image for this job.
    pub command: Option<Vec<String>>, // Optional command override for job run.
    pub cpu_limit: f64, // Requested CPU limit for scheduling/run.
    pub ram_limit_md: u64, // Requested RAM limit field (currently named ram_limit_md in this file).
    pub status: JobStatus, // Current lifecycle status of this job.
    pub assigned_node_id: Option<String>, // Node currently assigned to run this job.
    pub created_at_epoch_secs: u64 // Job creation timestamp (epoch seconds).
}