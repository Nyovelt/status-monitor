'use client';

import { useCallback } from 'react';
import { useQueries } from '@tanstack/react-query';
import { getLatestMetrics } from '@/lib/api';
import { Client, Metric } from '@/types';

const POLLING_INTERVAL = 5000; // 5 seconds

export function useMetricsPolling(clients: Client[] | undefined) {
  const clientIds = clients?.map(c => c.id) ?? [];

  const metricsQueries = useQueries({
    queries: clientIds.map(clientId => ({
      queryKey: ['latestMetrics', clientId],
      queryFn: () => getLatestMetrics(clientId),
      refetchInterval: POLLING_INTERVAL,
      staleTime: POLLING_INTERVAL - 1000,
    })),
  });

  const getLatestMetric = useCallback((clientId: string): Metric | undefined => {
    const index = clientIds.indexOf(clientId);
    if (index === -1) return undefined;
    const query = metricsQueries[index];
    if (!query?.data?.length) return undefined;
    return query.data[query.data.length - 1];
  }, [clientIds, metricsQueries]);

  const getRecentMetrics = useCallback((clientId: string): Metric[] => {
    const index = clientIds.indexOf(clientId);
    if (index === -1) return [];
    return metricsQueries[index]?.data ?? [];
  }, [clientIds, metricsQueries]);

  const isLoading = metricsQueries.some(q => q.isLoading);
  const isFetching = metricsQueries.some(q => q.isFetching);

  return {
    getLatestMetric,
    getRecentMetrics,
    isLoading,
    isFetching,
  };
}
