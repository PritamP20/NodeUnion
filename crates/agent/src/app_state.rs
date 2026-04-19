use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::models::{JobStatus, NodeStatus};

#[derive(Debug, Clone)]
pub struct RunningChunk {
    pub job_id:String,
    pub chunk_id: String,
    pub container_id: String,
    pub status: JobStatus
}

#[derive(Debug, Clone)]
pub struct NodeMetricsSnapshot {
    pub cpu_usage_pct: f32,
    pub cpu_available_pct: f32,
    pub ram_total_mb: u64,
    pub ram_available_mb: u64,
    pub disk_available_gb: u64,
}

impl Default for NodeMetricsSnapshot {
    fn default() -> Self {
        Self {
            cpu_usage_pct: 0.0,
            cpu_available_pct: 100.0,
            ram_total_mb: 0,
            ram_available_mb: 0,
            disk_available_gb: 0,
        }
    }
}

#[derive(Debug)]
pub struct AppState {
    pub node_id: String,
    pub public_url: Option<String>,
    pub node_status: NodeStatus, // Idle, Busy, Draining, Preempting.
    pub is_idle: bool, // Fast boolean flag used in scheduling filters and heartbeat.
    pub consecutive_preempt_spikes: usize, // Counts consecutive high-CPU samples to decide preemption trigger.
    pub idle_until_epoch_secs: Option<u64>, // Optional estimated time until machine is expected to stay idle.
    pub cpu_window: VecDeque<f32>, // Sliding window of recent CPU usage samples.
    pub metrics: NodeMetricsSnapshot, // Latest resource snapshot.
    pub running_chunks: HashMap<String, RunningChunk>,  // Active chunk map keyed by chunk_id.
}

impl AppState {
    pub fn new(node_id: String, idle_window_capacity: usize) -> Self {
        Self {
            node_id,
            public_url: None,
            node_status: NodeStatus::Idle,
            is_idle: true,
            consecutive_preempt_spikes: 0,
            idle_until_epoch_secs: None,
            cpu_window: VecDeque::with_capacity(idle_window_capacity),
            metrics: NodeMetricsSnapshot::default(),
            running_chunks: HashMap::new(),
        }
    }

    pub fn push_cpu_sample(&mut self, sample: f32, max_samples: usize) {
        if self.cpu_window.len() == max_samples {
            self.cpu_window.pop_front();
        }
        self.cpu_window.push_back(sample);
    }

    pub fn avg_cpu_window(&self) -> Option<f32> {
        if self.cpu_window.is_empty() {
            return None;
        }

        let sum: f32 = self.cpu_window.iter().copied().sum();
        Some(sum / self.cpu_window.len() as f32)
    }

    // Convenience helper for heartbeat/API logic.
    pub fn running_chunks_count(&self) -> usize {
        self.running_chunks.len()
    }
}
pub type SharedAppState = Arc<RwLock<AppState>>; // Type alias used across modules for clean function signatures.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_window_is_bounded_and_average_is_correct() {
        let mut state = AppState::new("node-1".to_string(), 3);

        state.push_cpu_sample(10.0, 3);
        state.push_cpu_sample(20.0, 3);
        state.push_cpu_sample(30.0, 3);
        state.push_cpu_sample(40.0, 3);

        assert_eq!(state.cpu_window.len(), 3);
        assert_eq!(state.cpu_window.front().copied(), Some(20.0));
        assert_eq!(state.cpu_window.back().copied(), Some(40.0));
        assert_eq!(state.avg_cpu_window(), Some(30.0));
    }

    #[test]
    fn running_chunks_count_tracks_insertions() {
        let mut state = AppState::new("node-2".to_string(), 10);
        assert_eq!(state.running_chunks_count(), 0);

        state.running_chunks.insert(
            "chunk-1".to_string(),
            RunningChunk {
                job_id: "job-1".to_string(),
                chunk_id: "chunk-1".to_string(),
                container_id: "container-1".to_string(),
                status: JobStatus::Running,
            },
        );

        assert_eq!(state.running_chunks_count(), 1);
    }
}