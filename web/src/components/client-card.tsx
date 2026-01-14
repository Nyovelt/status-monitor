'use client';

import Link from 'next/link';
import { Cpu, HardDrive, MemoryStick, Gpu } from 'lucide-react';
import { Client, Metric } from '@/types';
import { Sparkline } from './sparkline';
import { formatPercentage, formatTimeAgo, isClientOnline, cn } from '@/lib/utils';

interface ClientCardProps {
  client: Client;
  latestMetric?: Metric;
  recentMetrics: Metric[];
}

export function ClientCard({ client, latestMetric, recentMetrics }: ClientCardProps) {
  const online = isClientOnline(client.last_seen);
  const cpuData = recentMetrics.map(m => m.cpu_usage);
  const ramData = recentMetrics.map(m => m.ram_usage);

  return (
    <Link href={`/client/${client.id}`}>
      <div className="bg-gray-800 rounded-lg p-4 hover:bg-gray-750 transition-colors border border-gray-700 hover:border-gray-600">
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <div className={cn(
              'w-2 h-2 rounded-full',
              online ? 'bg-green-500' : 'bg-red-500'
            )} />
            <h3 className="font-medium text-white truncate">{client.hostname}</h3>
          </div>
          <span className="text-xs text-gray-400">
            {formatTimeAgo(client.last_seen)}
          </span>
        </div>

        {latestMetric ? (
          <>
            <div className="grid grid-cols-2 gap-3 mb-3">
              <MetricValue
                icon={Cpu}
                label="CPU"
                value={latestMetric.cpu_usage}
              />
              <MetricValue
                icon={MemoryStick}
                label="RAM"
                value={latestMetric.ram_usage}
              />
              <MetricValue
                icon={HardDrive}
                label="Disk"
                value={latestMetric.disk_usage}
              />
              {latestMetric.gpu_usage !== null && (
                <MetricValue
                  icon={Gpu}
                  label="GPU"
                  value={latestMetric.gpu_usage}
                />
              )}
            </div>

            <div className="grid grid-cols-2 gap-2">
              <div>
                <span className="text-xs text-gray-500 mb-1 block">CPU</span>
                <Sparkline data={cpuData} color="#3b82f6" />
              </div>
              <div>
                <span className="text-xs text-gray-500 mb-1 block">RAM</span>
                <Sparkline data={ramData} color="#10b981" />
              </div>
            </div>
          </>
        ) : (
          <div className="text-gray-500 text-sm py-4 text-center">
            Waiting for metrics...
          </div>
        )}
      </div>
    </Link>
  );
}

interface MetricValueProps {
  icon: React.ComponentType<{ className?: string }>;
  label: string;
  value: number;
}

function MetricValue({ icon: Icon, label, value }: MetricValueProps) {
  const colorClass = value >= 90 ? 'text-red-400' : value >= 70 ? 'text-yellow-400' : 'text-green-400';

  return (
    <div className="flex items-center gap-2">
      <Icon className="w-4 h-4 text-gray-500" />
      <div>
        <span className="text-xs text-gray-500">{label}</span>
        <p className={cn('text-sm font-medium', colorClass)}>
          {formatPercentage(value)}
        </p>
      </div>
    </div>
  );
}
