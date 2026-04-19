use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sysinfo::{Disks, System};
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
        let disks = Disks::new_with_refreshed_list();
        let cpu_usage_pct = system.global_cpu_info().cpu_usage();
        let total_mem_bytes = system.total_memory().max(macos_total_memory_bytes().unwrap_or(0));
        let total_mem_mb = bytes_to_mb(total_mem_bytes);
        let available_mem_bytes = system.available_memory();
        let fallback_available_bytes = system
            .free_memory()
            .max(system.total_memory().saturating_sub(system.used_memory()));
        let macos_available_bytes = macos_available_memory_bytes().unwrap_or(0);
        let available_mem_mb = bytes_to_mb(
            available_mem_bytes
                .max(fallback_available_bytes)
                .max(macos_available_bytes),
        );
        let disk_available_gb = bytes_to_gb(
            disks
                .iter()
                .map(|disk| disk.available_space())
                .max()
                .unwrap_or(0),
        );
        let cpu_available_pct = (100.0_f32 - cpu_usage_pct).max(0.0);
        let now_epoch = now_epoch_secs();

        {
            let mut guard = state.write().await;
            guard.metrics.cpu_usage_pct = cpu_usage_pct;
            guard.metrics.cpu_available_pct = cpu_available_pct;
            guard.metrics.ram_total_mb = total_mem_mb;
            guard.metrics.ram_available_mb = available_mem_mb;
            guard.metrics.disk_available_gb = disk_available_gb;
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

fn bytes_to_gb(bytes: u64) -> u64 {
    bytes / (1024 * 1024 * 1024)
}

#[cfg(target_os = "macos")]
fn macos_total_memory_bytes() -> Option<u64> {
    let output = Command::new("sysctl").arg("-n").arg("hw.memsize").output().ok()?;
    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8(output.stdout).ok()?;
    value.trim().parse::<u64>().ok()
}

#[cfg(not(target_os = "macos"))]
fn macos_total_memory_bytes() -> Option<u64> {
    None
}

#[cfg(target_os = "macos")]
fn macos_available_memory_bytes() -> Option<u64> {
    let output = Command::new("vm_stat").output().ok()?;
    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8(output.stdout).ok()?;
    let page_size = parse_vm_stat_page_size(&text)?;
    let free_pages = parse_vm_stat_pages(&text, "Pages free")?;
    let inactive_pages = parse_vm_stat_pages(&text, "Pages inactive")?;
    let speculative_pages = parse_vm_stat_pages(&text, "Pages speculative")?;

    Some((free_pages + inactive_pages + speculative_pages) * page_size)
}

#[cfg(not(target_os = "macos"))]
fn macos_available_memory_bytes() -> Option<u64> {
    None
}

#[cfg(target_os = "macos")]
fn parse_vm_stat_page_size(text: &str) -> Option<u64> {
    for line in text.lines() {
        if let Some(value) = line.split_once("page size of") {
            return value.1.split_whitespace().next()?.parse::<u64>().ok();
        }
    }

    None
}

#[cfg(target_os = "macos")]
fn parse_vm_stat_pages(text: &str, key: &str) -> Option<u64> {
    for line in text.lines() {
        if line.trim_start().starts_with(key) {
            let value = line
                .split_once(':')?
                .1
                .trim()
                .trim_end_matches('.')
                .parse::<u64>()
                .ok()?;
            return Some(value);
        }
    }

    None
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