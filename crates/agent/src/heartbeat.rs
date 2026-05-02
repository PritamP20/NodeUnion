use std::time::Duration;
use tokio::time::sleep;
use crate::app_state::SharedAppState;
use crate::config::Config;
use crate::models::HeartbeatPayload;
use crate::orchestrator_client::OrchestratorClient;

pub async fn run_heartbeat_loop(
    state: SharedAppState,
    client: OrchestratorClient,
    config: Config,
) {
    loop {
        let payload = {
            let guard = state.read().await;

            HeartbeatPayload {
                node_id: guard.node_id.clone(),
                network_id: config.network_id.clone(),
                cpu_available_pct: guard.metrics.cpu_available_pct,
                ram_available_mb: guard.metrics.ram_available_mb,
                disk_available_gb: guard.metrics.disk_available_gb,
                idle_until_epoch_secs: guard.idle_until_epoch_secs,
                running_chunks: guard.running_chunks_count(),
                is_idle: guard.is_idle,
                status: guard.node_status.clone(),
            }
        };

        if let Err(err) = client.send_heartbeat(&payload).await {
            eprintln!("heartbeat send failed: {}", err);
        }

        sleep(Duration::from_secs(config.heartbeat_interval_secs)).await;
    }
}