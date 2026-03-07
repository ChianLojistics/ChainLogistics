"use client";

import * as React from "react";
import {
  Bar,
  BarChart,
  Cell,
  Pie,
  PieChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";

import { cn } from "@/lib/utils";

type EventsByTypeDatum = {
  type: string;
  count: number;
};

const DEFAULT_COLORS = [
  "#16a34a", // green
  "#2563eb", // blue
  "#7c3aed", // purple
  "#ea580c", // orange
  "#0891b2", // cyan
  "#ca8a04", // yellow
  "#4f46e5", // indigo
  "#db2777", // pink
  "#64748b", // slate
];

export interface EventsChartProps {
  title?: string;
  data: EventsByTypeDatum[];
  className?: string;
}

export function EventsChart({
  title = "Events by type",
  data,
  className,
}: Readonly<EventsChartProps>) {
  const [mode, setMode] = React.useState<"pie" | "bar">("pie");

  const sorted = React.useMemo(() => {
    return [...data].sort((a, b) => b.count - a.count);
  }, [data]);

  const empty = sorted.length === 0;

  return (
    <div className={cn("rounded-xl border border-zinc-200 bg-white p-5 shadow-sm", className)}>
      <div className="flex items-start justify-between gap-3">
        <div>
          <h2 className="text-sm font-semibold text-zinc-900">{title}</h2>
          <p className="mt-1 text-sm text-zinc-500">Distribution of tracking events.</p>
        </div>
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => setMode("pie")}
            className={cn(
              "rounded-lg border px-3 py-1.5 text-xs font-semibold",
              mode === "pie" ? "bg-zinc-900 text-white border-zinc-900" : "bg-white text-zinc-700 border-zinc-200 hover:bg-zinc-50"
            )}
          >
            Pie
          </button>
          <button
            type="button"
            onClick={() => setMode("bar")}
            className={cn(
              "rounded-lg border px-3 py-1.5 text-xs font-semibold",
              mode === "bar" ? "bg-zinc-900 text-white border-zinc-900" : "bg-white text-zinc-700 border-zinc-200 hover:bg-zinc-50"
            )}
          >
            Bar
          </button>
        </div>
      </div>

      <div className="mt-4 h-72">
        {empty ? (
          <div className="h-full rounded-lg border border-dashed border-zinc-200 bg-zinc-50 flex items-center justify-center text-sm text-zinc-500">
            No event data yet.
          </div>
        ) : mode === "pie" ? (
          <ResponsiveContainer width="100%" height="100%">
            <PieChart>
              <Tooltip
                formatter={(value: unknown, name: unknown) => [value as number, name as string]}
              />
              <Pie
                data={sorted}
                dataKey="count"
                nameKey="type"
                innerRadius={60}
                outerRadius={100}
                paddingAngle={2}
                isAnimationActive
              >
                {sorted.map((entry, idx) => (
                  <Cell
                    key={`${entry.type}-${idx}`}
                    fill={DEFAULT_COLORS[idx % DEFAULT_COLORS.length]}
                  />
                ))}
              </Pie>
            </PieChart>
          </ResponsiveContainer>
        ) : (
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={sorted} margin={{ top: 10, right: 10, left: 0, bottom: 10 }}>
              <XAxis dataKey="type" tick={{ fontSize: 12 }} />
              <YAxis allowDecimals={false} tick={{ fontSize: 12 }} />
              <Tooltip />
              <Bar dataKey="count" radius={[6, 6, 0, 0]}>
                {sorted.map((entry, idx) => (
                  <Cell
                    key={`${entry.type}-${idx}`}
                    fill={DEFAULT_COLORS[idx % DEFAULT_COLORS.length]}
                  />
                ))}
              </Bar>
            </BarChart>
          </ResponsiveContainer>
        )}
      </div>
    </div>
  );
}
