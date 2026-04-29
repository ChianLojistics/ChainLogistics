"use client";

import * as React from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { StatCard } from "./StatCard";
import { EventsChart } from "./EventsChart";
import { cn } from "@/lib/utils";

export interface DashboardWidget {
  id: string;
  type: "stat" | "chart" | "timeline" | "heatmap" | "network";
  title: string;
  position: { x: number; y: number };
  size: { w: number; h: number };
  data: unknown;
  config?: Record<string, unknown>;
}

export interface AnalyticsDashboardProps {
  className?: string;
  widgets?: DashboardWidget[];
  onRefresh?: () => void;
  isRefreshing?: boolean;
}

export function AnalyticsDashboard({
  className,
  widgets = [],
  onRefresh,
  isRefreshing = false,
}: Readonly<AnalyticsDashboardProps>) {
  const [isEditMode, setIsEditMode] = React.useState(false);

  const defaultWidgets: DashboardWidget[] = [
    {
      id: "total-products",
      type: "stat",
      title: "Total Products",
      position: { x: 0, y: 0 },
      size: { w: 1, h: 1 },
      data: { value: 1234, description: "+12% from last month" },
    },
    {
      id: "active-batches",
      type: "stat",
      title: "Active Batches",
      position: { x: 1, y: 0 },
      size: { w: 1, h: 1 },
      data: { value: 567, description: "In production" },
    },
    {
      id: "events-chart",
      type: "chart",
      title: "Events Distribution",
      position: { x: 0, y: 1 },
      size: { w: 2, h: 2 },
      data: [
        { type: "production", count: 450 },
        { type: "shipping", count: 320 },
        { type: "quality", count: 180 },
        { type: "recall", count: 15 },
      ],
    },
  ];

  const displayWidgets = widgets.length > 0 ? widgets : defaultWidgets;

  const renderWidget = (widget: DashboardWidget) => {
    switch (widget.type) {
      case "stat":
        return (
          <StatCard
            key={widget.id}
            label={widget.title}
            value={(widget.data as { value: number; description: string }).value}
            description={(widget.data as { value: number; description: string }).description}
          />
        );
      case "chart":
        return (
          <EventsChart
            key={widget.id}
            title={widget.title}
            data={widget.data as Array<{ type: string; count: number }>}
            className="col-span-2"
          />
        );
      default:
        return null;
    }
  };

  return (
    <div className={cn("space-y-6", className)}>
      {/* Dashboard Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold text-zinc-900">Analytics Dashboard</h2>
          <p className="text-sm text-zinc-500">Real-time supply chain analytics</p>
        </div>
        <div className="flex items-center gap-2">
          <button
            className="inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground h-8 rounded-md px-3 text-xs"
            onClick={() => setIsEditMode(!isEditMode)}
          >
            {isEditMode ? "Done" : "Customize"}
          </button>
          <button
            className="inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground h-8 rounded-md px-3 text-xs"
            onClick={onRefresh}
            disabled={isRefreshing}
          >
            ↻ Refresh
          </button>
          <button className="inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground h-8 rounded-md px-3 text-xs">
            ↓ Export
          </button>
        </div>
      </div>

      {/* Dashboard Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {displayWidgets.map((widget) => renderWidget(widget))}
      </div>

      {/* Additional Analytics Sections */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card>
          <CardHeader>
            <CardTitle>Supply Chain Timeline</CardTitle>
            <CardDescription>Recent events and activities</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {[1, 2, 3, 4].map((i) => (
                <div key={i} className="flex items-start gap-3">
                  <div className="w-2 h-2 rounded-full bg-blue-500 mt-2" />
                  <div className="flex-1">
                    <p className="text-sm font-medium text-zinc-900">Batch #{1000 + i} shipped</p>
                    <p className="text-xs text-zinc-500">2 hours ago</p>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Performance Metrics</CardTitle>
            <CardDescription>KPIs and performance indicators</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div>
                <div className="flex justify-between text-sm mb-1">
                  <span className="text-zinc-600">On-time Delivery</span>
                  <span className="font-medium">94%</span>
                </div>
                <div className="h-2 bg-zinc-200 rounded-full overflow-hidden">
                  <div className="h-full bg-green-500 rounded-full" style={{ width: "94%" }} />
                </div>
              </div>
              <div>
                <div className="flex justify-between text-sm mb-1">
                  <span className="text-zinc-600">Quality Pass Rate</span>
                  <span className="font-medium">98%</span>
                </div>
                <div className="h-2 bg-zinc-200 rounded-full overflow-hidden">
                  <div className="h-full bg-blue-500 rounded-full" style={{ width: "98%" }} />
                </div>
              </div>
              <div>
                <div className="flex justify-between text-sm mb-1">
                  <span className="text-zinc-600">Inventory Accuracy</span>
                  <span className="font-medium">99%</span>
                </div>
                <div className="h-2 bg-zinc-200 rounded-full overflow-hidden">
                  <div className="h-full bg-purple-500 rounded-full" style={{ width: "99%" }} />
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
