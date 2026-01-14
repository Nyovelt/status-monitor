use reqwest::Client;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::metrics::Metric;

const MAX_BUFFER_SIZE: usize = 120; // ~2 minutes of metrics at 1s intervals

#[derive(Debug, Serialize)]
struct MetricBatch {
    hostname: String,
    version: Option<String>,
    metrics: Vec<Metric>,
}

pub struct Reporter {
    client: Client,
    config: Config,
    buffer: Arc<Mutex<Vec<Metric>>>,
}

impl Reporter {
    pub fn new(config: Config) -> Self {
        Self {
            client: Client::new(),
            config,
            buffer: Arc::new(Mutex::new(Vec::with_capacity(MAX_BUFFER_SIZE))),
        }
    }

    pub fn get_buffer(&self) -> Arc<Mutex<Vec<Metric>>> {
        Arc::clone(&self.buffer)
    }

    /// Add a metric to the buffer
    pub async fn add_metric(&self, metric: Metric) {
        let mut buffer = self.buffer.lock().await;

        // If buffer is full, remove oldest metrics
        if buffer.len() >= MAX_BUFFER_SIZE {
            let overflow = buffer.len() - MAX_BUFFER_SIZE + 1;
            buffer.drain(0..overflow);
            warn!("Buffer overflow, dropped {} old metrics", overflow);
        }

        buffer.push(metric);
        debug!("Buffered metric, total: {}", buffer.len());
    }

    /// Send all buffered metrics to the server
    pub async fn send_batch(&self) -> anyhow::Result<()> {
        let metrics = {
            let mut buffer = self.buffer.lock().await;
            if buffer.is_empty() {
                return Ok(());
            }
            std::mem::take(&mut *buffer)
        };

        let count = metrics.len();
        let batch = MetricBatch {
            hostname: self.config.hostname.clone(),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
            metrics,
        };

        let url = format!("{}/api/report", self.config.server_url);

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.config.token)
            .json(&batch)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                info!("Sent {} metrics to server", count);
                Ok(())
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                error!("Server returned error {}: {}", status, body);

                // Re-buffer metrics on failure (except for auth errors)
                if !status.is_client_error() {
                    self.rebuffer_metrics(batch.metrics).await;
                }

                anyhow::bail!("Server error: {}", status)
            }
            Err(e) => {
                error!("Failed to send metrics: {}", e);
                self.rebuffer_metrics(batch.metrics).await;
                anyhow::bail!("Request failed: {}", e)
            }
        }
    }

    async fn rebuffer_metrics(&self, metrics: Vec<Metric>) {
        let mut buffer = self.buffer.lock().await;

        // Prepend old metrics back to buffer
        let available_space = MAX_BUFFER_SIZE.saturating_sub(buffer.len());
        let to_keep = metrics.len().min(available_space);

        if to_keep > 0 {
            let mut new_buffer = metrics.into_iter().take(to_keep).collect::<Vec<_>>();
            new_buffer.append(&mut *buffer);
            *buffer = new_buffer;
            debug!("Re-buffered {} metrics for retry", to_keep);
        }
    }
}
