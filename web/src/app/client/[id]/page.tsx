'use client';

import { useState } from 'react';
import { useParams } from 'next/navigation';
import Link from 'next/link';
import { useQuery } from '@tanstack/react-query';
import { ArrowLeft, RefreshCw } from 'lucide-react';
import { getClient, getMetrics, getStats, getLatestMetrics } from '@/lib/api';
import { MetricGauge } from '@/components/metric-gauge';
import { HistoryChart } from '@/components/history-chart';
import { StatsTable } from '@/components/stats-table';
import { formatTimeAgo, formatBytes, isClientOnline, cn } from '@/lib/utils';

type Tab = 'live' | 'history' | 'stats';
type TimeRange = 12 | 24 | 168; // hours

export default function ClientDetailPage() {
  const params = useParams();
  const clientId = params.id as string;

  const [activeTab, setActiveTab] = useState<Tab>('live');
  const [timeRange, setTimeRange] = useState<TimeRange>(24);

  const { data: client, isLoading: clientLoading } = useQuery({
    queryKey: ['client', clientId],
    queryFn: () => getClient(clientId),
  });

  const { data: metrics, isLoading: metricsLoading } = useQuery({
    queryKey: ['metrics', clientId, timeRange],
    queryFn: () => getMetrics(clientId, timeRange),
    enabled: activeTab === 'history',
  });

  const { data: stats, isLoading: statsLoading } = useQuery({
    queryKey: ['stats', clientId, timeRange],
    queryFn: () => getStats(clientId, timeRange),
    enabled: activeTab === 'stats',
  });

  const { data: liveMetrics, isLoading: liveLoading } = useQuery({
    queryKey: ['latestMetrics', clientId],
    queryFn: () => getLatestMetrics(clientId),
    refetchInterval: 5000,
    enabled: activeTab === 'live',
  });

  const latestMetric = liveMetrics?.length ? liveMetrics[liveMetrics.length - 1] : undefined;

  if (clientLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <RefreshCw className="w-8 h-8 animate-spin text-gray-500" />
      </div>
    );
  }

  if (!client) {
    return (
      <div className="bg-red-900/20 border border-red-800 rounded-lg p-4 text-red-400">
        Client not found
      </div>
    );
  }

  const online = isClientOnline(client.last_seen);

  return (
    <div>
      {/* Header */}
      <div className="mb-6">
        <Link
          href="/"
          className="inline-flex items-center gap-2 text-gray-400 hover:text-white mb-4"
        >
          <ArrowLeft className="w-4 h-4" />
          Back to Dashboard
        </Link>

        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className={cn(
              'w-3 h-3 rounded-full',
              online ? 'bg-green-500' : 'bg-red-500'
            )} />
            <h1 className="text-2xl font-bold text-white">{client.hostname}</h1>
          </div>

          <div className="text-sm text-gray-400">
            Last seen: {formatTimeAgo(client.last_seen)}
            {client.version && (
              <span className="ml-4">Version: {client.version}</span>
            )}
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 mb-6 bg-gray-800 p-1 rounded-lg w-fit">
        {(['live', 'history', 'stats'] as Tab[]).map(tab => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={cn(
              'px-4 py-2 rounded-md text-sm font-medium transition-colors capitalize',
              activeTab === tab
                ? 'bg-gray-700 text-white'
                : 'text-gray-400 hover:text-white'
            )}
          >
            {tab}
          </button>
        ))}
      </div>

      {/* Live Tab */}
      {activeTab === 'live' && (
        <div className="bg-gray-800 rounded-lg p-6">
          {liveLoading ? (
            <div className="flex items-center justify-center h-40">
              <RefreshCw className="w-8 h-8 animate-spin text-gray-500" />
            </div>
          ) : latestMetric ? (
            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-8 justify-items-center">
              <MetricGauge label="CPU Usage" value={latestMetric.cpu_usage} />
              <MetricGauge label="RAM Usage" value={latestMetric.ram_usage} />
              <MetricGauge label="Disk Usage" value={latestMetric.disk_usage} />
              <MetricGauge label="Inode Usage" value={latestMetric.inode_usage} />
              {latestMetric.gpu_usage !== null && (
                <MetricGauge label="GPU Usage" value={latestMetric.gpu_usage} />
              )}
            </div>
          ) : (
            <div className="text-center py-8 text-gray-500">
              No metrics available
            </div>
          )}

          {latestMetric?.docker_sz && (
            <div className="mt-6 pt-6 border-t border-gray-700">
              <p className="text-gray-400">
                Docker Directory Size:{' '}
                <span className="text-white font-medium">
                  {formatBytes(latestMetric.docker_sz)}
                </span>
              </p>
            </div>
          )}
        </div>
      )}

      {/* History Tab */}
      {activeTab === 'history' && (
        <div className="bg-gray-800 rounded-lg p-6">
          <div className="flex justify-end mb-4">
            <div className="flex gap-1 bg-gray-900 p-1 rounded-lg">
              {([12, 24, 168] as TimeRange[]).map(hours => (
                <button
                  key={hours}
                  onClick={() => setTimeRange(hours)}
                  className={cn(
                    'px-3 py-1 rounded text-sm transition-colors',
                    timeRange === hours
                      ? 'bg-gray-700 text-white'
                      : 'text-gray-400 hover:text-white'
                  )}
                >
                  {hours === 168 ? '7d' : `${hours}h`}
                </button>
              ))}
            </div>
          </div>

          {metricsLoading ? (
            <div className="flex items-center justify-center h-80">
              <RefreshCw className="w-8 h-8 animate-spin text-gray-500" />
            </div>
          ) : (
            <HistoryChart
              metrics={metrics || []}
              selectedMetrics={['cpu_usage', 'ram_usage', 'disk_usage']}
            />
          )}
        </div>
      )}

      {/* Stats Tab */}
      {activeTab === 'stats' && (
        <div className="bg-gray-800 rounded-lg p-6">
          <div className="flex justify-end mb-4">
            <div className="flex gap-1 bg-gray-900 p-1 rounded-lg">
              {([12, 24, 168] as TimeRange[]).map(hours => (
                <button
                  key={hours}
                  onClick={() => setTimeRange(hours)}
                  className={cn(
                    'px-3 py-1 rounded text-sm transition-colors',
                    timeRange === hours
                      ? 'bg-gray-700 text-white'
                      : 'text-gray-400 hover:text-white'
                  )}
                >
                  {hours === 168 ? '7d' : `${hours}h`}
                </button>
              ))}
            </div>
          </div>

          {statsLoading ? (
            <div className="flex items-center justify-center h-40">
              <RefreshCw className="w-8 h-8 animate-spin text-gray-500" />
            </div>
          ) : (
            <StatsTable stats={stats || []} />
          )}
        </div>
      )}
    </div>
  );
}
