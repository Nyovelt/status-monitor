use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Client {
    pub id: String,
    pub hostname: String,
    pub token: String,
    pub last_seen: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientResponse {
    pub id: String,
    pub hostname: String,
    pub last_seen: String,
    pub version: Option<String>,
}

impl From<Client> for ClientResponse {
    fn from(c: Client) -> Self {
        Self {
            id: c.id,
            hostname: c.hostname,
            last_seen: c.last_seen,
            version: c.version,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Metric {
    pub id: i64,
    pub client_id: String,
    pub cpu_usage: f64,
    pub ram_usage: f64,
    pub disk_usage: f64,
    pub inode_usage: f64,
    pub docker_sz: Option<i64>,
    pub gpu_usage: Option<f64>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricInput {
    pub cpu_usage: f64,
    pub ram_usage: f64,
    pub disk_usage: f64,
    pub inode_usage: f64,
    pub docker_sz: Option<i64>,
    pub gpu_usage: Option<f64>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricBatch {
    pub hostname: String,
    pub version: Option<String>,
    pub metrics: Vec<MetricInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AlertRule {
    pub id: i64,
    pub client_id: Option<String>,
    pub metric_type: String,
    pub threshold: f64,
    pub duration_sec: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRuleInput {
    pub client_id: Option<String>,
    pub metric_type: String,
    pub threshold: f64,
    pub duration_sec: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub client_id: String,
    pub metric_type: String,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub p95: f64,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsQuery {
    pub metric_type: Option<String>,
    pub hours: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsQuery {
    pub hours: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClientRequest {
    pub hostname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClientResponse {
    pub id: String,
    pub hostname: String,
    pub token: String,
}
