'use client';

import React, { useCallback, useEffect, useState } from 'react';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, AreaChart, Area, BarChart, Bar } from 'recharts';
import { Activity, Calendar, Clock, AlertTriangle, Play, Thermometer } from 'lucide-react';

interface TwinStateData {
  temperature?: number;
  humidity?: number;
}

interface TwinMetrics {
  health_score?: number;
  decay_rate?: number;
  temperature_stress?: number;
  humidity_stress?: number;
}

interface HealthHistoryEntry {
  timestamp: string;
  score: number;
}

interface TwinState {
  id: string;
  state_data: TwinStateData;
  metrics: TwinMetrics;
  timestamp: string;
  source: string;
}

interface DigitalTwin {
  id: string;
  name: string;
  current_state: TwinStateData;
  current_health_score?: number;
  predicted_expiry_date?: string;
  health_history?: HealthHistoryEntry[];
}

interface TwinViewProps {
  twinId: string;
  className?: string;
}

export function TwinView({ twinId, className = '' }: TwinViewProps) {
  const [twin, setTwin] = useState<DigitalTwin | null>(null);
  const [stateHistory, setStateHistory] = useState<TwinState[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<'health' | 'temperature' | 'decay' | 'metrics'>('health');

  const fetchTwinData = useCallback(async () => {
    try {
      const [twinRes, historyRes] = await Promise.all([
        fetch(`/api/v1/digital-twins/${twinId}`),
        fetch(`/api/v1/digital-twins/${twinId}/states?limit=50`)
      ]);

      if (!twinRes.ok || !historyRes.ok) throw new Error('Failed to fetch twin data');

      const twinData = await twinRes.json();
      const historyData = await historyRes.json();

      setTwin(twinData);
      setStateHistory(historyData);
      setLoading(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      setLoading(false);
    }
  }, [twinId]);

  useEffect(() => {
    // setState happens only after an await inside fetchTwinData, so this is an
    // async data fetch, not a synchronous cascading render.
    // eslint-disable-next-line react-hooks/set-state-in-effect
    fetchTwinData();
    const interval = setInterval(fetchTwinData, 15000); // Refresh every 15s
    return () => clearInterval(interval);
  }, [fetchTwinData]);

  const getHealthChartData = () => {
    if (!twin?.health_history) return [];

    return twin.health_history.map((entry) => ({
      timestamp: new Date(entry.timestamp).toLocaleTimeString(),
      score: Math.round(entry.score * 100),
    }));
  };

  const getTemperatureChartData = () => {
    return stateHistory
      .map((state) => ({
        timestamp: new Date(state.timestamp).toLocaleTimeString(),
        temperature: state.state_data.temperature || 0,
        humidity: state.state_data.humidity || 0,
      }))
      .reverse();
  };

  const getDecayChartData = () => {
    return stateHistory
      .map((state) => ({
        timestamp: new Date(state.timestamp).toLocaleTimeString(),
        healthScore: state.metrics.health_score ? Math.round(state.metrics.health_score * 100) : 0,
        decayRate: state.metrics.decay_rate || 0,
      }))
      .reverse();
  };

  const getMetricsChartData = () => {
    return stateHistory
      .map((state) => ({
        timestamp: new Date(state.timestamp).toLocaleTimeString(),
        temperatureStress: state.metrics.temperature_stress || 0,
        humidityStress: state.metrics.humidity_stress || 0,
      }))
      .reverse();
  };

  const runMonteCarlo = async () => {
    try {
      const response = await fetch(`/api/v1/physics/twins/${twinId}/monte-carlo`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          num_runs: 1000,
          confidence_level: 0.95,
          parameter_ranges: {
            temperature: [15, 35],
            humidity: [30, 80],
            elapsed_hours: 48,
          },
        }),
      });

      if (!response.ok) throw new Error('Failed to run Monte Carlo simulation');
      const data = await response.json();
      alert(`Monte Carlo Simulation Results:\nMean Health Score: ${data.mean_health_score.toFixed(3)}\n95% CI: [${data.confidence_interval.lower.toFixed(3)}, ${data.confidence_interval.upper.toFixed(3)}]`);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Unknown error');
    }
  };

  if (loading) {
    return (
      <Card className={`p-6 ${className}`}>
        <Skeleton className="h-96 w-full" />
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

  const healthChartData = getHealthChartData();
  const temperatureChartData = getTemperatureChartData();
  const decayChartData = getDecayChartData();
  const metricsChartData = getMetricsChartData();

  return (
    <Card className={`p-6 ${className}`}>
      <div className="space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold">{twin?.name}</h2>
            <div className="flex items-center gap-2 mt-1">
              <Badge className="flex items-center gap-1 border border-gray-300">
                <Activity className="w-3 h-3" />
                Live
              </Badge>
              {twin?.predicted_expiry_date && (
                <Badge className="flex items-center gap-1 border border-gray-300">
                  <Calendar className="w-3 h-3" />
                  Exp: {new Date(twin.predicted_expiry_date).toLocaleDateString()}
                </Badge>
              )}
            </div>
          </div>
          <Button onClick={runMonteCarlo} className="flex items-center gap-2">
            <Play className="w-4 h-4" />
            Run Monte Carlo
          </Button>
        </div>

        {/* View Mode Selector */}
        <div className="flex gap-2">
          {(['health', 'temperature', 'decay', 'metrics'] as const).map((mode) => (
            <Button
              key={mode}
              onClick={() => setViewMode(mode)}
              className={`capitalize ${viewMode === mode ? 'bg-primary text-primary-foreground' : 'border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground'}`}
            >
              {mode}
            </Button>
          ))}
        </div>

        {/* Chart Display */}
        <div className="h-80">
          {viewMode === 'health' && (
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={healthChartData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="timestamp" />
                <YAxis domain={[0, 100]} />
                <Tooltip />
                <Legend />
                <Area
                  type="monotone"
                  dataKey="score"
                  stroke="#22c55e"
                  fill="#22c55e"
                  fillOpacity={0.3}
                  name="Health Score %"
                />
              </AreaChart>
            </ResponsiveContainer>
          )}

          {viewMode === 'temperature' && (
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={temperatureChartData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="timestamp" />
                <YAxis yAxisId="temp" orientation="left" />
                <YAxis yAxisId="humidity" orientation="right" />
                <Tooltip />
                <Legend />
                <Line
                  yAxisId="temp"
                  type="monotone"
                  dataKey="temperature"
                  stroke="#ef4444"
                  name="Temperature (°C)"
                />
                <Line
                  yAxisId="humidity"
                  type="monotone"
                  dataKey="humidity"
                  stroke="#3b82f6"
                  name="Humidity (%)"
                />
              </LineChart>
            </ResponsiveContainer>
          )}

          {viewMode === 'decay' && (
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={decayChartData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="timestamp" />
                <YAxis yAxisId="health" orientation="left" domain={[0, 100]} />
                <YAxis yAxisId="decay" orientation="right" />
                <Tooltip />
                <Legend />
                <Line
                  yAxisId="health"
                  type="monotone"
                  dataKey="healthScore"
                  stroke="#22c55e"
                  name="Health Score %"
                />
                <Line
                  yAxisId="decay"
                  type="monotone"
                  dataKey="decayRate"
                  stroke="#f59e0b"
                  name="Decay Rate"
                />
              </LineChart>
            </ResponsiveContainer>
          )}

          {viewMode === 'metrics' && (
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={metricsChartData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="timestamp" />
                <YAxis />
                <Tooltip />
                <Legend />
                <Bar dataKey="temperatureStress" fill="#ef4444" name="Temp Stress" />
                <Bar dataKey="humidityStress" fill="#3b82f6" name="Humidity Stress" />
              </BarChart>
            </ResponsiveContainer>
          )}
        </div>

        {/* Current State Summary */}
        <div className="grid grid-cols-3 gap-4">
          <div className="p-4 bg-gray-50 rounded-lg">
            <div className="flex items-center gap-2 text-gray-600 mb-1">
              <Activity className="w-4 h-4" />
              <span className="text-sm">Health Score</span>
            </div>
            <div className="text-2xl font-bold">
              {twin?.current_health_score ? Math.round(twin.current_health_score * 100) : 0}%
            </div>
          </div>
          <div className="p-4 bg-gray-50 rounded-lg">
            <div className="flex items-center gap-2 text-gray-600 mb-1">
              <Thermometer className="w-4 h-4" />
              <span className="text-sm">Temperature</span>
            </div>
            <div className="text-2xl font-bold">
              {twin?.current_state?.temperature || 'N/A'}°C
            </div>
          </div>
          <div className="p-4 bg-gray-50 rounded-lg">
            <div className="flex items-center gap-2 text-gray-600 mb-1">
              <Clock className="w-4 h-4" />
              <span className="text-sm">Last Update</span>
            </div>
            <div className="text-lg font-bold">
              {stateHistory.length > 0
                ? new Date(stateHistory[0].timestamp).toLocaleTimeString()
                : 'N/A'}
            </div>
          </div>
        </div>

        {/* Risk Indicators */}
        {(twin?.current_health_score || 0) < 0.7 && (
          <div className="p-4 bg-yellow-50 rounded-lg border border-yellow-200 flex items-center gap-3">
            <AlertTriangle className="w-5 h-5 text-yellow-600" />
            <div className="text-yellow-800">
              <div className="font-medium">Health Score Below Threshold</div>
              <div className="text-sm">Current health score indicates potential quality issues. Monitor closely.</div>
            </div>
          </div>
        )}
      </div>
    </Card>
  );
}
