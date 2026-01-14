'use client';

import { useQuery } from '@tanstack/react-query';
import { RefreshCw } from 'lucide-react';
import { getClients } from '@/lib/api';
import { ClientCard } from '@/components/client-card';
import { useMetricsPolling } from '@/hooks/useMetricsPolling';

export default function Dashboard() {
  const { data: clients, isLoading, error, refetch } = useQuery({
    queryKey: ['clients'],
    queryFn: getClients,
  });

  const { getLatestMetric, getRecentMetrics, isFetching } = useMetricsPolling(clients);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <RefreshCw className="w-8 h-8 animate-spin text-gray-500" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-900/20 border border-red-800 rounded-lg p-4 text-red-400">
        Failed to load clients. Make sure the server is running.
        <button
          onClick={() => refetch()}
          className="ml-4 underline hover:no-underline"
        >
          Retry
        </button>
      </div>
    );
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-white">Dashboard</h1>
          <p className="text-gray-400">
            {clients?.length || 0} monitored {clients?.length === 1 ? 'server' : 'servers'}
          </p>
        </div>

        <div className="flex items-center gap-4">
          <button
            onClick={() => refetch()}
            className="p-2 rounded-lg bg-gray-800 hover:bg-gray-700 transition-colors"
            title="Refresh"
          >
            <RefreshCw className={`w-4 h-4 ${isFetching ? 'animate-spin' : ''}`} />
          </button>
        </div>
      </div>

      {clients && clients.length > 0 ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {clients.map(client => (
            <ClientCard
              key={client.id}
              client={client}
              latestMetric={getLatestMetric(client.id)}
              recentMetrics={getRecentMetrics(client.id)}
            />
          ))}
        </div>
      ) : (
        <div className="bg-gray-800 rounded-lg p-8 text-center">
          <h2 className="text-xl font-semibold text-white mb-2">No clients registered</h2>
          <p className="text-gray-400 mb-4">
            Register a client to start monitoring your servers.
          </p>
          <code className="block bg-gray-900 rounded p-4 text-sm text-gray-300 text-left max-w-lg mx-auto">
            curl -X POST http://localhost:8080/api/clients \<br />
            {'  '}-H &apos;Content-Type: application/json&apos; \<br />
            {'  '}-d &apos;{'{'}&#34;hostname&#34;: &#34;my-server&#34;{'}'}&apos;
          </code>
        </div>
      )}
    </div>
  );
}
