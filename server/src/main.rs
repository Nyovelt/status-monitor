mod db;
mod models;
mod routes;
mod services;
mod ws;

use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Instant};

use axum::{
    routing::{delete, get, post},
    Router,
};
use tokio::sync::{broadcast, mpsc, Mutex};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::models::AlertRule;

pub type DbPool = db::DbPool;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub ws_broadcast: broadcast::Sender<String>,
    pub alert_tx: mpsc::Sender<(String, AlertRule, f64)>,
    pub alert_debounce: Arc<Mutex<HashMap<String, Instant>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Database setup
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:data/monitor.db".to_string());
    let pool = db::init_db(&database_url).await?;
    info!("Database initialized at {}", database_url);

    // Create broadcast channel for WebSocket
    let (ws_tx, _) = broadcast::channel::<String>(1000);

    // Create alert channel
    let (alert_tx, alert_rx) = mpsc::channel::<(String, AlertRule, f64)>(100);

    // Create app state
    let state = AppState {
        db: pool.clone(),
        ws_broadcast: ws_tx,
        alert_tx,
        alert_debounce: Arc::new(Mutex::new(HashMap::new())),
    };

    // Start background services
    tokio::spawn(services::start_cleanup_task(pool.clone()));
    tokio::spawn(services::start_alert_worker(pool.clone(), alert_rx));

    // Build router
    let app = Router::new()
        // Client management
        .route("/api/clients", get(routes::clients::list_clients))
        .route("/api/clients", post(routes::clients::create_client))
        .route("/api/clients/{id}", get(routes::clients::get_client))
        .route("/api/clients/{id}", delete(routes::clients::delete_client))
        // Metrics
        .route("/api/report", post(routes::metrics::report_metrics))
        .route("/api/metrics/{id}", get(routes::metrics::get_metrics))
        .route("/api/metrics/{id}/latest", get(routes::metrics::get_latest_metrics))
        .route("/api/stats/{id}", get(routes::metrics::get_stats))
        // Settings & Alert Rules
        .route("/api/settings", get(routes::settings::get_settings))
        .route("/api/settings", post(routes::settings::update_settings))
        .route("/api/alerts", get(routes::settings::get_alert_rules))
        .route("/api/alerts", post(routes::settings::create_alert_rule))
        .route("/api/alerts/{id}", delete(routes::settings::delete_alert_rule))
        // WebSocket
        .route("/ws/live", get(ws::ws_handler))
        // Middleware
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
