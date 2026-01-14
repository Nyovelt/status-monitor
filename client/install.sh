#!/bin/bash
set -e

# Status Monitor Client Install Script

INSTALL_DIR="/usr/local/bin"
SERVICE_FILE="/etc/systemd/system/status-monitor-client.service"
ENV_FILE="/etc/status-monitor-client.env"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Status Monitor Client Installer${NC}"
echo "================================"

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: Please run as root (sudo)${NC}"
    exit 1
fi

# Check for required arguments
if [ -z "$1" ] || [ -z "$2" ]; then
    echo -e "${YELLOW}Usage: $0 <SERVER_URL> <CLIENT_TOKEN>${NC}"
    echo ""
    echo "Example:"
    echo "  sudo ./install.sh http://monitor.example.com:8080 abc123-token"
    echo ""
    echo "To get a token, create a client on the server:"
    echo "  curl -X POST http://server:8080/api/clients -H 'Content-Type: application/json' -d '{\"hostname\": \"$(hostname)\"}'"
    exit 1
fi

SERVER_URL="$1"
CLIENT_TOKEN="$2"

# Build release binary if not exists
BINARY_PATH="./target/release/status-monitor-client"
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${YELLOW}Building release binary...${NC}"
    cargo build --release
fi

# Stop existing service if running
if systemctl is-active --quiet status-monitor-client; then
    echo "Stopping existing service..."
    systemctl stop status-monitor-client
fi

# Install binary
echo "Installing binary to ${INSTALL_DIR}..."
cp "$BINARY_PATH" "${INSTALL_DIR}/status-monitor-client"
chmod +x "${INSTALL_DIR}/status-monitor-client"

# Create environment file
echo "Creating environment file..."
cat > "$ENV_FILE" << EOF
SERVER_URL=${SERVER_URL}
CLIENT_TOKEN=${CLIENT_TOKEN}
DOCKER_PATH=/var/lib/docker
# GITHUB_REPO=username/status-monitor  # Uncomment to enable auto-updates
EOF
chmod 600 "$ENV_FILE"

# Install systemd service
echo "Installing systemd service..."
cp status-monitor-client.service "$SERVICE_FILE"

# Reload systemd
systemctl daemon-reload

# Enable and start service
echo "Enabling and starting service..."
systemctl enable status-monitor-client
systemctl start status-monitor-client

# Check status
sleep 2
if systemctl is-active --quiet status-monitor-client; then
    echo -e "${GREEN}✓ Status Monitor Client installed and running!${NC}"
    echo ""
    echo "Useful commands:"
    echo "  View logs:     journalctl -u status-monitor-client -f"
    echo "  Check status:  systemctl status status-monitor-client"
    echo "  Stop service:  systemctl stop status-monitor-client"
    echo "  Edit config:   nano /etc/status-monitor-client.env"
else
    echo -e "${RED}✗ Service failed to start. Check logs:${NC}"
    journalctl -u status-monitor-client --no-pager -n 20
    exit 1
fi
