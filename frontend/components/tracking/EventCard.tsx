"use client";

import type { EventCardProps } from "@/lib/types/tracking";
import { formatEventTimestamp, getRelativeTime } from "@/lib/contract/events";
import { shortenPublicKey } from "@/lib/utils/format";

import { EventTypeBadge, getEventTypeBadgeMeta } from "./EventTypeBadge";

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export function EventCard({ event, isFirst, isLast }: EventCardProps) {
  const meta = getEventTypeBadgeMeta(event.event_type);
  const Icon = meta.icon;

  return (
    <div className="relative flex gap-4 sm:gap-6">
      {/* Timeline line */}
      <div className="flex flex-col items-center">
        <div
          className={`flex h-10 w-10 sm:h-12 sm:w-12 items-center justify-center rounded-full border-2 ${meta.className}`}
        >
          <Icon className="h-5 w-5 sm:h-6 sm:w-6" aria-hidden="true" />
        </div>
        {!isLast && (
          <div className="mt-2 h-full w-0.5 bg-gray-200"></div>
        )}
      </div>

      {/* Event content */}
      <div className="flex-1 pb-6 sm:pb-8">
        <div className="rounded-lg border border-gray-200 bg-white p-4 shadow-sm hover:shadow-md transition-shadow">
          <div className="flex items-start justify-between">
            <div className="flex-1 min-w-0">
              <div className="mb-2 flex flex-col gap-2 sm:flex-row sm:items-center">
                <EventTypeBadge eventType={event.event_type} size="md" />
                <span className="text-xs sm:text-sm text-gray-500">
                  {getRelativeTime(event.timestamp)}
                </span>
              </div>

              {event.note && (
                <p className="mb-3 text-sm text-gray-700 break-words">{event.note}</p>
              )}

              <div className="flex flex-col gap-2 sm:flex-row sm:flex-wrap sm:items-center text-xs text-gray-500">
                <div className="flex items-center gap-1">
                  <span className="font-medium">Actor:</span>
                  <span className="font-mono">
                    {shortenPublicKey(event.actor)}
                  </span>
                </div>
                <div className="flex items-center gap-1">
                  <span className="font-medium">Time:</span>
                  <span className="break-all sm:break-normal">
                    {formatEventTimestamp(event.timestamp)}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
