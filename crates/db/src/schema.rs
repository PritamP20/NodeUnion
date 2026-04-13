use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NodeRow {
    pub node_id: String,
    pub agent_url: String,
    pub region: Option<String>,
    pub labels: String,
    pub status: String,
    pub is_idle: bool,
    pub cpu_available_pct: f32,
    pub ram_available_mb: i64,
    pub disk_available_gb: i64,
    pub running_chunks: i32,
    pub last_seen_epoch_secs: i64
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct JobRow{
    pub job_id: String,
    pub image: String,
    pub command: Option<String>,
    pub cpu_limit: f64,
    pub ram_limit_mb: i64,
    pub status: String,
    pub assigned_node_id: Option<String>,
    pub created_at_epoch_secs: i64
}
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AttemptRow {
    pub attempt_id: String,
    pub job_id: String,
    pub attempt_number: i32,
    pub assigned_node_id: Option<String>,
    pub last_error: Option<String>,
    pub next_retry_at_epoch_secs: Option<i64>,
    pub created_at_epoch_secs: i64
}