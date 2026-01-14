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
