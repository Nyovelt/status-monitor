# Deployment Guide

This guide explains how to deploy the Status Monitor application using pre-built Docker images from GitHub Container Registry.

## GitHub Container Registry

Docker images are automatically built and published to GitHub Container Registry (GHCR) via GitHub Actions whenever code is pushed to the main branch or when tags are created.

### Available Images

- **Server (Backend)**: `ghcr.io/YOUR_GITHUB_USERNAME/status-monitor-server:latest`
- **Web (Frontend)**: `ghcr.io/YOUR_GITHUB_USERNAME/status-monitor-web:latest`

Replace `YOUR_GITHUB_USERNAME` with your actual GitHub username or organization name.

## Image Tags

The GitHub Actions workflow creates multiple tags for each build:

- `latest` - Latest build from the main branch
- `main` - Latest build from the main branch
- `v1.2.3` - Semantic version tags (when you create a release)
- `v1.2` - Major.minor version
- `v1` - Major version only
- `main-sha-abc123` - Branch name with commit SHA

## Deploying with Portainer

### Step 1: Make Images Public (Optional)

If you want to pull images without authentication, make your GitHub Container Registry packages public:

1. Go to your GitHub repository
2. Click on "Packages" in the right sidebar
3. Click on each package (status-monitor-server and status-monitor-web)
4. Click "Package settings"
5. Scroll down to "Danger Zone" and click "Change visibility"
6. Select "Public"

### Step 2: Deploy with Portainer

1. **Log in to Portainer**

2. **Create a new Stack**:
   - Go to "Stacks" → "Add stack"
   - Name your stack (e.g., "status-monitor")

3. **Use the docker-compose file**:
   - Copy the contents of `docker-compose.production.yml`
   - Replace `YOUR_GITHUB_USERNAME` with your GitHub username
   - Update environment variables as needed (especially API URLs)
   - Paste into the Web editor

4. **Deploy the stack**:
   - Click "Deploy the stack"
   - Wait for the containers to start

### Step 3: Access Your Application

- **Frontend**: http://your-server-ip:3000
- **Backend API**: http://your-server-ip:8080

## Deploying with Docker Compose

If you prefer to use Docker Compose directly:

```bash
# Clone or download docker-compose.production.yml
wget https://raw.githubusercontent.com/YOUR_GITHUB_USERNAME/status-monitor/main/docker-compose.production.yml

# Edit the file to replace YOUR_GITHUB_USERNAME and update environment variables
nano docker-compose.production.yml

# Deploy
docker-compose -f docker-compose.production.yml up -d
```

## Authentication for Private Images

If your images are private, you need to authenticate with GitHub Container Registry:

```bash
# Create a Personal Access Token (PAT) with read:packages scope
# Then login:
echo YOUR_PAT | docker login ghcr.io -u YOUR_GITHUB_USERNAME --password-stdin
```

In Portainer:
1. Go to "Registries" → "Add registry"
2. Select "Custom registry"
3. Registry URL: `ghcr.io`
4. Username: Your GitHub username
5. Password: Your GitHub Personal Access Token (with `read:packages` scope)

## Environment Variables

### Server (Backend)

- `DATABASE_URL`: SQLite database path (default: `sqlite:/app/data/monitor.db`)
- `RUST_LOG`: Logging level (default: `server=info,tower_http=info`)

### Web (Frontend)

- `NEXT_PUBLIC_API_URL`: Backend API URL (e.g., `http://localhost:8080` or `https://api.yourdomain.com`)
- `NEXT_PUBLIC_WS_URL`: WebSocket URL for live updates (e.g., `ws://localhost:8080/ws/live` or `wss://api.yourdomain.com/ws/live`)

**Important**: For production deployments, update these URLs to match your actual domain or server IP.

## Volume Mapping

The server container needs persistent storage for the SQLite database:

```yaml
volumes:
  - ./data:/app/data
```

This maps a local `data` directory to the container's `/app/data` directory where the database is stored.

## Updating Images

To pull the latest images and restart your containers:

```bash
# With Docker Compose
docker-compose -f docker-compose.production.yml pull
docker-compose -f docker-compose.production.yml up -d
```

In Portainer:
1. Go to your stack
2. Click "Update"
3. Enable "Re-pull image and redeploy"
4. Click "Update"

## Building Custom Images

If you need to modify the code and build custom images:

1. Fork the repository
2. Make your changes
3. Push to your fork
4. GitHub Actions will automatically build and publish images to your fork's GHCR

