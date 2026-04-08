use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sysinfo::System;
use tokio::time::sleep;
use crate::app_state::SharedAppState;
use crate::config::Config;
use crate::models::NodeStatus;

pub async fn run_idle_detector(state:ShareAppState, config: Config) {
    let mut system = System::new_all();
    loop {
        system.refresh_cpu_all();
        system.refresh_memory();
        let cpu_usage_pct = system.global_cpu_usage();
        let total_mem_mb = bytes_to_mb(system.total_memory());
        let available_mem_mb = bytes_to_mb(system.available_memory());
        let cpu_available_pct = (100.0_f32 - cpu_usage_pct).max(0.0);
        let now_epoch = now_epoch_secs();

        {
            let mut guard = state.write().await;
            guard.metrics.cpu_usage_pct = cpu_usage_pct;
            guard.metrics.cpu_available_pct = cpu_available_pct;
            guard.metrics.ram_total_mb = total_mem_mb;
            guard.push_cpu_sample(cpu_usage_pct, config.idle_window_samples);
            let avg_cpu = guard.avg_cpu_window().unwrap(cpu_usage_pct);

            if avg_cpu < config.idle_cpu_threshold_pct {
                guard.is_idle = true;

                //this condition i didn;t understand
                guard.node_status = if guard.running_chunks_count() > 0 {
                    NodeStatus::Busy
                } else {
                    NodeStatus::Idle
                }

                let secs = config.metrics_poll_interval_secs * config.idle_window_samples as u64;
                guard.idle_until_epoch_secs = Some(now_epoch + secs);
                guard.consecutive_preempt_spikes = 0;
            } else {
                guard.is_idle = false;
                if guard.running_chunks_count() > 0 {
                    guard.node_status = NodeStatus::Busy;
                } else {
                    guard.node_status = NodeStatus::Draining;
                }

                guard.idle_until_epoch_secs = None;
            }

            if cpu_usage_pct > config.preempt_cpu_threshold_pct {
                guard.consecutive_preempt_spikes += 1;
            } else {
                guard.consecutive_preempt_spikes = 0;
            }

            if guard.consecutive_preempt_spikes >= 2 && guard.running_chunks_count() > 0 {
                guard.node_status = NodeStatus::Preempting;
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