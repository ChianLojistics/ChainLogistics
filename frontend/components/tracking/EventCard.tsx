"use client";

import type { EventCardProps } from "@/lib/types/tracking";
import { formatEventTimestamp, getRelativeTime } from "@/lib/contract/events";
import { shortenPublicKey } from "@/lib/utils/format";

const EVENT_COLORS: Record<string, string> = {
  HARVEST: "bg-green-100 text-green-800 border-green-300",
  PROCESS: "bg-blue-100 text-blue-800 border-blue-300",
  PACKAGE: "bg-purple-100 text-purple-800 border-purple-300",
  SHIP: "bg-orange-100 text-orange-800 border-orange-300",
  RECEIVE: "bg-cyan-100 text-cyan-800 border-cyan-300",
  QUALITY_CHECK: "bg-yellow-100 text-yellow-800 border-yellow-300",
  TRANSFER: "bg-indigo-100 text-indigo-800 border-indigo-300",
  REGISTER: "bg-gray-100 text-gray-800 border-gray-300",
};

const EVENT_ICONS: Record<string, string> = {
  HARVEST: "🌱",
  PROCESS: "⚙️",
  PACKAGE: "📦",
  SHIP: "🚚",
  RECEIVE: "📥",
  QUALITY_CHECK: "✅",
  TRANSFER: "🔄",
  REGISTER: "📝",
};

export function EventCard({ event, isFirst, isLast }: EventCardProps) {
  const eventColor = EVENT_COLORS[event.event_type] || EVENT_COLORS.REGISTER;
  const eventIcon = EVENT_ICONS[event.event_type] || "📋";

  return (
    <div className="relative flex gap-4 sm:gap-6">
      {/* Timeline line */}
      <div className="flex flex-col items-center">
        <div
          className={`flex h-10 w-10 sm:h-12 sm:w-12 items-center justify-center rounded-full border-2 text-xl sm:text-2xl ${eventColor}`}
        >
          {eventIcon}
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
                <span
                  className={`inline-flex rounded-full border px-3 py-1 text-xs font-semibold ${eventColor}`}
                >
                  {event.event_type}
                </span>
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
