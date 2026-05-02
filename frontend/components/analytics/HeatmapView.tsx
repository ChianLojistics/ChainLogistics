"use client";

import * as React from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";

export interface HeatmapData {
  x: string;
  y: string;
  value: number;
  label?: string;
}

export interface HeatmapViewProps {
  data: HeatmapData[];
  title?: string;
  description?: string;
  colorScale?: "blue" | "green" | "red" | "purple";
  className?: string;
  onCellClick?: (data: HeatmapData) => void;
}

export function HeatmapView({
  data,
  title = "Supply Chain Heatmap",
  description = "Geographic or temporal distribution visualization",
  colorScale = "blue",
  className,
  onCellClick,
}: Readonly<HeatmapViewProps>) {
  const [hoveredCell, setHoveredCell] = React.useState<HeatmapData | null>(null);

  const xValues = React.useMemo(() => {
    return Array.from(new Set(data.map((d) => d.x))).sort();
  }, [data]);

  const yValues = React.useMemo(() => {
    return Array.from(new Set(data.map((d) => d.y))).sort();
  }, [data]);

  const getValueColor = (value: number, min: number, max: number) => {
    const normalized = (value - min) / (max - min || 1);
    const colors: Record<string, { start: string; end: string }> = {
      blue: { start: "#eff6ff", end: "#1e40af" },
      green: { start: "#f0fdf4", end: "#166534" },
      red: { start: "#fef2f2", end: "#991b1b" },
      purple: { start: "#faf5ff", end: "#6b21a8" },
    };
    const scale = colors[colorScale];
    // Simple interpolation
    const r = Math.round(
      parseInt(scale.start.slice(1, 3), 16) +
        (parseInt(scale.end.slice(1, 3), 16) - parseInt(scale.start.slice(1, 3), 16)) * normalized
    );
    const g = Math.round(
      parseInt(scale.start.slice(3, 5), 16) +
        (parseInt(scale.end.slice(3, 5), 16) - parseInt(scale.start.slice(3, 5), 16)) * normalized
    );
    const b = Math.round(
      parseInt(scale.start.slice(5, 7), 16) +
        (parseInt(scale.end.slice(5, 7), 16) - parseInt(scale.start.slice(5, 7), 16)) * normalized
    );
    return `rgb(${r}, ${g}, ${b})`;
  };

  const minValue = Math.min(...data.map((d) => d.value));
  const maxValue = Math.max(...data.map((d) => d.value));

  return (
    <Card className={cn("", className)}>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>{title}</CardTitle>
            <CardDescription>{description}</CardDescription>
          </div>
          <button className="inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground h-8 rounded-md px-3 text-xs">
            ℹ Legend
          </button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="relative">
          {/* Heatmap Grid */}
          <div className="overflow-x-auto">
            <div className="inline-block min-w-full">
              {/* Header Row */}
              <div className="flex">
                <div className="w-24 flex-shrink-0" />
                {xValues.map((x) => (
                  <div
                    key={x}
                    className="w-16 flex-shrink-0 text-xs text-center text-zinc-600 font-medium py-2"
                  >
                    {x}
                  </div>
                ))}
              </div>
              {/* Data Rows */}
              {yValues.map((y) => (
                <div key={y} className="flex items-center">
                  <div className="w-24 flex-shrink-0 text-xs text-zinc-600 font-medium py-2 pr-2 text-right">
                    {y}
                  </div>
                  {xValues.map((x) => {
                    const cellData = data.find((d) => d.x === x && d.y === y);
                    return (
                      <div
                        key={`${x}-${y}`}
                        className={cn(
                          "w-16 h-16 flex-shrink-0 m-0.5 rounded cursor-pointer transition-all hover:scale-110 hover:shadow-lg",
                          cellData ? "" : "bg-zinc-100"
                        )}
                        style={{
                          backgroundColor: cellData
                            ? getValueColor(cellData.value, minValue, maxValue)
                            : undefined,
                        }}
                        onClick={() => cellData && onCellClick?.(cellData)}
                        onMouseEnter={() => setHoveredCell(cellData || null)}
                        onMouseLeave={() => setHoveredCell(null)}
                      >
                        {cellData && (
                          <div className="flex items-center justify-center h-full text-xs font-medium text-white">
                            {cellData.value}
                          </div>
                        )}
                      </div>
                    );
                  })}
                </div>
              ))}
            </div>
          </div>

          {/* Tooltip */}
          {hoveredCell && (
            <div className="absolute top-0 right-0 bg-white border border-zinc-200 rounded-lg p-3 shadow-lg z-10">
              <div className="text-sm font-medium text-zinc-900">{hoveredCell.label || hoveredCell.x}</div>
              <div className="text-xs text-zinc-500 mt-1">
                {hoveredCell.x} × {hoveredCell.y}
              </div>
              <div className="text-lg font-semibold text-zinc-900 mt-2">{hoveredCell.value}</div>
            </div>
          )}

          {/* Color Scale Legend */}
          <div className="mt-4 flex items-center gap-2">
            <span className="text-xs text-zinc-500">Low</span>
            <div className="flex-1 h-4 rounded" style={{
              background: `linear-gradient(to right, ${getValueColor(minValue, minValue, maxValue)}, ${getValueColor(maxValue, minValue, maxValue)})`
            }} />
            <span className="text-xs text-zinc-500">High</span>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
