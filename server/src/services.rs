use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::{db, models::AlertRule, DbPool};

pub async fn start_cleanup_task(pool: DbPool) {
    let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Every hour

    loop {
        interval.tick().await;

        match db::delete_old_metrics(&pool, 7).await {
            Ok(deleted) => {
                if deleted > 0 {
                    info!("Cleanup task: deleted {} old metrics", deleted);
                }
            }
            Err(e) => {
                error!("Cleanup task failed: {}", e);
            }
        }
    }
}

pub async fn start_alert_worker(
    pool: DbPool,
    mut rx: mpsc::Receiver<(String, AlertRule, f64)>,
) {
    while let Some((client_id, rule, value)) = rx.recv().await {
        // Get Slack webhook URL
        let webhook_url = match db::get_setting(&pool, "slack_webhook_url").await {
            Ok(Some(url)) if !url.is_empty() => url,
            _ => continue,
        };

        // Get client hostname
        let hostname = match db::get_client_by_id(&pool, &client_id).await {
            Ok(Some(client)) => client.hostname,
            _ => client_id.clone(),
        };

        // Send Slack notification
        let message = format!(
            "🚨 *Alert*: {} on `{}` is at {:.1}% (threshold: {:.1}%)",
            rule.metric_type.to_uppercase(),
            hostname,
            value,
            rule.threshold
        );

        if let Err(e) = send_slack_notification(&webhook_url, &message).await {
            error!("Failed to send Slack notification: {}", e);
        } else {
            info!("Sent alert for {} on {}: {:.1}%", rule.metric_type, hostname, value);
        }
    }
}

async fn send_slack_notification(webhook_url: &str, message: &str) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    let payload = serde_json::json!({
        "text": message,
        "mrkdwn": true
    });

    let response = client
        .post(webhook_url)
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("Slack API returned status: {}", response.status());
    }

    Ok(())
}
