import { Client, Metric, Stats, AlertRule } from '@/types';

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

async function fetchAPI<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const response = await fetch(`${API_URL}${endpoint}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });

  if (!response.ok) {
    throw new Error(`API error: ${response.status}`);
  }

  return response.json();
}

// Clients
export async function getClients(): Promise<Client[]> {
  return fetchAPI<Client[]>('/api/clients');
}

export async function getClient(id: string): Promise<Client> {
  return fetchAPI<Client>(`/api/clients/${id}`);
}

export async function createClient(hostname: string): Promise<{ id: string; hostname: string; token: string }> {
  return fetchAPI('/api/clients', {
    method: 'POST',
    body: JSON.stringify({ hostname }),
  });
}

export async function deleteClient(id: string): Promise<void> {
  await fetch(`${API_URL}/api/clients/${id}`, { method: 'DELETE' });
}

// Metrics
export async function getMetrics(clientId: string, hours?: number): Promise<Metric[]> {
  const params = hours ? `?hours=${hours}` : '';
  return fetchAPI<Metric[]>(`/api/metrics/${clientId}${params}`);
}

export async function getLatestMetrics(clientId: string): Promise<Metric[]> {
  return fetchAPI<Metric[]>(`/api/metrics/${clientId}/latest`);
}

// Stats
export async function getStats(clientId: string, hours?: number): Promise<Stats[]> {
  const params = hours ? `?hours=${hours}` : '';
  return fetchAPI<Stats[]>(`/api/stats/${clientId}${params}`);
}

// Settings
export async function getSettings(): Promise<Record<string, string>> {
  return fetchAPI<Record<string, string>>('/api/settings');
}

export async function updateSettings(settings: Record<string, string>): Promise<void> {
  await fetch(`${API_URL}/api/settings`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(settings),
  });
}

// Alert Rules
export async function getAlertRules(): Promise<AlertRule[]> {
  return fetchAPI<AlertRule[]>('/api/alerts');
}

export async function createAlertRule(rule: Omit<AlertRule, 'id'>): Promise<AlertRule> {
  return fetchAPI<AlertRule>('/api/alerts', {
    method: 'POST',
    body: JSON.stringify(rule),
  });
}

export async function deleteAlertRule(id: number): Promise<void> {
  await fetch(`${API_URL}/api/alerts/${id}`, { method: 'DELETE' });
}
