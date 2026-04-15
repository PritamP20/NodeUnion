use crate::model::{JobRecord, NetworkRecord, NodeRecord};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Default)]
pub struct OrchestratorState {
    pub networks: HashMap<String, NetworkRecord>,
    pub nodes: HashMap<String, NodeRecord>,
    pub jobs: HashMap<String, JobRecord>,
    pub next_job_seq: u64
}

pub type SharedState = Arc<RwLock<OrchestratorState>>;