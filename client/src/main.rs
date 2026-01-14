mod config;
mod metrics;
mod reporter;
mod updater;

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::{interval, sleep};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::Config;
use metrics::MetricCollector;
use reporter::Reporter;
use updater::Updater;

const FAST_INTERVAL: Duration = Duration::from_secs(1);
const SLOW_INTERVAL: Duration = Duration::from_secs(300); // 5 minutes
const REPORT_INTERVAL: Duration = Duration::from_secs(10);
const UPDATE_INTERVAL: Duration = Duration::from_secs(86400); // 24 hours

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "status_monitor_client=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Load configuration
    let config = Config::from_env()?;
    info!("Starting status-monitor-client v{}", env!("CARGO_PKG_VERSION"));
    info!("Hostname: {}", config.hostname);
    info!("Server: {}", config.server_url);
    info!("Docker path: {}", config.docker_path);

    // Initialize components
    let collector = Arc::new(Mutex::new(MetricCollector::new(config.docker_path.clone())));
    let reporter = Arc::new(Reporter::new(config.clone()));
    let updater = Updater::new(config.github_repo.clone());

    // Check for updates on startup
    if let Err(e) = updater.check_and_update().await {
        error!("Update check failed: {}", e);
    }

    // Spawn fast collection loop (1s interval)
    let collector_fast = Arc::clone(&collector);
    let reporter_fast = Arc::clone(&reporter);
    tokio::spawn(async move {
        let mut ticker = interval(FAST_INTERVAL);
        // Skip first tick (immediate)
        ticker.tick().await;

        loop {
            ticker.tick().await;

            let metric = {
                let mut collector = collector_fast.lock().await;
                collector.collect_fast()
            };

            reporter_fast.add_metric(metric).await;
        }
    });

    // Spawn slow collection loop (5m interval) - Docker size
    let collector_slow = Arc::clone(&collector);
    tokio::spawn(async move {
        // Initial collection
        {
            let collector = collector_slow.lock().await;
            collector.update_docker_size();
        }

        let mut ticker = interval(SLOW_INTERVAL);
        ticker.tick().await; // Skip first (we already did initial)

        loop {
            ticker.tick().await;
            let collector = collector_slow.lock().await;
            collector.update_docker_size();
        }
    });

    // Spawn report loop (10s interval)
    let reporter_send = Arc::clone(&reporter);
    tokio::spawn(async move {
        let mut ticker = interval(REPORT_INTERVAL);

        loop {
            ticker.tick().await;

            if let Err(e) = reporter_send.send_batch().await {
                error!("Failed to send batch: {}", e);
            }
        }
    });

    // Spawn update check loop (24h interval)
    tokio::spawn(async move {
        loop {
            sleep(UPDATE_INTERVAL).await;

            if let Err(e) = updater.check_and_update().await {
                error!("Update check failed: {}", e);
            }
        }
    });

    // Keep main task alive
    info!("Client started, collecting metrics...");
    loop {
        sleep(Duration::from_secs(3600)).await;
    }
}
