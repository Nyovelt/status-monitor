Here is the full execution plan for the **Status Monitor**.

---

### Phase 1: Architecture & Data Schema

We will use **SQLite** (via `SQLx`) for simplicity and easy deployment in Docker.

#### Database Tables

1. **`clients`**: Registry of monitored servers.
* `id` (UUID), `hostname`, `token`, `last_seen`, `version` (for update checks).


2. **`metrics`**: The raw data dump (Time-Series).
* `id`, `client_id`, `cpu_usage`, `ram_usage`, `disk_usage`, `inode_usage`, `docker_sz`, `gpu_usage`, `timestamp`.
* *Index:* `(client_id, timestamp)` for fast range queries.


3. **`alert_rules`**: User-defined thresholds.
* `id`, `client_id`, `metric_type` (e.g., 'CPU'), `threshold` (e.g., 90.0), `duration_sec` (e.g., must be >90% for 30s).


4. **`settings`**: Global config.
* `key` (e.g., 'slack_webhook_url'), `value`.



---

### Phase 2: Rust Backend (The Orchestrator)

**Stack:** Axum (Server), SQLx (DB), Tokio (Async), Serde.

#### 1. API Layers

* **HTTP (Ingest):** `POST /api/report`
* Accepts JSON batch from client.
* Validates Bearer Token.
* Inserts into DB.
* **Trigger:** Pushes this new data immediately to connected WebSocket clients.
* **Trigger:** Checks `alert_rules`. If a rule is violated (and not recently alerted), fire a Slack hook.


* **WebSocket (Live View):** `ws://.../live`
* Frontend connects here.
* Server maintains a list of active WS connections (using `Arc<Mutex<>>` or `Tokio broadcast` channel).
* Broadcasts new metrics as they arrive.


* **HTTP (Management):**
* `GET /api/stats/:id`: Returns historical data (min/max/avg/p95).
* `POST /api/settings`: Set Slack URL, thresholds, etc.



#### 2. Background Services

* **Cleanup Task:** A cron job (via `tokio::time`) runs every hour.
* `DELETE FROM metrics WHERE timestamp < (NOW - 7 DAYS)`.


* **Alert Manager:**
* Uses a "Debounce" map to ensure we don't spam Slack 100 times a minute if CPU stays at 100%.



---

### Phase 3: Rust Client (The Root Agent)

**Stack:** Sysinfo, Reqwest, Tokio, Walkdir (for Docker size).
**Privilege:** Runs as `root`.

#### 1. Metric Collection (The "Fast" Loop)

* **Interval:** Every 1 second.
* **Task:** Read CPU, RAM, GPU (nvidia-smi or nvml), Inodes.
* **Buffer:** Push to a local `Vec<Metric>`.

#### 2. Heavy Metric Collection (The "Slow" Loop)

* **Interval:** Every 5 minutes.
* **Task:** Calculate `/var/lib/docker` size.
* Since we have `root`, use the `walkdir` crate to sum file sizes efficiently.
* Store this value and reuse it in the fast loop until the next update.



#### 3. Reporting (The Batch Sender)

* **Interval:** Every 10 seconds.
* **Task:** Take the buffered `Vec<Metric>`, serialize to JSON, POST to backend.
* **Retry:** If Backend is down, clear buffer (or keep 1 minute buffer) to prevent memory leaks, but log error.

#### 4. Auto-Updater

* **Interval:** On Startup + Every 24 hours.
* **Logic:**
1. Fetch `https://api.github.com/repos/YOUR_USER/YOUR_REPO/releases/latest`.
2. Compare `tag_name` with current `Cargo.toml` version.
3. If newer:
* Download binary to `/tmp/new_client`.
* `chmod +x`.
* Move to `/usr/local/bin/status-monitor-client`.
* Exit process (Systemd will auto-restart it, loading the new binary).





---

### Phase 4: Frontend (Next.js Dashboard)

**Stack:** Next.js 14+ (App Router), Tailwind, Recharts, `use-websocket`.

#### 1. Views

* **Dashboard (Home):** Grid of cards for each Client.
* Cards show: Hostname, Live Sparkline (last 5 mins), Status Badge (Green/Red).


* **Detail View (`/client/[id]`):**
* **Live Tab:** Real-time updating gauges.
* **History Tab:** Recharts Line Chart (12h, 24h, 7d).
* **Stats Tab:** Table showing Average, **Max**, and **P95** (95th percentile) for the selected range.


* **Settings View:** Input field for Slack Webhook URL and Threshold configuration.

#### 2. State Management

* **Live Data:** Handle via a Context Provider that wraps the app with the WebSocket connection.
* **Historical Data:** Use `TanStack Query` (React Query) to fetch from the REST API.

---

### Phase 5: Deployment (Docker Compose)

We will deploy the Backend, Frontend, and Database together. The Client is deployed separately on the target servers.

**`docker-compose.yml`**:

```yaml
version: '3.8'
services:
  # The Rust Backend
  server:
    build: ./server
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=sqlite://data/monitor.db
      - RUST_LOG=info
    volumes:
      - ./data:/app/data
    restart: always

  # Next.js Frontend
  web:
    build: ./web
    ports:
      - "3000:3000"
    environment:
      - NEXT_PUBLIC_API_URL=http://localhost:8080
      - NEXT_PUBLIC_WS_URL=ws://localhost:8080/ws
    depends_on:
      - server
    restart: always

  # Optional: Reverse Proxy (Caddy/Nginx) for HTTPS
  # caddy: ...

```

---

### Summary of Responsibilities

| Component | Responsibility | Frequency |
| --- | --- | --- |
| **Client** | Collect CPU/RAM | Every 1s |
| **Client** | Collect Docker Dir Size | Every 5m |
| **Client** | HTTP POST Batch to Server | Every 10s |
| **Client** | Check GitHub for Updates | Every 24h |
| **Server** | Push to WebSocket | Immediate on receipt |
| **Server** | Check Alerts (Slack) | On receipt |
| **Server** | Prune Old Data (>7d) | Every 1h |
| **Frontend** | Calculate P95/Max | On Demand (API Request) |

### Immediate Next Step

I recommend setting up the **Rust Backend API and Database schema** first, as both the Frontend and Client depend on the API contract.

