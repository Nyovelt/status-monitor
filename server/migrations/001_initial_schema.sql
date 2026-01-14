-- Clients table: Registry of monitored servers
CREATE TABLE IF NOT EXISTS clients (
    id TEXT PRIMARY KEY NOT NULL,
    hostname TEXT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    last_seen TEXT NOT NULL,
    version TEXT
);

-- Metrics table: Time-series data
CREATE TABLE IF NOT EXISTS metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id TEXT NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    cpu_usage REAL NOT NULL,
    ram_usage REAL NOT NULL,
    disk_usage REAL NOT NULL,
    inode_usage REAL NOT NULL,
    docker_sz INTEGER,
    gpu_usage REAL,
    timestamp TEXT NOT NULL
);

-- Index for efficient range queries
CREATE INDEX IF NOT EXISTS idx_metrics_client_timestamp ON metrics(client_id, timestamp);

-- Alert rules table: User-defined thresholds
CREATE TABLE IF NOT EXISTS alert_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id TEXT REFERENCES clients(id) ON DELETE CASCADE,
    metric_type TEXT NOT NULL,
    threshold REAL NOT NULL,
    duration_sec INTEGER NOT NULL DEFAULT 30
);

-- Settings table: Global configuration
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