Alternatively, build locally:

```bash
# Build server
docker build -t my-status-monitor-server ./server

# Build web
docker build -t my-status-monitor-web \
  --build-arg NEXT_PUBLIC_API_URL=http://localhost:8080 \
  --build-arg NEXT_PUBLIC_WS_URL=ws://localhost:8080/ws/live \
  ./web
```

## Troubleshooting

### Images won't pull

- Check if images are public or if you're authenticated
- Verify the image name and tag exist: `docker pull ghcr.io/YOUR_GITHUB_USERNAME/status-monitor-server:latest`

### Frontend can't connect to backend

- Ensure `NEXT_PUBLIC_API_URL` and `NEXT_PUBLIC_WS_URL` are correctly set
- Check if the backend is accessible from the frontend container
- For external access, use your server's IP or domain instead of `localhost`

### Database issues

- Ensure the data volume is properly mapped
- Check permissions on the host data directory
- View logs: `docker logs <container-name>`

## Health Checks

The server includes a health check endpoint:

```bash
curl http://localhost:8080/api/clients
```

Docker health checks are configured in the docker-compose file and will automatically restart the container if it becomes unhealthy.

---

## Deploying with Caddy and Cloudflare

For production deployments with automatic HTTPS and Cloudflare integration, use Caddy as a reverse proxy.

### Architecture

```
Internet → Cloudflare → Caddy (HTTPS) → Docker Containers
                                        ├─ Server (8080)
                                        └─ Web (3000)
```

### Prerequisites

1. **Domain name** managed by Cloudflare
2. **Server** with Docker and Docker Compose installed
3. **Cloudflare account** with your domain added

### Step 1: Configure Cloudflare DNS

1. **Log in to Cloudflare Dashboard**

2. **Add DNS Records**:

   **Option A: Separate Subdomains (Recommended)**
   - `status.yourdomain.com` → Your server IP (A record)
   - `api.status.yourdomain.com` → Your server IP (A record)

   **Option B: Single Domain**
   - `status.yourdomain.com` → Your server IP (A record)

3. **SSL/TLS Settings**:
   - Go to SSL/TLS → Overview
   - Set encryption mode to **"Full"** or **"Full (strict)"**
   - This ensures end-to-end encryption between Cloudflare and your server

4. **Proxy Settings**:
   - **Orange cloud icon (Proxied)**: Traffic goes through Cloudflare (DDoS protection, caching)
   - **Gray cloud icon (DNS only)**: Direct to your server (faster, but no Cloudflare protection)
   - Recommended: **Proxied** for production

### Step 2: Configure Caddy

1. **Edit the Caddyfile**:

   ```bash
   nano Caddyfile
   ```

2. **Replace `example.com` with your domain**:

   For separate subdomains:
   ```caddyfile
   status.yourdomain.com {
       reverse_proxy web:3000
       # ... rest of config
   }

   api.status.yourdomain.com {
       reverse_proxy server:8080
       # ... rest of config
   }
   ```

   For single domain, uncomment "Option 2" in the Caddyfile.

### Step 3: Update Docker Compose Configuration

1. **Edit `docker-compose.caddy.yml`**:

   ```bash
   nano docker-compose.caddy.yml
   ```

2. **Update environment variables** in the `web` service:

   For separate subdomains:
   ```yaml
   environment:
     - NEXT_PUBLIC_API_URL=https://api.status.yourdomain.com
     - NEXT_PUBLIC_WS_URL=wss://api.status.yourdomain.com/ws/live
   ```

   For single domain:
   ```yaml
   environment:
     - NEXT_PUBLIC_API_URL=https://status.yourdomain.com/api
     - NEXT_PUBLIC_WS_URL=wss://status.yourdomain.com/ws/live
   ```

### Step 4: Deploy

1. **Start the stack**:

   ```bash
   docker-compose -f docker-compose.caddy.yml up -d
   ```

2. **Check logs**:

   ```bash
   # Check all services
   docker-compose -f docker-compose.caddy.yml logs -f

   # Check Caddy specifically
   docker-compose -f docker-compose.caddy.yml logs -f caddy
   ```

3. **Verify HTTPS certificates**:

   Caddy automatically obtains and renews Let's Encrypt certificates. Check the logs for:
   ```
   certificate obtained successfully
   ```

### Step 5: Access Your Application

- **Frontend**: `https://status.yourdomain.com`
- **Backend API**: `https://api.status.yourdomain.com` (or `/api` path)

