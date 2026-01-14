use anyhow::Result;
use chrono::{Duration, Utc};
use sqlx::{Pool, Sqlite, SqlitePool};
use std::path::Path;
use uuid::Uuid;

use crate::models::{AlertRule, AlertRuleInput, Client, Metric, MetricInput, Setting, Stats};

pub type DbPool = Pool<Sqlite>;

pub async fn init_db(database_url: &str) -> Result<DbPool> {
    // Create database file if it doesn't exist
    if database_url.starts_with("sqlite:") {
        let path = database_url.trim_start_matches("sqlite:");
        if let Some(parent) = Path::new(path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }

    let pool = SqlitePool::connect(database_url).await?;

    // Run migrations
    run_migrations(&pool).await?;

    Ok(pool)
}

async fn run_migrations(pool: &DbPool) -> Result<()> {
    let migration = include_str!("../migrations/001_initial_schema.sql");
    sqlx::raw_sql(migration).execute(pool).await?;
    Ok(())
}

// Client operations
pub async fn get_client_by_token(pool: &DbPool, token: &str) -> Result<Option<Client>> {
    let client = sqlx::query_as::<_, Client>("SELECT * FROM clients WHERE token = ?")
        .bind(token)
        .fetch_optional(pool)
        .await?;
    Ok(client)
}

pub async fn get_client_by_id(pool: &DbPool, id: &str) -> Result<Option<Client>> {
    let client = sqlx::query_as::<_, Client>("SELECT * FROM clients WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(client)
}

pub async fn get_all_clients(pool: &DbPool) -> Result<Vec<Client>> {
    let clients = sqlx::query_as::<_, Client>("SELECT * FROM clients ORDER BY hostname")
        .fetch_all(pool)
        .await?;
    Ok(clients)
}

pub async fn create_client(pool: &DbPool, hostname: &str) -> Result<Client> {
    let id = Uuid::new_v4().to_string();
    let token = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO clients (id, hostname, token, last_seen) VALUES (?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(hostname)
    .bind(&token)
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(Client {
        id,
        hostname: hostname.to_string(),
        token,
        last_seen: now,
        version: None,
    })
}

pub async fn update_client_last_seen(
    pool: &DbPool,
    client_id: &str,
    version: Option<&str>,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE clients SET last_seen = ?, version = COALESCE(?, version) WHERE id = ?")
        .bind(&now)
        .bind(version)
        .bind(client_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_client(pool: &DbPool, id: &str) -> Result<bool> {
    let result = sqlx::query("DELETE FROM clients WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

// Metric operations
pub async fn insert_metrics(
    pool: &DbPool,
    client_id: &str,
    metrics: &[MetricInput],
) -> Result<Vec<Metric>> {
    let mut inserted = Vec::new();

    for m in metrics {
        let result = sqlx::query(
            r#"
            INSERT INTO metrics (client_id, cpu_usage, ram_usage, disk_usage, inode_usage, docker_sz, gpu_usage, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(client_id)
        .bind(m.cpu_usage)
        .bind(m.ram_usage)
        .bind(m.disk_usage)
        .bind(m.inode_usage)
        .bind(m.docker_sz)
        .bind(m.gpu_usage)
        .bind(&m.timestamp)
        .execute(pool)
        .await?;

        inserted.push(Metric {
            id: result.last_insert_rowid(),
            client_id: client_id.to_string(),
            cpu_usage: m.cpu_usage,
            ram_usage: m.ram_usage,
            disk_usage: m.disk_usage,
            inode_usage: m.inode_usage,
            docker_sz: m.docker_sz,
            gpu_usage: m.gpu_usage,
            timestamp: m.timestamp.clone(),
        });
    }

    Ok(inserted)
}

pub async fn get_metrics(
    pool: &DbPool,
    client_id: &str,
    hours: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<Metric>> {
    let hours = hours.unwrap_or(24);
    let limit = limit.unwrap_or(1000);
    let since = (Utc::now() - Duration::hours(hours)).to_rfc3339();

    let metrics = sqlx::query_as::<_, Metric>(
        r#"
        SELECT * FROM metrics
        WHERE client_id = ? AND timestamp >= ?
        ORDER BY timestamp DESC
        LIMIT ?
        "#,
    )
    .bind(client_id)
    .bind(&since)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(metrics)
}

pub async fn get_latest_metrics(pool: &DbPool, client_id: &str, count: i64) -> Result<Vec<Metric>> {
    let metrics = sqlx::query_as::<_, Metric>(
        r#"
        SELECT * FROM metrics
        WHERE client_id = ?
        ORDER BY timestamp DESC
        LIMIT ?
        "#,
    )
    .bind(client_id)
    .bind(count)
    .fetch_all(pool)
    .await?;

    Ok(metrics)
}

pub async fn get_stats(pool: &DbPool, client_id: &str, hours: Option<i64>) -> Result<Vec<Stats>> {
    let hours = hours.unwrap_or(24);
    let since = (Utc::now() - Duration::hours(hours)).to_rfc3339();

    // Get raw metrics for calculation
    let metrics = sqlx::query_as::<_, Metric>(
        r#"
        SELECT * FROM metrics
        WHERE client_id = ? AND timestamp >= ?
        ORDER BY timestamp ASC
        "#,
    )
    .bind(client_id)
    .bind(&since)
    .fetch_all(pool)
    .await?;

    if metrics.is_empty() {
        return Ok(vec![]);
    }

    let mut stats = vec![];

    // Calculate stats for each metric type
    for (metric_type, values) in [
        ("cpu", metrics.iter().map(|m| m.cpu_usage).collect::<Vec<_>>()),
        ("ram", metrics.iter().map(|m| m.ram_usage).collect::<Vec<_>>()),
        ("disk", metrics.iter().map(|m| m.disk_usage).collect::<Vec<_>>()),
        ("inode", metrics.iter().map(|m| m.inode_usage).collect::<Vec<_>>()),
    ] {
        if !values.is_empty() {
            stats.push(calculate_stats(client_id, metric_type, &values));
        }
    }

    // GPU stats (if available)
    let gpu_values: Vec<f64> = metrics.iter().filter_map(|m| m.gpu_usage).collect();
    if !gpu_values.is_empty() {
        stats.push(calculate_stats(client_id, "gpu", &gpu_values));
    }

    // Docker size stats (if available)
    let docker_values: Vec<f64> = metrics
        .iter()
        .filter_map(|m| m.docker_sz.map(|v| v as f64))
        .collect();
    if !docker_values.is_empty() {
        stats.push(calculate_stats(client_id, "docker", &docker_values));
    }

    Ok(stats)
}

fn calculate_stats(client_id: &str, metric_type: &str, values: &[f64]) -> Stats {
    let count = values.len() as i64;
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let avg = values.iter().sum::<f64>() / count as f64;

    // Calculate P95
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p95_idx = ((count as f64 * 0.95) as usize).min(sorted.len() - 1);
    let p95 = sorted[p95_idx];

    Stats {
        client_id: client_id.to_string(),
        metric_type: metric_type.to_string(),
        min,
        max,
        avg,
        p95,
        count,
    }
}

pub async fn delete_old_metrics(pool: &DbPool, days: i64) -> Result<u64> {
    let cutoff = (Utc::now() - Duration::days(days)).to_rfc3339();
    let result = sqlx::query("DELETE FROM metrics WHERE timestamp < ?")
        .bind(&cutoff)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

// Alert rule operations
pub async fn get_alert_rules(pool: &DbPool) -> Result<Vec<AlertRule>> {
    let rules = sqlx::query_as::<_, AlertRule>("SELECT * FROM alert_rules")
        .fetch_all(pool)
        .await?;
    Ok(rules)
}

pub async fn get_alert_rules_for_client(pool: &DbPool, client_id: &str) -> Result<Vec<AlertRule>> {
    let rules = sqlx::query_as::<_, AlertRule>(
        "SELECT * FROM alert_rules WHERE client_id IS NULL OR client_id = ?",
    )
    .bind(client_id)
    .fetch_all(pool)
    .await?;
    Ok(rules)
}

pub async fn create_alert_rule(pool: &DbPool, rule: &AlertRuleInput) -> Result<AlertRule> {
    let duration_sec = rule.duration_sec.unwrap_or(30);
    let result = sqlx::query(
        "INSERT INTO alert_rules (client_id, metric_type, threshold, duration_sec) VALUES (?, ?, ?, ?)",
    )
    .bind(&rule.client_id)
    .bind(&rule.metric_type)
    .bind(rule.threshold)
    .bind(duration_sec)
    .execute(pool)
    .await?;

    Ok(AlertRule {
        id: result.last_insert_rowid(),
        client_id: rule.client_id.clone(),
        metric_type: rule.metric_type.clone(),
        threshold: rule.threshold,
        duration_sec,
    })
}

pub async fn delete_alert_rule(pool: &DbPool, id: i64) -> Result<bool> {
    let result = sqlx::query("DELETE FROM alert_rules WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

// Settings operations
pub async fn get_setting(pool: &DbPool, key: &str) -> Result<Option<String>> {
    let setting = sqlx::query_as::<_, Setting>("SELECT * FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;
    Ok(setting.map(|s| s.value))
}

pub async fn get_all_settings(pool: &DbPool) -> Result<Vec<Setting>> {
    let settings = sqlx::query_as::<_, Setting>("SELECT * FROM settings")
        .fetch_all(pool)
        .await?;
    Ok(settings)
}

pub async fn set_setting(pool: &DbPool, key: &str, value: &str) -> Result<()> {
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = ?",
    )
    .bind(key)
    .bind(value)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}
