'use client';

import { useCallback, useEffect, useState } from 'react';
import useWebSocket, { ReadyState } from 'react-use-websocket';
import { Metric, WebSocketMessage } from '@/types';

const WS_URL = process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:8080/ws/live';

export function useMetricsWebSocket() {
  const [latestMetrics, setLatestMetrics] = useState<Map<string, Metric>>(new Map());
  const [recentMetrics, setRecentMetrics] = useState<Map<string, Metric[]>>(new Map());

  const { lastMessage, readyState } = useWebSocket(WS_URL, {
    shouldReconnect: () => true,
    reconnectAttempts: 10,
    reconnectInterval: 3000,
  });

  useEffect(() => {
    if (lastMessage !== null) {
      try {
        const message: WebSocketMessage = JSON.parse(lastMessage.data);
        if (message.event === 'metric') {
          const metric = message.data;

          // Update latest metric for client
          setLatestMetrics(prev => {
            const next = new Map(prev);
            next.set(metric.client_id, metric);
            return next;
          });

          // Update recent metrics (keep last 60 for sparklines)
          setRecentMetrics(prev => {
            const next = new Map(prev);
            const clientMetrics = next.get(metric.client_id) || [];
            const updated = [...clientMetrics, metric].slice(-60);
            next.set(metric.client_id, updated);
            return next;
          });
        }
      } catch (e) {
        console.error('Failed to parse WebSocket message:', e);
      }
    }
  }, [lastMessage]);

  const getLatestMetric = useCallback((clientId: string) => {
    return latestMetrics.get(clientId);
  }, [latestMetrics]);

  const getRecentMetrics = useCallback((clientId: string) => {
    return recentMetrics.get(clientId) || [];
  }, [recentMetrics]);

  const connectionStatus = {
    [ReadyState.CONNECTING]: 'Connecting',
    [ReadyState.OPEN]: 'Connected',
    [ReadyState.CLOSING]: 'Closing',
    [ReadyState.CLOSED]: 'Disconnected',
    [ReadyState.UNINSTANTIATED]: 'Uninstantiated',
  }[readyState];

  return {
    latestMetrics,
    recentMetrics,
    getLatestMetric,
    getRecentMetrics,
    connectionStatus,
    isConnected: readyState === ReadyState.OPEN,
  };
}
