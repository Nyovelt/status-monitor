# status-monitor

This project contains three parts: a web frontend that demos all stats, a server that serve the backend, and a client that report the metrics.

## Quick Start

### Deploy with Pre-built Images (Recommended)

The easiest way to deploy is using pre-built Docker images from GitHub Container Registry:

```bash
docker-compose up -d
```

This will pull and run:
- **Server**: `ghcr.io/nyovelt/status-monitor-server:latest`
- **Web**: `ghcr.io/nyovelt/status-monitor-web:latest`

Access the application:
- Frontend: http://localhost:3000
- Backend API: http://localhost:8080

### Build from Source (Development)

For local development with live code changes:

```bash
docker-compose -f docker-compose.build.yml up -d
```

### Deploy with Portainer

See [DEPLOYMENT.md](./DEPLOYMENT.md) for detailed Portainer deployment instructions and configuration options.

## Architecture

### Web frontend
Features:
- hostname
- CPU usage
- GPU usage
- disk usage
- inode
- docker data root
- Historical data and statistics (averages)

### Web backend
- Serves the frontend and collects metrics from clients
- Client management with token-based authentication
- SQLite database for metrics storage

### Client
- Reports system metrics to the backend
- Runs as systemd service
- Configurable reporting interval

## Technology Stack

- **Frontend**: React, Next.js, TypeScript
- **Backend**: Rust (Axum, SQLx, Tokio)
- **Client**: Rust
- **Database**: SQLite
