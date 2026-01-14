import { formatDistanceToNow, parseISO } from 'date-fns';

export function formatBytes(bytes: number | null | undefined): string {
  if (bytes === null || bytes === undefined) return 'N/A';

  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let unitIndex = 0;
  let value = bytes;

  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex++;
  }

  return `${value.toFixed(1)} ${units[unitIndex]}`;
}

export function formatPercentage(value: number | null | undefined): string {
  if (value === null || value === undefined) return 'N/A';
  return `${value.toFixed(1)}%`;
}

export function formatTimeAgo(timestamp: string): string {
  try {
    return formatDistanceToNow(parseISO(timestamp), { addSuffix: true });
  } catch {
    return 'Unknown';
  }
}

export function getStatusColor(value: number, thresholds = { warning: 70, critical: 90 }): string {
  if (value >= thresholds.critical) return 'text-red-500';
  if (value >= thresholds.warning) return 'text-yellow-500';
  return 'text-green-500';
}

export function getStatusBgColor(value: number, thresholds = { warning: 70, critical: 90 }): string {
  if (value >= thresholds.critical) return 'bg-red-500';
  if (value >= thresholds.warning) return 'bg-yellow-500';
  return 'bg-green-500';
}

export function isClientOnline(lastSeen: string, thresholdSeconds = 60): boolean {
  try {
    const lastSeenDate = parseISO(lastSeen);
    const now = new Date();
    const diffSeconds = (now.getTime() - lastSeenDate.getTime()) / 1000;
    return diffSeconds < thresholdSeconds;
  } catch {
    return false;
  }
}

export function cn(...classes: (string | undefined | false)[]): string {
  return classes.filter(Boolean).join(' ');
}