### Cloudflare Configuration Tips

#### 1. **Enable HTTP/3** (Recommended)
   - Go to Network settings
   - Enable HTTP/3 (QUIC)
   - Caddy already supports HTTP/3 on port 443/udp

#### 2. **Configure Firewall Rules** (Optional)
   - Create rules to block malicious traffic
   - Example: Block common attack patterns

#### 3. **Enable Caching** (Optional)
   - Go to Caching → Configuration
   - Create page rules to cache static assets
   - Don't cache API endpoints

#### 4. **Configure SSL/TLS Settings**
   - Minimum TLS Version: TLS 1.2
   - Enable Always Use HTTPS
   - Enable Automatic HTTPS Rewrites

#### 5. **WebSocket Support**
   - Cloudflare automatically proxies WebSocket connections
   - No additional configuration needed
   - Ensure SSL/TLS mode is "Full" or "Full (strict)"

### Security Best Practices

1. **Restrict Port Access**:
   ```bash
   # Only allow ports 80, 443 from outside
   # Block direct access to 3000, 8080
   ufw allow 80/tcp
   ufw allow 443/tcp
   ufw enable
   ```

2. **Enable Cloudflare Firewall**:
   - Use Cloudflare's Web Application Firewall (WAF)
   - Enable Bot Fight Mode
   - Configure rate limiting

3. **Secure Headers**:
   - Already configured in Caddyfile (HSTS, X-Frame-Options, etc.)

4. **Regular Updates**:
   ```bash
   # Update images
   docker-compose -f docker-compose.caddy.yml pull
   docker-compose -f docker-compose.caddy.yml up -d
   ```

### Advanced: Cloudflare API Token for DNS Challenge

If you want to use Cloudflare's DNS-01 challenge for certificates (optional):

1. **Create Cloudflare API Token**:
   - Go to Cloudflare Dashboard → My Profile → API Tokens
   - Create Token → Edit zone DNS template
   - Permissions: Zone → DNS → Edit
   - Zone Resources: Include → Specific zone → your domain

2. **Add to docker-compose.caddy.yml**:
   ```yaml
   caddy:
     environment:
       - CLOUDFLARE_API_TOKEN=your_token_here
   ```

3. **Update Caddyfile** to use DNS challenge:
   ```caddyfile
   status.yourdomain.com {
       tls {
           dns cloudflare {env.CLOUDFLARE_API_TOKEN}
       }
       # ... rest of config
   }
   ```

### Troubleshooting Caddy + Cloudflare

#### HTTPS Certificate Issues

**Problem**: Certificate errors or "too many redirects"
**Solution**:
- Ensure Cloudflare SSL/TLS mode is "Full" or "Full (strict)", not "Flexible"
- Check Caddy logs: `docker logs status-monitor-caddy`
- Verify DNS records are correct and propagated

#### WebSocket Connection Fails

**Problem**: Live updates not working (WebSocket connection fails)
**Solution**:
- Verify WebSocket URL uses `wss://` (secure WebSocket)
- Check Cloudflare is proxying WebSocket connections (should be automatic)
- Ensure Caddyfile has WebSocket handling configuration
- Check browser console for connection errors

#### 502 Bad Gateway

**Problem**: Caddy shows 502 error
**Solution**:
- Check if backend containers are running: `docker ps`
- Verify container names match in Caddyfile (`web:3000`, `server:8080`)
- Check backend health: `docker-compose -f docker-compose.caddy.yml exec server curl http://localhost:8080/api/clients`
- Review logs: `docker-compose -f docker-compose.caddy.yml logs`

#### Cloudflare Shows Origin Unreachable

**Problem**: Cloudflare can't reach your server
**Solution**:
- Verify server IP is correct in DNS records
- Check server firewall allows ports 80 and 443
- Ensure Caddy is running and bound to correct ports
- Test direct connection: `curl -I http://YOUR_SERVER_IP`

### Monitoring and Logs

**View Caddy access logs**:
```bash
docker-compose -f docker-compose.caddy.yml exec caddy tail -f /var/log/caddy/status-access.log
docker-compose -f docker-compose.caddy.yml exec caddy tail -f /var/log/caddy/api-access.log
```

**View application logs**:
```bash
# Server logs
docker logs status-monitor-server -f

# Web logs
docker logs status-monitor-web -f
```

**Check Cloudflare Analytics**:
- Go to Analytics & Logs in Cloudflare Dashboard
- Monitor traffic, threats blocked, and performance

