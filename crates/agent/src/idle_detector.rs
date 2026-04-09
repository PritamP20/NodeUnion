use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sysinfo::System;
use tokio::time::sleep;
use crate::app_state::SharedAppState;
use crate::config::Config;
use crate::models::NodeStatus;

fn evaluate_state(
    avg_cpu: f32,
    cpu_usage_pct: f32,
    running_chunks: usize,
    idle_cpu_threshold_pct: f32,
    preempt_cpu_threshold_pct: f32,
    previous_spikes: usize,
) -> (bool, NodeStatus, usize) {
    let mut consecutive_spikes = if cpu_usage_pct > preempt_cpu_threshold_pct {
        previous_spikes + 1
    } else {
        0
    };

    let mut status = if avg_cpu < idle_cpu_threshold_pct {
        if running_chunks > 0 {
            NodeStatus::Busy
        } else {
            NodeStatus::Idle
        }
    } else if running_chunks > 0 {
        NodeStatus::Busy
    } else {
        NodeStatus::Draining
    };

    if consecutive_spikes >= 2 && running_chunks > 0 {
        status = NodeStatus::Preempting;
    }

    // If node is clearly idle, reset spikes to avoid stale preemption state.
    if matches!(status, NodeStatus::Idle) {
        consecutive_spikes = 0;
    }

    let is_idle = matches!(status, NodeStatus::Idle);
    (is_idle, status, consecutive_spikes)
}

pub async fn run_idle_detector(state: SharedAppState, config: Config) {
    let mut system = System::new_all();
    loop {
        system.refresh_cpu();
        system.refresh_memory();
        let cpu_usage_pct = system.global_cpu_info().cpu_usage();
        let total_mem_mb = bytes_to_mb(system.total_memory());
        let available_mem_mb = bytes_to_mb(system.available_memory());
        let cpu_available_pct = (100.0_f32 - cpu_usage_pct).max(0.0);
        let now_epoch = now_epoch_secs();

        {
            let mut guard = state.write().await;
            guard.metrics.cpu_usage_pct = cpu_usage_pct;
            guard.metrics.cpu_available_pct = cpu_available_pct;
            guard.metrics.ram_total_mb = total_mem_mb;
            guard.metrics.ram_available_mb = available_mem_mb;
            guard.push_cpu_sample(cpu_usage_pct, config.idle_window_samples);
            let avg_cpu = guard.avg_cpu_window().unwrap_or(cpu_usage_pct);

            let (is_idle, node_status, consecutive_preempt_spikes) = evaluate_state(
                avg_cpu,
                cpu_usage_pct,
                guard.running_chunks_count(),
                config.idle_cpu_threshold_pct,
                config.preempt_cpu_threshold_pct,
                guard.consecutive_preempt_spikes,
            );

            guard.is_idle = is_idle;
            guard.node_status = node_status;
            guard.consecutive_preempt_spikes = consecutive_preempt_spikes;

            if guard.is_idle {
                let secs = config.metrics_poll_interval_secs * config.idle_window_samples as u64;
                guard.idle_until_epoch_secs = Some(now_epoch + secs);
            } else {
                guard.idle_until_epoch_secs = None;
            }
        }

        sleep(Duration::from_secs(config.metrics_poll_interval_secs)).await;
    }
}

fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn bytes_to_mb(bytes: u64) -> u64 {
    bytes / (1024 * 1024)
}

#[cfg(test)]
mod tests {
    use super::evaluate_state;
    use crate::models::NodeStatus;

    #[test]
    fn becomes_idle_when_cpu_is_low_and_no_running_chunks() {
        let (is_idle, status, spikes) = evaluate_state(10.0, 10.0, 0, 15.0, 60.0, 0);
        assert!(is_idle);
        assert!(matches!(status, NodeStatus::Idle));
        assert_eq!(spikes, 0);
    }

    #[test]
    fn becomes_draining_when_not_idle_and_no_running_chunks() {
        let (is_idle, status, spikes) = evaluate_state(35.0, 35.0, 0, 15.0, 60.0, 0);
        assert!(!is_idle);
        assert!(matches!(status, NodeStatus::Draining));
        assert_eq!(spikes, 0);
    }

    #[test]
    fn becomes_preempting_after_two_high_cpu_spikes_with_running_chunks() {
        let (_, first_status, first_spikes) = evaluate_state(50.0, 70.0, 1, 15.0, 60.0, 0);
        assert!(matches!(first_status, NodeStatus::Busy));
        assert_eq!(first_spikes, 1);

        let (is_idle, second_status, second_spikes) =
            evaluate_state(50.0, 75.0, 1, 15.0, 60.0, first_spikes);
        assert!(!is_idle);
        assert!(matches!(second_status, NodeStatus::Preempting));
        assert_eq!(second_spikes, 2);
    }
}