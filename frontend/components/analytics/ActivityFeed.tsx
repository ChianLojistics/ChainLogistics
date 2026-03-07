import * as React from "react";

import type { TimelineEvent } from "@/lib/types/tracking";
import { formatEventTimestamp } from "@/lib/contract/events";
import { cn } from "@/lib/utils";

export interface ActivityFeedProps {
  title?: string;
  events: TimelineEvent[];
  isLoading?: boolean;
  className?: string;
}

export function ActivityFeed({
  title = "Recent activity",
  events,
  isLoading,
  className,
}: Readonly<ActivityFeedProps>) {
  return (
    <div className={cn("rounded-xl border border-zinc-200 bg-white p-5 shadow-sm", className)}>
      <div>
        <h2 className="text-sm font-semibold text-zinc-900">{title}</h2>
        <p className="mt-1 text-sm text-zinc-500">Latest events across your products.</p>
      </div>

      <div className="mt-4">
        {isLoading ? (
          <div className="space-y-3" aria-hidden="true">
            {Array.from({ length: 6 }, (_, i) => (
              <div key={i} className="h-12 rounded-lg bg-zinc-100 animate-pulse" />
            ))}
          </div>
        ) : events.length === 0 ? (
          <div className="rounded-lg border border-dashed border-zinc-200 bg-zinc-50 p-6 text-sm text-zinc-500">
            No recent events.
          </div>
        ) : (
          <ul className="divide-y divide-zinc-100">
            {events.map((e) => (
              <li key={e.event_id} className="py-3">
                <div className="flex items-start justify-between gap-3">
                  <div className="min-w-0">
                    <p className="text-sm font-semibold text-zinc-900 truncate">
                      {e.event_type}
                      <span className="text-zinc-400 font-normal"> · </span>
                      <span className="text-zinc-600 font-medium">{e.product_id}</span>
                    </p>
                    {e.note ? (
                      <p className="mt-0.5 text-sm text-zinc-600 wrap-break-word">
                        {e.note}
                      </p>
                    ) : null}
                  </div>
                  <p className="shrink-0 text-xs text-zinc-500 whitespace-nowrap">
                    {formatEventTimestamp(e.timestamp)}
                  </p>
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
