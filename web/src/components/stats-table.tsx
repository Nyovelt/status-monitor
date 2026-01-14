'use client';

import { Stats } from '@/types';
import { formatPercentage } from '@/lib/utils';

interface StatsTableProps {
  stats: Stats[];
}

const metricLabels: Record<string, string> = {
  cpu: 'CPU Usage',
  ram: 'RAM Usage',
  disk: 'Disk Usage',
  inode: 'Inode Usage',
  gpu: 'GPU Usage',
  docker: 'Docker Size',
};

export function StatsTable({ stats }: StatsTableProps) {
  if (stats.length === 0) {
    return (
      <div className="text-gray-500 text-center py-8">
        No statistics available
      </div>
    );
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full">
        <thead>
          <tr className="border-b border-gray-700">
            <th className="text-left py-3 px-4 text-gray-400 font-medium">Metric</th>
            <th className="text-right py-3 px-4 text-gray-400 font-medium">Min</th>
            <th className="text-right py-3 px-4 text-gray-400 font-medium">Avg</th>
            <th className="text-right py-3 px-4 text-gray-400 font-medium">Max</th>
            <th className="text-right py-3 px-4 text-gray-400 font-medium">P95</th>
            <th className="text-right py-3 px-4 text-gray-400 font-medium">Samples</th>
          </tr>
        </thead>
        <tbody>
          {stats.map(stat => (
            <tr key={stat.metric_type} className="border-b border-gray-800">
              <td className="py-3 px-4 text-white">
                {metricLabels[stat.metric_type] || stat.metric_type}
              </td>
              <td className="py-3 px-4 text-right text-green-400">
                {formatPercentage(stat.min)}
              </td>
              <td className="py-3 px-4 text-right text-blue-400">
                {formatPercentage(stat.avg)}
              </td>
              <td className="py-3 px-4 text-right text-yellow-400">
                {formatPercentage(stat.max)}
              </td>
              <td className="py-3 px-4 text-right text-orange-400">
                {formatPercentage(stat.p95)}
              </td>
              <td className="py-3 px-4 text-right text-gray-400">
                {stat.count.toLocaleString()}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
