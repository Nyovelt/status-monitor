# Deployment Guide: Host-based Caddy with Cloudflare

This guide is for deploying Status Monitor with Caddy running on the host (not in Docker) and using Cloudflare DNS.

**Your Configuration:**
- Domain: `swat-status.aaaab3n.moe`
- Backend: `localhost:32022` → `/api` path
- Frontend: `localhost:32023` → root path
- Caddy: Running on host

## Step 1: Configure Cloudflare DNS

1. Log in to your Cloudflare dashboard

2. Add DNS record for your domain:
   ```
   Type: A
   Name: swat-status (or @ if using root domain)
   Content: YOUR_SERVER_IP
   Proxy status: Proxied (orange cloud) ✓
   ```

3. SSL/TLS Settings:
   - Go to SSL/TLS → Overview
   - Set encryption mode to: **"Full (strict)"** or **"Full"**
   - Enable: "Always Use HTTPS"
   - Enable: "Automatic HTTPS Rewrites"

## Step 2: Configure Caddy on Host

1. **Copy the Caddyfile to your Caddy config directory:**

   ```bash
   sudo cp Caddyfile.host /etc/caddy/Caddyfile
   # OR append to existing Caddyfile:
   sudo cat Caddyfile.host >> /etc/caddy/Caddyfile
   ```

