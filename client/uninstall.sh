#!/bin/bash
set -e

# Status Monitor Client Uninstall Script

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

echo -e "${RED}Status Monitor Client Uninstaller${NC}"
echo "=================================="

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: Please run as root (sudo)${NC}"
    exit 1
fi

# Stop and disable service
if systemctl is-active --quiet status-monitor-client; then
    echo "Stopping service..."
    systemctl stop status-monitor-client
fi

if systemctl is-enabled --quiet status-monitor-client 2>/dev/null; then
    echo "Disabling service..."
    systemctl disable status-monitor-client
fi

# Remove files
echo "Removing files..."
rm -f /usr/local/bin/status-monitor-client
rm -f /etc/systemd/system/status-monitor-client.service
rm -f /etc/status-monitor-client.env

# Reload systemd
systemctl daemon-reload

echo -e "${GREEN}✓ Status Monitor Client uninstalled!${NC}"
