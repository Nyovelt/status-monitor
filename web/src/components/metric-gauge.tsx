'use client';

import { cn } from '@/lib/utils';

interface MetricGaugeProps {
  label: string;
  value: number;
  unit?: string;
  thresholds?: { warning: number; critical: number };
}

export function MetricGauge({
  label,
  value,
  unit = '%',
  thresholds = { warning: 70, critical: 90 },
}: MetricGaugeProps) {
  const percentage = Math.min(Math.max(value, 0), 100);
  const circumference = 2 * Math.PI * 45;
  const strokeDashoffset = circumference - (percentage / 100) * circumference;

  const getColor = () => {
    if (value >= thresholds.critical) return '#ef4444'; // red-500
    if (value >= thresholds.warning) return '#eab308'; // yellow-500
    return '#22c55e'; // green-500
  };

  const getTextColor = () => {
    if (value >= thresholds.critical) return 'text-red-500';
    if (value >= thresholds.warning) return 'text-yellow-500';
    return 'text-green-500';
  };

  return (
    <div className="flex flex-col items-center">
      <div className="relative w-28 h-28">
        <svg className="w-28 h-28 transform -rotate-90">
          {/* Background circle */}
          <circle
            cx="56"
            cy="56"
            r="45"
            stroke="#374151"
            strokeWidth="8"
            fill="transparent"
          />
          {/* Progress circle */}
          <circle
            cx="56"
            cy="56"
            r="45"
            stroke={getColor()}
            strokeWidth="8"
            fill="transparent"
            strokeLinecap="round"
            strokeDasharray={circumference}
            strokeDashoffset={strokeDashoffset}
            className="transition-all duration-500"
          />
        </svg>
        <div className="absolute inset-0 flex items-center justify-center">
          <span className={cn('text-2xl font-bold', getTextColor())}>
            {value.toFixed(1)}
            <span className="text-sm">{unit}</span>
          </span>
        </div>
      </div>
      <span className="mt-2 text-sm text-gray-400">{label}</span>
    </div>
  );
}
