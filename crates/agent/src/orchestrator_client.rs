use std::time::Duration;
use reqwest::Client;
use tokio::time::sleep;
use crate::config::Config;
use crate::models::{ChunkStatusUpdate, HeartbeatPayload};

#[derive(Clone)]
pub struct OrchestratorClient {
    http: Client,
    base_url: String,
}

#[cfg(test)]
mod tests {
    use super::OrchestratorClient;
    use crate::config::Config;
    use crate::models::{ChunkStatusUpdate, HeartbeatPayload, JobStatus, NodeStatus};
    use axum::{extract::State, http::StatusCode, routing::post, Router};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use tokio::net::TcpListener;

    async fn spawn_server(router: Router) -> (String, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind test listener");
        let addr = listener
            .local_addr()
            .expect("failed to read listener addr");
        let handle = tokio::spawn(async move {
            axum::serve(listener, router)
                .await
                .expect("test server failed");
        });
        (format!("http://{}", addr), handle)
    }

    fn test_config(base_url: String) -> Config {
        Config {
            node_id: "node-test".to_string(),
            bind_addr: "127.0.0.1:8090".to_string(),
            orchestrator_base_url: base_url,
            heartbeat_interval_secs: 60,
            metrics_poll_interval_secs: 30,
            idle_cpu_threshold_pct: 15.0,
            preempt_cpu_threshold_pct: 60.0,
            idle_window_samples: 10,
            request_timeout_secs: 5,
        }
    }

    #[tokio::test]
    async fn send_heartbeat_succeeds_against_local_server() {
        let router = Router::new().route(
            "/agent/heartbeat",
            post(|| async { StatusCode::OK }),
        );

        let (base_url, server_handle) = spawn_server(router).await;
        let client = OrchestratorClient::new(&test_config(base_url));

        let payload = HeartbeatPayload {
            node_id: "node-1".to_string(),
            cpu_available_pct: 88.0,
            ram_available_mb: 4096,
            disk_available_gb: 120,
            idle_until_epoch_secs: Some(1_700_000_000),
            running_chunks: 0,
            is_idle: true,
            status: NodeStatus::Idle,
        };

        let result = client.send_heartbeat(&payload).await;
        server_handle.abort();

        assert!(result.is_ok(), "heartbeat should succeed against local server");
    }

    #[tokio::test]
    async fn send_chunk_status_retries_and_then_succeeds() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let router = Router::new()
            .route(
                "/agent/chunk-status",
                post(
                    |State(attempts): State<Arc<AtomicUsize>>| async move {
                        let current = attempts.fetch_add(1, Ordering::SeqCst);
                        if current == 0 {
                            StatusCode::INTERNAL_SERVER_ERROR
                        } else {
                            StatusCode::OK
                        }
                    },
                ),
            )
            .with_state(attempts.clone());

        let (base_url, server_handle) = spawn_server(router).await;
        let client = OrchestratorClient::new(&test_config(base_url));

        let payload = ChunkStatusUpdate {
            node_id: "node-1".to_string(),
            job_id: "job-1".to_string(),
            chunk_id: "chunk-1".to_string(),
            status: JobStatus::Running,
            detail: Some("started".to_string()),
        };

        let result = client.send_chunk_status(&payload).await;
        server_handle.abort();

        assert!(result.is_ok(), "chunk status should succeed after retry");
        assert!(
            attempts.load(Ordering::SeqCst) >= 2,
            "client should retry after initial server error"
        );
    }
}

impl OrchestratorClient {
    pub fn new(config: &Config) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(config.request_timeout_secs))
            .build()
            .expect("failed to build reqwest client");

        Self {
            http,
            base_url: config.orchestrator_base_url.trim_end_matches('/').to_string(),
        }
    }

    pub async fn send_heartbeat(&self, payload: &HeartbeatPayload) -> Result<(), String> {
        let url = format!("{}/agent/heartbeat", self.base_url);
        self.post_with_retry(&url, payload, 3).await
    }

    pub async fn send_chunk_status(&self, payload: &ChunkStatusUpdate) -> Result<(), String> {
        let url = format!("{}/agent/chunk-status", self.base_url);
        self.post_with_retry(&url, payload, 3).await
    }
    async fn post_with_retry<T: serde::Serialize + ?Sized>(
        &self,
        url: &str,
        body: &T,
        max_attempts: usize,
    ) -> Result<(), String> {
        let mut attempt = 1usize;
        loop {
            let result = self.http.post(url).json(body).send().await;

            match result {
                Ok(resp) if resp.status().is_success() => {
                    return Ok(());
                }

                Ok(resp) => {
                    if attempt >= max_attempts {
                        return Err(format!("request failed with status {}", resp.status()));
                    }
                }

                Err(err) => {
                    if attempt >= max_attempts {
                        return Err(format!("request error: {}", err));
                    }
                }
            }
            let backoff_ms = 250u64 * attempt as u64;
            sleep(Duration::from_millis(backoff_ms)).await;

            attempt += 1;
        }
    }
}