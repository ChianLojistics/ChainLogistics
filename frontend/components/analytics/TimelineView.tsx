"use client";

import * as React from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";

export interface TimelineEvent {
  id: string;
  timestamp: Date;
  type: string;
  title: string;
  description: string;
  location?: string;
  metadata?: Record<string, unknown>;
}

export interface TimelineViewProps {
  events: TimelineEvent[];
  className?: string;
  onEventClick?: (event: TimelineEvent) => void;
}

export function TimelineView({
  events,
  className,
  onEventClick,
}: Readonly<TimelineViewProps>) {
  const [zoom, setZoom] = React.useState(1);
  const [selectedType, setSelectedType] = React.useState<string | null>(null);

  const eventTypes = React.useMemo(() => {
    const types = new Set(events.map((e) => e.type));
    return Array.from(types);
  }, [events]);

  const filteredEvents = React.useMemo(() => {
    if (!selectedType) return events;
    return events.filter((e) => e.type === selectedType);
  }, [events, selectedType]);

  const groupedEvents = React.useMemo(() => {
    const groups = new Map<string, TimelineEvent[]>();
    filteredEvents.forEach((event) => {
      const date = event.timestamp.toDateString();
      if (!groups.has(date)) {
        groups.set(date, []);
      }
      groups.get(date)!.push(event);
    });
    return Array.from(groups.entries()).map(([date, events]) => ({
      date,
      events: events.sort((a, b) => a.timestamp.getTime() - b.timestamp.getTime()),
    }));
  }, [filteredEvents]);

  const getEventColor = (type: string) => {
    const colors: Record<string, string> = {
      production: "bg-blue-500",
      shipping: "bg-green-500",
      quality: "bg-purple-500",
      recall: "bg-red-500",
      inspection: "bg-yellow-500",
    };
    return colors[type] || "bg-zinc-500";
  };

  return (
    <Card className={cn("", className)}>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>Supply Chain Timeline</CardTitle>
            <CardDescription>Interactive timeline of all supply chain events</CardDescription>
          </div>
          <div className="flex items-center gap-2">
            <button
              className="inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground h-8 rounded-md px-3 text-xs"
              onClick={() => setZoom(Math.max(0.5, zoom - 0.25))}
            >
              +
            </button>
            <button
              className="inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground h-8 rounded-md px-3 text-xs"
              onClick={() => setZoom(Math.min(2, zoom + 0.25))}
            >
              -
            </button>
            <button
              className="inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground h-8 rounded-md px-3 text-xs"
              onClick={() => setSelectedType(null)}
            >
              Clear Filter
            </button>
          </div>
        </div>
        {/* Event Type Filters */}
        <div className="flex flex-wrap gap-2 mt-4">
          {eventTypes.map((type) => (
            <button
              key={type}
              className={cn(
                "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium h-8 rounded-md px-3 text-xs",
                selectedType === type
                  ? "bg-primary text-primary-foreground shadow hover:bg-primary/90"
                  : "border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground"
              )}
              onClick={() => setSelectedType(selectedType === type ? null : type)}
            >
              {type}
            </button>
          ))}
        </div>
      </CardHeader>
      <CardContent>
        <div
          className="space-y-6 overflow-y-auto"
          style={{ maxHeight: "600px", transform: `scale(${zoom})` }}
        >
          {groupedEvents.map(({ date, events: dayEvents }) => (
            <div key={date} className="relative">
              <div className="text-sm font-semibold text-zinc-900 mb-3">{date}</div>
              <div className="space-y-3 pl-4 border-l-2 border-zinc-200">
                {dayEvents.map((event) => (
                  <div
                    key={event.id}
                    className="relative cursor-pointer group"
                    onClick={() => onEventClick?.(event)}
                  >
                    <div
                      className={cn(
                        "absolute -left-[21px] top-1 w-3 h-3 rounded-full border-2 border-white",
                        getEventColor(event.type)
                      )}
                    />
                    <div className="bg-zinc-50 rounded-lg p-3 hover:bg-zinc-100 transition-colors">
                      <div className="flex items-start justify-between">
                        <div className="flex-1">
                          <p className="text-sm font-medium text-zinc-900">{event.title}</p>
                          <p className="text-xs text-zinc-500 mt-1">{event.description}</p>
                          {event.location && (
                            <p className="text-xs text-zinc-400 mt-1">📍 {event.location}</p>
                          )}
                        </div>
                        <span className="text-xs text-zinc-400">
                          {event.timestamp.toLocaleTimeString()}
                        </span>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
