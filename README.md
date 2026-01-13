# status-monitor

This project contains three parts: a web frontend that demos all stats, a server that serve the backend, and a client that report the metrics.

## Web frontend
Need to demo:
- hostname
- CPU usage
- GPU usage
- disk usage
- inode
- docker data root 
- and all these need to show historical and stats data (like average)

## Web backend
- Serves the frontend, and collects metrics from backend
- Can add / remove clients (with tokens)

## Client
- Report metrics
- systemd service

## Other:
- frontend: react, nextjs, typescript
- backend and client: Rust
