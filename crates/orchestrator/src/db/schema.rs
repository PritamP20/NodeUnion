use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NetworkRow {
    pub network_id: String,
    pub name: String,
    pub description: Option<String>,
    pub orchestrator_url: Option<String>,
    pub status: String,
    pub created_at_epoch_secs: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NodeRow {
    pub node_id: String,
    pub network_id: String,
    pub agent_url: String,
    pub provider_wallet: Option<String>,
    pub region: Option<String>,
    pub labels: String,
    pub status: String,
    pub is_idle: bool,
    pub cpu_available_pct: f32,
    pub ram_available_mb: i64,
    pub disk_available_gb: i64,
    pub running_chunks: i32,
    pub last_seen_epoch_secs: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct JobRow {
    pub job_id: String,
    pub network_id: String,
    pub user_wallet: Option<String>,
    pub image: String,
    pub command: Option<String>,
    pub cpu_limit: f64,
    pub ram_limit_mb: i64,
    pub exposed_port: Option<i64>,
    pub status: String,
    pub assigned_node_id: Option<String>,
    pub created_at_epoch_secs: i64,
    pub error_detail: Option<String>,
    pub deploy_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AttemptRow {
    pub attempt_id: String,
    pub job_id: String,
    pub attempt_number: i32,
    pub assigned_node_id: Option<String>,
    pub last_error: Option<String>,
    pub next_retry_at_epoch_secs: Option<i64>,
    pub created_at_epoch_secs: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserEntitlementRow {
    pub entitlement_id: String,
    pub user_wallet: String,
    pub network_id: String,
    pub bought_units: i64,
    pub used_units: i64,
    pub escrow_account: Option<String>,
    pub escrow_tx_hash: Option<String>,
    pub expiry_epoch_secs: Option<i64>,
    pub created_at_epoch_secs: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SettlementRow {
    pub settlement_id: String,
    pub job_id: String,
    pub user_wallet: String,
    pub provider_wallet: Option<String>,
    pub network_id: String,
    pub units_metered: i64,
    pub amount_tokens: i64,
    pub tx_hash: Option<String>,
    pub tx_status: Option<String>,
    pub settlement_type: Option<String>,
    pub created_at_epoch_secs: i64,
}