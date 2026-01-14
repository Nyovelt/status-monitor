# Client Setup Guide

This guide explains how to add and configure monitoring clients for your Status Monitor system.

## Overview

The Status Monitor system has two main components for client monitoring:

1. **Server (Backend)**: Manages clients and collects metrics
2. **Client Agent**: Runs on monitored machines and reports metrics

## Table of Contents

- [Adding a Client](#adding-a-client)
  - [Method 1: Web UI](#method-1-web-ui-recommended)
  - [Method 2: API (curl)](#method-2-api-curl)
- [Installing the Client Agent](#installing-the-client-agent)
  - [Prerequisites](#prerequisites)
  - [Installation Steps](#installation-steps)
- [Configuration](#configuration)
- [Verification](#verification)
- [Managing Clients](#managing-clients)
- [Troubleshooting](#troubleshooting)

---

## Adding a Client

Before installing the client agent on a machine, you need to register it with the server and obtain an authentication token.

### Method 1: Web UI (Recommended)

1. **Open the Status Monitor web interface**:
   ```
   https://swat-status.aaaab3n.moe
   ```

2. **Navigate to Settings** (or Clients page)

3. **Click "Add Client" button**

4. **Enter the hostname**:
   - Use the hostname of the machine you want to monitor
   - Example: `web-server-01`, `database-prod`, etc.

5. **Save** and **copy the generated token**
   - ⚠️ **Important**: Save this token immediately! It's only shown once.
   - You'll need this token to configure the client agent

### Method 2: API (curl)

Create a new client using the API:

```bash
# Replace with your server URL
SERVER_URL="https://swat-status.aaaab3n.moe"

# Replace with your desired hostname
HOSTNAME="my-server-name"

# Create the client
curl -X POST "${SERVER_URL}/api/clients" \
  -H "Content-Type: application/json" \
  -d "{\"hostname\": \"${HOSTNAME}\"}"
```

**Response**:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "hostname": "my-server-name",
  "token": "abc123def456ghi789jkl012mno345pqr678stu901vwx234yz"
}
```

⚠️ **Save the token** - you'll need it for the client installation!

---

## Installing the Client Agent

The client agent is a Rust application that runs as a systemd service and reports metrics to your server.

### Prerequisites

**On the machine you want to monitor:**

1. **Rust toolchain** (for building from source):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **System requirements**:
   - Linux with systemd
   - Root/sudo access
   - Network access to your Status Monitor server

3. **Optional**: Docker (if you want to monitor Docker disk usage)

### Installation Steps

#### Step 1: Clone the Repository

```bash
# On the machine to be monitored
git clone https://github.com/Nyovelt/status-monitor.git
cd status-monitor/client
```

#### Step 2: Run the Install Script

The install script builds the client, installs it as a systemd service, and starts monitoring.

```bash
# Syntax: sudo ./install.sh <SERVER_URL> <CLIENT_TOKEN>

# Example:
sudo ./install.sh https://swat-status.aaaab3n.moe abc123def456ghi789jkl012mno345pqr678stu901vwx234yz
```

**Parameters**:
- `SERVER_URL`: Your Status Monitor server URL (include https:// and port if needed)
- `CLIENT_TOKEN`: The token you received when creating the client

The script will:
1. Build the client in release mode
2. Install the binary to `/usr/local/bin/status-monitor-client`
3. Create configuration at `/etc/status-monitor-client.env`
4. Install systemd service at `/etc/systemd/system/status-monitor-client.service`
5. Enable and start the service

#### Step 3: Verify Installation

```bash
# Check service status
sudo systemctl status status-monitor-client

# View logs
sudo journalctl -u status-monitor-client -f
```

You should see output like:
```
status-monitor-client.service - Status Monitor Client
     Loaded: loaded (/etc/systemd/system/status-monitor-client.service; enabled)
     Active: active (running)
```

---

## Configuration

### Environment Variables

Configuration is stored in `/etc/status-monitor-client.env`:

```bash
# Edit configuration
sudo nano /etc/status-monitor-client.env
```

**Available settings**:

```bash
# Server connection
SERVER_URL=https://swat-status.aaaab3n.moe
CLIENT_TOKEN=your_token_here

# Docker monitoring (optional)
DOCKER_PATH=/var/lib/docker

# Reporting interval in seconds (default: 60)
REPORT_INTERVAL=60

# Auto-update configuration (optional)
# GITHUB_REPO=Nyovelt/status-monitor
```

After editing, restart the service:
```bash
sudo systemctl restart status-monitor-client
```

### Metrics Collected

The client reports the following metrics every 60 seconds (default):

| Metric | Description |
|--------|-------------|
| **CPU Usage** | Overall CPU utilization percentage |
| **RAM Usage** | Memory usage percentage |
| **Disk Usage** | Root filesystem usage percentage |
| **Inode Usage** | Inode usage percentage |
| **Docker Size** | Size of `/var/lib/docker` (if Docker is installed) |
| **GPU Usage** | GPU utilization (if NVIDIA GPU detected) |

### Monitoring Docker Disk Usage

The client can monitor Docker's disk usage:

1. **Ensure the client has access** to `/var/lib/docker`:
   ```bash
   # The service already runs as root, so this should work by default
   ls -la /var/lib/docker
   ```

2. **Configure the path** if Docker uses a different location:
   ```bash
   sudo nano /etc/status-monitor-client.env
   # Set: DOCKER_PATH=/custom/docker/path
   ```

3. **Restart the service**:
   ```bash
   sudo systemctl restart status-monitor-client
   ```

---

## Verification

### Check Client Status on Server

#### Via Web UI:
1. Open https://swat-status.aaaab3n.moe
2. You should see your client listed with recent metrics

#### Via API:
```bash
# List all clients
curl https://swat-status.aaaab3n.moe/api/clients

# Get specific client details
curl https://swat-status.aaaab3n.moe/api/clients/{client_id}

# Get recent metrics
curl https://swat-status.aaaab3n.moe/api/metrics/{client_id}/recent
```

### Check Client Logs

```bash
# Real-time logs
sudo journalctl -u status-monitor-client -f

# Last 50 lines
sudo journalctl -u status-monitor-client -n 50

# Logs from today
sudo journalctl -u status-monitor-client --since today
```

**Healthy logs** should show:
```
Successfully reported metrics to server
CPU: 15.2%, RAM: 45.8%, Disk: 32.1%
```

---

## Managing Clients

### Common Operations

#### Stop the Client
```bash
sudo systemctl stop status-monitor-client
```

#### Start the Client
```bash
sudo systemctl start status-monitor-client
```

#### Restart the Client
```bash
sudo systemctl restart status-monitor-client
```

#### Disable Auto-start
```bash
sudo systemctl disable status-monitor-client
```

#### Enable Auto-start
```bash
sudo systemctl enable status-monitor-client
```

#### Uninstall the Client

```bash
# Stop and disable service
sudo systemctl stop status-monitor-client
sudo systemctl disable status-monitor-client

# Remove files
sudo rm /usr/local/bin/status-monitor-client
sudo rm /etc/systemd/system/status-monitor-client.service
sudo rm /etc/status-monitor-client.env

# Reload systemd
sudo systemctl daemon-reload
```

### Remove Client from Server

#### Via Web UI:
1. Go to Settings or Clients page
2. Click "Delete" on the client you want to remove
3. Confirm deletion

#### Via API:
```bash
# Replace {client_id} with the actual client ID
curl -X DELETE https://swat-status.aaaab3n.moe/api/clients/{client_id}
```

---

## Troubleshooting

### Client Not Appearing on Dashboard

**Check if the service is running**:
```bash
sudo systemctl status status-monitor-client
```

**Check logs for errors**:
```bash
sudo journalctl -u status-monitor-client -n 50
```

**Common issues**:
1. **Wrong server URL**: Verify `SERVER_URL` in `/etc/status-monitor-client.env`
2. **Invalid token**: Check `CLIENT_TOKEN` is correct
3. **Network connectivity**: Test connection to server:
   ```bash
   curl https://swat-status.aaaab3n.moe/api/clients
   ```
4. **SSL certificate issues**: Ensure system has updated CA certificates:
   ```bash
   sudo update-ca-certificates
   ```

### Metrics Not Updating

**Check reporting frequency**:
```bash
# Logs should show reports every 60 seconds by default
sudo journalctl -u status-monitor-client -f
```

**Increase verbosity**:
```bash
# Edit the service file to add more logging
sudo nano /etc/systemd/system/status-monitor-client.service
# Add: Environment="RUST_LOG=debug"
sudo systemctl daemon-reload
sudo systemctl restart status-monitor-client
```

### Permission Denied Errors

**Docker path access**:
```bash
# Ensure service runs as root
sudo systemctl cat status-monitor-client | grep User
# Should show: User=root
```

**Fix permissions**:
```bash
# If needed, adjust permissions
sudo chown -R root:root /var/lib/docker
```

### High CPU/Memory Usage

The client is designed to be lightweight, but if you notice high resource usage:

1. **Check reporting interval**:
   ```bash
   # Increase interval to reduce frequency
   sudo nano /etc/status-monitor-client.env
   # Set: REPORT_INTERVAL=120  # Report every 2 minutes
   ```

2. **Check for errors in logs**:
   ```bash
   sudo journalctl -u status-monitor-client | grep -i error
   ```

### Connection Refused

**Verify server is accessible**:
```bash
# Test HTTPS connection
curl -I https://swat-status.aaaab3n.moe

# Test API endpoint
curl https://swat-status.aaaab3n.moe/api/clients
```

**Check firewall**:
```bash
# Ensure outbound HTTPS (443) is allowed
sudo ufw status
```

### Token Authentication Failed

**Error**: `401 Unauthorized` or `Invalid token`

**Solution**:
1. Verify token in `/etc/status-monitor-client.env` matches the one from server
2. Check if client was deleted from server
3. Re-create the client and update token:
   ```bash
   sudo nano /etc/status-monitor-client.env
   # Update CLIENT_TOKEN=new_token
   sudo systemctl restart status-monitor-client
   ```

---

## Multiple Clients

To monitor multiple machines:

1. **On the server**: Create a client for each machine (each gets a unique token)
2. **On each machine**: Install the client agent with its corresponding token
3. **View all clients**: Check the dashboard to see all monitored machines

### Example: Monitoring 3 Servers

**Server-side (create clients)**:
```bash
# Create client for web server
curl -X POST https://swat-status.aaaab3n.moe/api/clients \
  -H "Content-Type: application/json" \
  -d '{"hostname": "web-prod-01"}'
# Save token: token1

# Create client for database server
curl -X POST https://swat-status.aaaab3n.moe/api/clients \
  -H "Content-Type: application/json" \
  -d '{"hostname": "db-prod-01"}'
# Save token: token2

# Create client for cache server
curl -X POST https://swat-status.aaaab3n.moe/api/clients \
  -H "Content-Type: application/json" \
  -d '{"hostname": "cache-prod-01"}'
# Save token: token3
```

**Client-side (install on each machine)**:
```bash
# On web-prod-01
sudo ./install.sh https://swat-status.aaaab3n.moe token1

# On db-prod-01
sudo ./install.sh https://swat-status.aaaab3n.moe token2

# On cache-prod-01
sudo ./install.sh https://swat-status.aaaab3n.moe token3
```

---

## Advanced Configuration

### Custom Systemd Service

Edit the service file for advanced configuration:

```bash
sudo nano /etc/systemd/system/status-monitor-client.service
```

Example customizations:
```ini
[Service]
# Run under different user (not recommended for Docker monitoring)
User=monitoring

# Set resource limits
MemoryMax=100M
CPUQuota=10%

# Restart policy
Restart=always
RestartSec=10s

# Additional environment variables
Environment="RUST_LOG=info"
Environment="RUST_BACKTRACE=1"
```

After editing:
```bash
sudo systemctl daemon-reload
sudo systemctl restart status-monitor-client
```

---

## Next Steps

After setting up your clients:

1. **Configure alerts**: Set up alert rules in the Settings page
2. **Monitor metrics**: View real-time and historical data on the dashboard
3. **Optimize thresholds**: Adjust alert thresholds based on your baseline metrics
4. **Scale**: Add more clients as needed

For more information, see:
- [DEPLOYMENT.md](./DEPLOYMENT.md) - Server deployment guide
- [README.md](./README.md) - Project overview
- [DEPLOY-CADDY-HOST.md](./DEPLOY-CADDY-HOST.md) - Caddy configuration

---

## Quick Reference

### Client Creation (API)
```bash
curl -X POST https://swat-status.aaaab3n.moe/api/clients \
  -H "Content-Type: application/json" \
  -d '{"hostname": "my-server"}'
```

### Client Installation
```bash
sudo ./install.sh https://swat-status.aaaab3n.moe <TOKEN>
```

### Client Management
```bash
# Status
sudo systemctl status status-monitor-client

# Logs
sudo journalctl -u status-monitor-client -f

# Restart
sudo systemctl restart status-monitor-client

# Edit config
sudo nano /etc/status-monitor-client.env
```

### API Endpoints
- `GET /api/clients` - List all clients
- `POST /api/clients` - Create new client
- `GET /api/clients/{id}` - Get client details
- `DELETE /api/clients/{id}` - Delete client
- `GET /api/metrics/{client_id}/recent` - Get recent metrics
