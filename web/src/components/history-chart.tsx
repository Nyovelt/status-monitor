'use client';

import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from 'recharts';
import { format, parseISO } from 'date-fns';
import { Metric } from '@/types';

interface HistoryChartProps {
  metrics: Metric[];
  selectedMetrics?: string[];
}

const metricConfig: Record<string, { color: string; label: string }> = {
  cpu_usage: { color: '#3b82f6', label: 'CPU' },
  ram_usage: { color: '#10b981', label: 'RAM' },
  disk_usage: { color: '#f59e0b', label: 'Disk' },
  inode_usage: { color: '#8b5cf6', label: 'Inodes' },
  gpu_usage: { color: '#ef4444', label: 'GPU' },
};

export function HistoryChart({
  metrics,
  selectedMetrics = ['cpu_usage', 'ram_usage'],
}: HistoryChartProps) {
  if (metrics.length === 0) {
    return (
      <div className="h-80 flex items-center justify-center text-gray-500">
        No data available
      </div>
    );
  }

  // Reverse to show oldest first
  const chartData = [...metrics].reverse().map(m => ({
    ...m,
    time: format(parseISO(m.timestamp), 'HH:mm'),
    fullTime: format(parseISO(m.timestamp), 'MMM d, HH:mm:ss'),
  }));

  return (
    <ResponsiveContainer width="100%" height={320}>
      <LineChart data={chartData} margin={{ top: 5, right: 20, left: 0, bottom: 5 }}>
        <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
        <XAxis
          dataKey="time"
          stroke="#9ca3af"
          tick={{ fill: '#9ca3af', fontSize: 12 }}
          interval="preserveStartEnd"
        />
        <YAxis
          stroke="#9ca3af"
          tick={{ fill: '#9ca3af', fontSize: 12 }}
          domain={[0, 100]}
          tickFormatter={(v) => `${v}%`}
        />
        <Tooltip
          contentStyle={{
            backgroundColor: '#1f2937',
            border: '1px solid #374151',
            borderRadius: '8px',
          }}
          labelStyle={{ color: '#9ca3af' }}
          labelFormatter={(_, payload) => {
            if (payload && payload[0]) {
              return payload[0].payload.fullTime;
            }
            return '';
          }}
          formatter={(value, name) => {
            const numValue = typeof value === 'number' ? value : 0;
            const strName = String(name);
            return [
              `${numValue.toFixed(1)}%`,
              metricConfig[strName]?.label || strName,
            ];
          }}
        />
        <Legend
          formatter={(value) => metricConfig[value]?.label || value}
          wrapperStyle={{ paddingTop: '10px' }}
        />
        {selectedMetrics.map(metric => (
          metricConfig[metric] && (
            <Line
              key={metric}
              type="monotone"
              dataKey={metric}
              stroke={metricConfig[metric].color}
              strokeWidth={2}
              dot={false}
              isAnimationActive={false}
            />
          )
        ))}
      </LineChart>
    </ResponsiveContainer>
  );
}
