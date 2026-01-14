use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    Json,
};
use tracing::info;

use crate::{
    db,
    models::{Metric, MetricBatch, MetricsQuery, Stats, StatsQuery},
    AppState,
};

pub async fn report_metrics(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(batch): Json<MetricBatch>,
) -> Result<StatusCode, StatusCode> {
    // Extract token from Authorization header
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Get client by token
    let client = db::get_client_by_token(&state.db, token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Update client last_seen and version
    db::update_client_last_seen(&state.db, &client.id, batch.version.as_deref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Insert metrics
    let inserted = db::insert_metrics(&state.db, &client.id, &batch.metrics)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!(
        "Received {} metrics from client {} ({})",
        inserted.len(),
        client.hostname,
        client.id
    );

    // Check alert rules
    if let Some(latest) = inserted.last() {
        check_alerts(&state, &client.id, latest).await;
    }

    Ok(StatusCode::OK)
}

async fn check_alerts(state: &AppState, client_id: &str, metric: &Metric) {
    let rules = match db::get_alert_rules_for_client(&state.db, client_id).await {
        Ok(r) => r,
        Err(_) => return,
    };

    for rule in rules {
        let value = match rule.metric_type.as_str() {
            "cpu" => Some(metric.cpu_usage),
            "ram" => Some(metric.ram_usage),
            "disk" => Some(metric.disk_usage),
            "inode" => Some(metric.inode_usage),
            "gpu" => metric.gpu_usage,
            _ => None,
        };

        if let Some(v) = value {
            if v > rule.threshold {
                // Check debounce
                let key = format!("{}:{}", client_id, rule.metric_type);
                let mut debounce = state.alert_debounce.lock().await;
                let now = std::time::Instant::now();

                if let Some(last_alert) = debounce.get(&key) {
                    if now.duration_since(*last_alert).as_secs() < 300 {
                        // 5 minute debounce
                        continue;
                    }
                }

                debounce.insert(key.clone(), now);
                drop(debounce);

                // Fire alert
                let _ = state.alert_tx.send((client_id.to_string(), rule, v)).await;
            }
        }
    }
}

pub async fn get_metrics(
    State(state): State<AppState>,
    Path(client_id): Path<String>,
    Query(query): Query<MetricsQuery>,
) -> Result<Json<Vec<Metric>>, StatusCode> {
    // Verify client exists
    db::get_client_by_id(&state.db, &client_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let metrics = db::get_metrics(&state.db, &client_id, query.hours, query.limit)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(metrics))
}

pub async fn get_stats(
    State(state): State<AppState>,
    Path(client_id): Path<String>,
    Query(query): Query<StatsQuery>,
) -> Result<Json<Vec<Stats>>, StatusCode> {
    // Verify client exists
    db::get_client_by_id(&state.db, &client_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let stats = db::get_stats(&state.db, &client_id, query.hours)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(stats))
}

pub async fn get_latest_metrics(
    State(state): State<AppState>,
    Path(client_id): Path<String>,
) -> Result<Json<Vec<Metric>>, StatusCode> {
    // Verify client exists
    db::get_client_by_id(&state.db, &client_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let metrics = db::get_latest_metrics(&state.db, &client_id, 60)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(metrics))
}
