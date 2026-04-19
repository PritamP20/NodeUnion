use std::time::Duration;
use tokio::time::sleep;
use bollard::Docker;
use crate::app_state::SharedAppState;
use crate::models::{ChunkStatusUpdate, JobStatus};
use crate::orchestrator_client::OrchestratorClient;

pub async fn run_container_monitor(
    state: SharedAppState,
    docker: Docker,
    orchestrator_client: OrchestratorClient,
    node_id: String,
) {
    loop {
        let chunks_to_check = {
            let guard = state.read().await;
            guard
                .running_chunks
                .iter()
                .map(|(chunk_id, chunk)| (chunk_id.clone(), chunk.container_id.clone(), chunk.job_id.clone()))
                .collect::<Vec<_>>()
        };

        for (chunk_id, container_id, job_id) in chunks_to_check {
            match docker.inspect_container(&container_id, None).await {
                Ok(container_info) => {
                    if let Some(state_info) = container_info.state {
                        if state_info.running == Some(false) {
                            let exit_code = state_info.exit_code.unwrap_or(-1);
                            let status = if exit_code == 0 {
                                JobStatus::Done
                            } else {
                                JobStatus::Failed
                            };

                            // Capture container logs for failed containers
                            let detail = if exit_code != 0 {
                                match get_container_logs(&docker, &container_id).await {
                                    Ok(logs) => {
                                        let truncated = if logs.len() > 500 {
                                            format!("{}... (truncated)", &logs[..500])
                                        } else {
                                            logs
                                        };
                                        Some(format!("container failed with exit code {}. Last logs:\n{}", exit_code, truncated))
                                    }
                                    Err(_) => Some(format!("container exited with code {}", exit_code)),
                                }
                            } else {
                                Some("container completed successfully".to_string())
                            };

                            let payload = ChunkStatusUpdate {
                                node_id: node_id.clone(),
                                job_id: job_id.clone(),
                                chunk_id: chunk_id.clone(),
                                status: status.clone(),
                                detail,
                            };

                            if let Err(err) = orchestrator_client.send_chunk_status(&payload).await {
                                eprintln!("failed to send chunk status update: {}", err);
                            } else {
                                let mut guard = state.write().await;
                                guard.running_chunks.remove(&chunk_id);
                            }
                        }
                    }
                }
                Err(err) => {
                    eprintln!("failed to inspect container {}: {}", container_id, err);
                }
            }
        }

        sleep(Duration::from_secs(5)).await;
    }
}

async fn get_container_logs(docker: &Docker, container_id: &str) -> Result<String, String> {
    use bollard::container::LogsOptions;
    use futures_util::stream::StreamExt;

    let options = LogsOptions::<String> {
        stdout: true,
        stderr: true,
        ..Default::default()
    };

    let mut logs_stream = docker.logs(container_id, Some(options));
    let mut output = String::new();

    while let Some(log_result) = logs_stream.next().await {
        match log_result {
            Ok(log_output) => {
                output.push_str(&log_output.to_string());
            }
            Err(_) => break,
        }
    }

    Ok(output)
}
