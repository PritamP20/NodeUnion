use std::env;
#[derive(Debug, Clone)]
pub struct Config {
    pub node_id: String,
    pub bind_addr: String,
    pub orchestrator_base_url: String,
    pub heartbeat_interval_secs: u64,
    pub metrics_poll_interval_secs: u64,
    pub idle_cpu_threshold_pct: f32,
    pub preempt_cpu_threshold_pct: f32,
    pub idle_window_samples: usize,
    pub request_timeout_secs: u64
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            node_id: read_string("NODE_ID", "local-node-1"),
            bind_addr: read_string("AGENT_BIND_ADDR", "0.0.0.0:8090"),
            orchestrator_base_url: read_string("ORCHESTRATOR_BASE_URL", "http://127.0.0.1:8080"),
            heartbeat_interval_secs: read_u64("HEARTBEAT_INTERVAL_SECS", 60),
            metrics_poll_interval_secs: read_u64("METRICS_POLL_INTERVAL_SECS", 30),
            idle_cpu_threshold_pct: read_f32("IDLE_CPU_THRESHOLD_PCT", 15.0),
            preempt_cpu_threshold_pct: read_f32("PREEMPT_CPU_THRESHOLD_PCT", 60.0),
            idle_window_samples: read_usize("IDLE_WINDOW_SAMPLES", 10),
            request_timeout_secs: read_u64("REQUEST_TIMEOUT_SECS", 10),
        }
    }
}

fn read_string(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn read_u64(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(default)
}

fn read_usize(key: &str, default: usize) -> usize {
    env::var(key)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(default)
}

fn read_f32(key: &str, default: f32) -> f32 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(default)
}