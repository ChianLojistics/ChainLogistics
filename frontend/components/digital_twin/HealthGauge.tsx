'use client';

import React, { useCallback, useEffect, useState } from 'react';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Skeleton } from '@/components/ui/skeleton';
import { AlertTriangle, CheckCircle, Activity, Thermometer, Droplets } from 'lucide-react';

interface HealthMetric {
  id: string;
  metric_type: string;
  metric_value: number;
  threshold_min?: number;
  threshold_max?: number;
  severity?: string;
  calculated_at: string;
  metadata: Record<string, unknown>;
}

interface HealthGaugeProps {
  twinId: string;
  className?: string;
}

export function HealthGauge({ twinId, className = '' }: HealthGaugeProps) {
  const [healthMetrics, setHealthMetrics] = useState<HealthMetric[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchHealthMetrics = useCallback(async () => {
    try {
      const response = await fetch(`/api/v1/physics/twins/${twinId}/health-metrics`);
      if (!response.ok) throw new Error('Failed to fetch health metrics');
      const data = await response.json();
      setHealthMetrics(data);
      setLoading(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      setLoading(false);
    }
  }, [twinId]);

  useEffect(() => {
    // setState happens only after an await inside fetchHealthMetrics, so this is
    // an async data fetch, not a synchronous cascading render.
    // eslint-disable-next-line react-hooks/set-state-in-effect
    fetchHealthMetrics();
    const interval = setInterval(fetchHealthMetrics, 30000); // Refresh every 30s
    return () => clearInterval(interval);
  }, [fetchHealthMetrics]);

  const getSeverityIcon = (severity?: string) => {
    switch (severity) {
      case 'normal':
        return <CheckCircle className="w-5 h-5 text-green-500" />;
      case 'warning':
        return <AlertTriangle className="w-5 h-5 text-yellow-500" />;
      case 'critical':
        return <AlertTriangle className="w-5 h-5 text-red-500" />;
      default:
        return <Activity className="w-5 h-5 text-gray-500" />;
    }
  };

  const getMetricIcon = (metricType: string) => {
    switch (metricType) {
      case 'overall_health':
        return <Activity className="w-5 h-5" />;
      case 'temperature_stress':
        return <Thermometer	className="w-5 h-5" />;
      case 'humidity_stress':
        return <Droplets className="w-5 h-5" />;
      default:
        return <Activity className="w-5 h-5" />;
    }
  };

  const getOverallHealth = () => {
    const overall = healthMetrics.find(m => m.metric_type === 'overall_health');
    return overall?.metric_value || 0;
  };

  const getHealthPercentage = () => {
    return Math.round(getOverallHealth() * 100);
  };

  const getHealthColor = () => {
    const health = getOverallHealth();
    if (health >= 0.8) return 'text-green-500';
    if (health >= 0.6) return 'text-yellow-500';
    return 'text-red-500';
  };

  const getGaugeColor = () => {
    const health = getOverallHealth();
    if (health >= 0.8) return '#22c55e';
    if (health >= 0.6) return '#eab308';
    return '#ef4444';
  };

  if (loading) {
    return (
      <Card className={`p-6 ${className}`}>
        <Skeleton className="h-48 w-full" />
      </Card>
    );
  }

  if (error) {
    return (
      <Card className={`p-6 ${className}`}>
        <div className="flex items-center gap-2 text-red-500">
          <AlertTriangle className="w-5 h-5" />
          <span>{error}</span>
        </div>
      </Card>
    );
  }

  const healthPercentage = getHealthPercentage();
  const gaugeColor = getGaugeColor();
  const circumference = 2 * Math.PI * 45; // radius = 45
  const strokeDashoffset = circumference - (healthPercentage / 100) * circumference;

  return (
    <Card className={`p-6 ${className}`}>
      <div className="space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <h2 className="text-2xl font-bold">Health Gauge</h2>
          <Badge className="flex items-center gap-1 border border-gray-300">
            <Activity className="w-3 h-3" />
            Live
          </Badge>
        </div>

        {/* Main Health Score Display */}
        <div className="flex items-center justify-center">
          <div className="relative">
            <svg width="200" height="200" className="transform -rotate-90">
              {/* Background circle */}
              <circle
                cx="100"
                cy="100"
                r="45"
                stroke="#e5e7eb"
                strokeWidth="10"
                fill="none"
              />
              {/* Progress circle */}
              <circle
                cx="100"
                cy="100"
                r="45"
                stroke={gaugeColor}
                strokeWidth="10"
                fill="none"
                strokeDasharray={circumference}
                strokeDashoffset={strokeDashoffset}
                strokeLinecap="round"
                className="transition-all duration-500 ease-out"
              />
            </svg>
            <div className="absolute inset-0 flex items-center justify-center transform rotate-90">
              <div className="text-center">
                <div className={`text-4xl font-bold ${getHealthColor()}`}>
                  {healthPercentage}%
                </div>
                <div className="text-sm text-gray-500">Health Score</div>
              </div>
            </div>
          </div>
        </div>

        {/* Detailed Metrics */}
        <div className="space-y-3">
          {healthMetrics.map((metric) => (
            <div
              key={metric.id}
              className="flex items-center justify-between p-3 bg-gray-50 rounded-lg"
            >
              <div className="flex items-center gap-3">
                {getMetricIcon(metric.metric_type)}
                <div>
                  <div className="font-medium capitalize">
                    {metric.metric_type.replace(/_/g, ' ')}
                  </div>
                  <div className="text-sm text-gray-500">
                    {new Date(metric.calculated_at).toLocaleString()}
                  </div>
                </div>
              </div>
              <div className="flex items-center gap-3">
                <div className="text-right">
                  <div className="font-bold text-lg">
                    {metric.metric_type === 'overall_health'
                      ? `${Math.round(metric.metric_value * 100)}%`
                      : metric.metric_value.toFixed(2)}
                  </div>
                  {metric.threshold_min !== undefined && metric.threshold_max !== undefined && (
                    <div className="text-xs text-gray-500">
                      Range: {metric.threshold_min} - {metric.threshold_max}
                    </div>
                  )}
                </div>
                {getSeverityIcon(metric.severity)}
              </div>
            </div>
          ))}
        </div>

        {/* Risk Factors and Recommendations */}
        {healthMetrics.length > 0 && (
          <div className="grid grid-cols-2 gap-4">
            <div className="p-3 bg-yellow-50 rounded-lg border border-yellow-200">
              <div className="font-medium text-yellow-800 mb-2">Risk Factors</div>
              <div className="text-sm text-yellow-700">
                {getOverallHealth() < 0.8 && (
                  <div className="flex items-center gap-1">
                    <AlertTriangle className="w-4 h-4" />
                    Health score below optimal
                  </div>
                )}
                {healthMetrics
                  .filter(m => m.severity === 'critical')
                  .map(m => (
                    <div key={m.id} className="flex items-center gap-1 mt-1">
                      <AlertTriangle className="w-4 h-4" />
                      {m.metric_type.replace(/_/g, ' ')} critical
                    </div>
                  ))}
              </div>
            </div>
            <div className="p-3 bg-blue-50 rounded-lg border border-blue-200">
              <div className="font-medium text-blue-800 mb-2">Recommendations</div>
              <div className="text-sm text-blue-700 space-y-1">
                {getOverallHealth() < 0.8 && (
                  <div>• Monitor conditions closely</div>
                )}
                {getOverallHealth() < 0.6 && (
                  <div>• Consider expedited processing</div>
                )}
                {healthMetrics.some(m => m.metric_type === 'temperature_stress' && m.severity === 'critical') && (
                  <div>• Improve temperature control</div>
                )}
                {healthMetrics.some(m => m.metric_type === 'humidity_stress' && m.severity === 'critical') && (
                  <div>• Adjust humidity levels</div>
                )}
              </div>
            </div>
          </div>
        )}
      </div>
    </Card>
  );
}
