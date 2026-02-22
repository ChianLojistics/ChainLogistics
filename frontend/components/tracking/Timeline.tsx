"use client";

import { useEffect, useState } from "react";
import type { TimelineEvent } from "@/lib/types/tracking";
import { fetchProductEvents } from "@/lib/contract/events";
import { EventCard } from "./EventCard";

interface TimelineProps {
  productId: string;
}

export function Timeline({ productId }: TimelineProps) {
  const [events, setEvents] = useState<TimelineEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function loadEvents() {
      try {
        setLoading(true);
        setError(null);
        const fetchedEvents = await fetchProductEvents(productId);
        // Sort by timestamp descending (newest first)
        const sorted = fetchedEvents.sort((a, b) => b.timestamp - a.timestamp);
        setEvents(sorted);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to load events");
      } finally {
        setLoading(false);
      }
    }

    loadEvents();
  }, [productId]);

  if (loading) {
    return <TimelineSkeleton />;
  }

  if (error) {
    return (
      <div className="rounded-lg border border-red-200 bg-red-50 p-6 text-center">
        <p className="text-sm font-medium text-red-800">
          Failed to load events
        </p>
        <p className="mt-1 text-xs text-red-600">{error}</p>
      </div>
    );
  }

  if (events.length === 0) {
    return (
      <div className="rounded-lg border border-gray-200 bg-gray-50 p-12 text-center">
        <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-gray-100">
          <svg
            className="h-8 w-8 text-gray-400"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
        </div>
        <h3 className="text-sm font-semibold text-gray-900">
          No tracking events yet
        </h3>
        <p className="mt-1 text-sm text-gray-500">
          Events will appear here as the product moves through the supply chain.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-0">
      {events.map((event, index) => (
        <EventCard
          key={event.event_id}
          event={event}
          isFirst={index === 0}
          isLast={index === events.length - 1}
        />
      ))}
    </div>
  );
}

function TimelineSkeleton() {
  return (
    <div className="space-y-0">
      {[1, 2, 3].map((i) => (
        <div key={i} className="relative flex gap-4 sm:gap-6">
          <div className="flex flex-col items-center">
            <div className="h-10 w-10 sm:h-12 sm:w-12 animate-pulse rounded-full bg-gray-200"></div>
            {i < 3 && <div className="mt-2 h-full w-0.5 bg-gray-200"></div>}
          </div>
          <div className="flex-1 pb-6 sm:pb-8">
            <div className="animate-pulse rounded-lg border border-gray-200 bg-white p-4">
              <div className="mb-2 h-4 w-32 rounded bg-gray-200"></div>
              <div className="mb-3 h-3 w-full rounded bg-gray-200"></div>
              <div className="h-3 w-48 rounded bg-gray-200"></div>
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}
