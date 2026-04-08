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

impl OrchestratorClient {
    pub fn new(config: &Config) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(config.request_timeout_secs))
            .build()
            .expect("failed to build reqwest client");

        Self {
            http,
            base_url: config.orchestrator_base_url.trim_end_matches('/').to_string();
        }
    }

    pub async fn send_heartbeat(&self, payload: &HeartbeatPayload) -> Result<(), String> {
        let url = format!("{}/agent/heartbeat", self.base_url);
        self.post_with_retry(&url, payload, 3).await;
    }

    pub async fn send_chunk_status(&self, payload: &ChunkStatusUpdate) -> Result<(), String> {
        let url = format!("{}/agnet/chunk-status", self.base_url);
        self.post_with_retry(&url, payload, 3).await
    }
    async fn post_with_retry<T: serde::Serialize + ?Sized> (
        &self,
        url: &str,
        body: &T,
        max_attempts: usize
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

            attempt +=1;
        }
    }
}