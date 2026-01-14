export interface Client {
  id: string;
  hostname: string;
  last_seen: string;
  version: string | null;
}

export interface Metric {
  id: number;
  client_id: string;
  cpu_usage: number;
  ram_usage: number;
  disk_usage: number;
  inode_usage: number;
  docker_sz: number | null;
  gpu_usage: number | null;
  timestamp: string;
}

export interface Stats {
  client_id: string;
  metric_type: string;
  min: number;
  max: number;
  avg: number;
  p95: number;
  count: number;
}

export interface AlertRule {
  id: number;
  client_id: string | null;
  metric_type: string;
  threshold: number;
  duration_sec: number;
}

export interface WebSocketMessage {
  event: string;
  data: Metric;
}