2. **Create log directory (if it doesn't exist):**

   ```bash
   sudo mkdir -p /var/log/caddy
   sudo chown caddy:caddy /var/log/caddy
   ```

3. **Validate Caddy configuration:**

   ```bash
   sudo caddy validate --config /etc/caddy/Caddyfile
   ```

4. **Reload Caddy:**

   ```bash
   # If using systemd:
   sudo systemctl reload caddy

   # OR if running Caddy manually:
   sudo caddy reload --config /etc/caddy/Caddyfile
   ```

5. **Check Caddy status:**

   ```bash
   sudo systemctl status caddy
   # OR check logs:
   sudo journalctl -u caddy -f
   ```

## Step 3: Deploy Docker Containers

### Option A: Using Pre-built Images (Recommended)

```bash
# Pull and deploy with production configuration
docker-compose -f docker-compose.production-caddy.yml pull
docker-compose -f docker-compose.production-caddy.yml up -d
```

### Option B: Building from Source

If you want to build locally instead of using GHCR images:

```bash
# Use your existing docker-compose but update environment variables
docker-compose up -d --build
```

**Important:** Update your existing `docker-compose.yml` environment variables:

```yaml
environment:
  # Change from http://localhost to https with your domain
  - NEXT_PUBLIC_API_URL=https://swat-status.aaaab3n.moe/api
  - NEXT_PUBLIC_WS_URL=wss://swat-status.aaaab3n.moe/ws/live
```

## Step 4: Verify Deployment

1. **Check Docker containers are running:**

   ```bash
   docker-compose ps
   ```

   You should see both `server` and `web` services running.

2. **Test backend locally:**

   ```bash
   curl http://localhost:32022/api/clients
   ```

3. **Test frontend locally:**

   ```bash
   curl http://localhost:32023
   ```

4. **Test through Caddy (HTTPS):**

   ```bash
   curl https://swat-status.aaaab3n.moe
   curl https://swat-status.aaaab3n.moe/api/clients
   ```

5. **Check Caddy logs:**

   ```bash
   sudo tail -f /var/log/caddy/swat-status-access.log
   ```

## Step 5: Access Your Application

Open your browser and navigate to:
- **Application**: https://swat-status.aaaab3n.moe

Caddy will automatically obtain and manage Let's Encrypt SSL certificates.

## Security Considerations

### 1. Firewall Configuration

Since Caddy is handling external access, you should **only allow Caddy ports** externally:

```bash
# If using ufw:
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw allow 443/udp  # HTTP/3

# Block direct access to application ports from outside
# (They're already bound to 127.0.0.1, so this is extra safety)
sudo ufw deny 32022
sudo ufw deny 32023

sudo ufw enable
```

### 2. Port Binding

The `docker-compose.production-caddy.yml` binds ports to `127.0.0.1` only:

```yaml
ports:
  - "127.0.0.1:32022:8080"  # Only accessible from localhost
  - "127.0.0.1:32023:3000"
```

This prevents direct external access to your containers - all traffic must go through Caddy.

### 3. Cloudflare Security Features

Enable these in Cloudflare Dashboard:

1. **Firewall Rules**: Block common attack patterns
2. **Rate Limiting**: Prevent abuse
3. **Bot Fight Mode**: Block malicious bots
4. **DDoS Protection**: Enabled by default with proxied DNS

## Updating

### Update Docker Images

```bash
# Pull latest images
docker-compose -f docker-compose.production-caddy.yml pull

# Restart with new images
docker-compose -f docker-compose.production-caddy.yml up -d
```

### Update Caddy Configuration

```bash
# Edit Caddyfile
sudo nano /etc/caddy/Caddyfile

# Validate
sudo caddy validate --config /etc/caddy/Caddyfile

# Reload (zero downtime)
sudo systemctl reload caddy
```

## Troubleshooting

### Issue: 502 Bad Gateway

**Cause**: Docker containers aren't running or not accessible

**Solution**:
```bash
# Check container status
docker-compose ps

# Check container logs
docker-compose logs server
docker-compose logs web

# Restart containers
docker-compose restart
```

### Issue: WebSocket connection fails

**Cause**: WebSocket proxy not configured correctly

**Solution**:
- Verify Caddyfile has `@websockets` section
- Check WebSocket URL in frontend uses `wss://` (secure)
- Ensure Cloudflare is proxying WebSocket (automatic with Full SSL mode)

### Issue: Certificate errors

**Cause**: Cloudflare SSL/TLS mode misconfigured

**Solution**:
- Ensure Cloudflare SSL/TLS mode is "Full" or "Full (strict)", NOT "Flexible"
- Check Caddy logs: `sudo journalctl -u caddy -f`
- Wait a few minutes for certificate to be issued

### Issue: "Too many redirects"

**Cause**: Cloudflare SSL mode set to "Flexible"

**Solution**:
- Change Cloudflare SSL/TLS mode to "Full" or "Full (strict)"

## Monitoring

### View Application Logs

```bash
# Docker container logs
docker-compose logs -f server
docker-compose logs -f web

# Caddy access logs
sudo tail -f /var/log/caddy/swat-status-access.log

# Caddy error logs
sudo journalctl -u caddy -f
```

### Monitor Resource Usage

```bash
# Docker stats
docker stats

# Disk usage
docker system df
```

## Backup

Important files to backup:

```bash
# Database
./data/

# Caddy configuration
/etc/caddy/Caddyfile

# Caddy SSL certificates (auto-renewed, but good to backup)
/var/lib/caddy/
```

## Rolling Back

If you need to roll back to a previous version:

```bash
# Use a specific tag
docker-compose -f docker-compose.production-caddy.yml pull
# Edit the file to use a specific tag like :v1.0.0 instead of :latest
docker-compose -f docker-compose.production-caddy.yml up -d
```

---

## Quick Reference

### Useful Commands

```bash
# Start services
docker-compose -f docker-compose.production-caddy.yml up -d

# Stop services
docker-compose -f docker-compose.production-caddy.yml down

# View logs
docker-compose -f docker-compose.production-caddy.yml logs -f

# Restart services
docker-compose -f docker-compose.production-caddy.yml restart

# Update images
docker-compose -f docker-compose.production-caddy.yml pull
docker-compose -f docker-compose.production-caddy.yml up -d

# Reload Caddy
sudo systemctl reload caddy

# View Caddy logs
sudo journalctl -u caddy -f
```

### File Locations

- Caddy config: `/etc/caddy/Caddyfile`
- Caddy logs: `/var/log/caddy/`
- Caddy data: `/var/lib/caddy/`
- Application data: `./data/`
- Docker compose: `./docker-compose.production-caddy.yml`

---

Need help? Check the main [DEPLOYMENT.md](./DEPLOYMENT.md) for more details or open an issue on GitHub.
