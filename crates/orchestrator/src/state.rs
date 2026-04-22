use crate::model::{JobRecord, NetworkRecord, NodeRecord};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Default)]
pub struct OrchestratorState {
    pub networks: HashMap<String, NetworkRecord>,
    pub nodes: HashMap<String, NodeRecord>,
    pub jobs: HashMap<String, JobRecord>,
    pub next_job_seq: u64,
    pub job_exposed_ports: HashMap<String, u16>,
    pub job_deploy_urls: HashMap<String, String>,
    pub job_chunk_ids: HashMap<String, String>,
}

pub type SharedState = Arc<RwLock<OrchestratorState>>;