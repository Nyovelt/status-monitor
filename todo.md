# Status Monitor - Implementation Todo

## Phase 1: Rust Backend (Server) ✅

### 1.1 Project Setup
- [x] Initialize Rust project with Cargo (`server/`)
- [x] Add dependencies: axum, sqlx, tokio, serde, reqwest, uuid, chrono
- [x] Configure SQLx for SQLite with migrations

### 1.2 Database Schema
- [x] Create migration: `clients` table (id, hostname, token, last_seen, version)
- [x] Create migration: `metrics` table (id, client_id, cpu_usage, ram_usage, disk_usage, inode_usage, docker_sz, gpu_usage, timestamp)
- [x] Create migration: `alert_rules` table (id, client_id, metric_type, threshold, duration_sec)
- [x] Create migration: `settings` table (key, value)
- [x] Add index on `metrics(client_id, timestamp)`

### 1.3 API Endpoints
- [x] `POST /api/report` - Metric ingestion with Bearer token auth
- [x] `GET /api/clients` - List all clients
- [x] `GET /api/stats/:id` - Historical stats (min/max/avg/p95)
- [x] `GET /api/metrics/:id` - Raw metrics with time range query
- [x] `POST /api/settings` - Update global settings
- [x] `GET /api/settings` - Retrieve settings

### 1.4 WebSocket
- [x] Setup WebSocket endpoint at `/ws/live`
- [x] Implement broadcast channel for connected clients
- [x] Push new metrics to all connected frontends on receipt

### 1.5 Background Services
- [x] Cleanup task: prune metrics older than 7 days (hourly)
- [x] Alert manager with debounce logic
- [x] Slack webhook integration for alerts

### 1.6 Docker
- [x] Create `Dockerfile` for backend
- [x] Configure environment variables (DATABASE_URL, RUST_LOG)

---

## Phase 2: Rust Client (Agent) ✅

### 2.1 Project Setup
- [x] Initialize Rust project with Cargo (`client/`)
- [x] Add dependencies: sysinfo, reqwest, tokio, walkdir, serde, nvml-wrapper
- [x] Setup as systemd service template

### 2.2 Fast Metric Collection (1s interval)
- [x] CPU usage collection
- [x] RAM usage collection
- [x] GPU usage via NVML (NVIDIA)
- [x] Inode usage collection
- [x] Buffer metrics in local Vec

### 2.3 Slow Metric Collection (5m interval)
- [x] Docker directory size calculation (`/var/lib/docker`)
- [x] Cache value for reuse in fast loop

### 2.4 Batch Reporting (10s interval)
- [x] Serialize buffered metrics to JSON
- [x] POST to backend with Bearer token
- [x] Retry logic with buffer management
- [x] Error logging on failure

### 2.5 Auto-Updater (24h interval)
- [x] Fetch latest release from GitHub API
- [x] Compare versions with current binary
- [x] Download and replace binary
- [x] Exit for systemd restart

### 2.6 Deployment
- [x] Create systemd service file
- [x] Create install script
- [x] Build release binary

---

## Phase 3: Next.js Frontend (Dashboard) ✅

### 3.1 Project Setup
- [x] Initialize Next.js 14+ with App Router (`web/`)
- [x] Configure TypeScript
- [x] Setup Tailwind CSS
- [x] Add dependencies: recharts, @tanstack/react-query, react-use-websocket

### 3.2 Layout & Navigation
- [x] Create root layout with navigation
- [x] Dark theme implemented
- [x] Responsive header with navigation

### 3.3 Dashboard View (Home)
- [x] Client cards grid layout
- [x] Live sparkline charts (last 60 data points)
- [x] Status badges (green/red based on online status)
- [x] Last seen timestamp

### 3.4 Client Detail View (`/client/[id]`)
- [x] Live tab: Real-time gauges for all metrics
- [x] History tab: Line charts with time range selector (12h/24h/7d)
- [x] Stats tab: Table with avg/max/p95 statistics

### 3.5 Settings View
- [x] Slack webhook URL configuration
- [x] Alert threshold configuration per metric
- [x] Client management (add, view, remove clients)

### 3.6 State Management
- [x] WebSocket hook for live data
- [x] TanStack Query setup for REST API calls
- [x] Type definitions for API responses

### 3.7 Docker
- [x] Create `Dockerfile` for frontend
- [x] Configure environment variables (API_URL, WS_URL)

---

## Phase 4: Deployment ✅

### 4.1 Docker Compose
- [x] Create `docker-compose.yml` with server + web services
- [x] Configure volumes for SQLite persistence
- [x] Setup service dependencies

### 4.2 Documentation
- [ ] Update README with setup instructions
- [ ] Document API endpoints
- [ ] Document client installation steps
- [ ] Add example configuration

---

## Implementation Order

1. **Backend API + Database** ✅ (foundation for everything)
2. **Client Agent** ✅ (start collecting data)
3. **Frontend Dashboard** ✅ (visualize data)
4. **Docker Compose** ✅ (deployment)
