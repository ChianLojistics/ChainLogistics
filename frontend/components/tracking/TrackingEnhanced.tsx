"use client";

/**
 * Enhanced real-time tracking panel (#322).
 *
 * Adds:
 *  - ETA calculation based on average inter-event velocity
 *  - Delay detection when the last event is older than the expected interval
 *  - Delay notification banner surfaced to the operator
 *  - Customer tracking portal link with a copy-able URL
 */

import * as React from "react";
import {
  AlertTriangle,
  CheckCircle2,
  Clock,
  Copy,
  ExternalLink,
  MapPin,
  RefreshCw,
} from "lucide-react";

import type { TimelineEvent } from "@/lib/types/tracking";
import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

// ── Types ──────────────────────────────────────────────────────────────────────

export interface TrackingStatus {
  lastEventAt: Date | null;
  /** Estimated time of next event, or arrival, in ms since epoch */
  etaMs: number | null;
  isDelayed: boolean;
  delayMinutes: number;
  avgIntervalMs: number | null;
  recentLocation: string | null;
}

interface TrackingEnhancedProps {
  productId: string;
  events: TimelineEvent[];
  /** Expected max hours between events before flagging a delay. Default 24 h. */
  maxIntervalHours?: number;
  className?: string;
}

// ── ETA / delay logic ─────────────────────────────────────────────────────────

function computeTrackingStatus(
  events: TimelineEvent[],
  maxIntervalHours: number,
): TrackingStatus {
  if (events.length === 0) {
    return {
      lastEventAt: null,
      etaMs: null,
      isDelayed: false,
      delayMinutes: 0,
      avgIntervalMs: null,
      recentLocation: null,
    };
  }

  const sorted = [...events].sort(
    (a, b) => Number(a.timestamp) - Number(b.timestamp),
  );

  const lastEvent = sorted[sorted.length - 1];
  const lastEventAt = new Date(Number(lastEvent.timestamp) * 1000);
  const recentLocation = String(lastEvent.location ?? "");

  // Average interval between consecutive events
  let avgIntervalMs: number | null = null;
  if (sorted.length >= 2) {
    const intervals: number[] = [];
    for (let i = 1; i < sorted.length; i++) {
      const diff =
        (Number(sorted[i].timestamp) - Number(sorted[i - 1].timestamp)) * 1000;
      if (diff > 0) intervals.push(diff);
    }
    if (intervals.length > 0) {
      avgIntervalMs =
        intervals.reduce((s, v) => s + v, 0) / intervals.length;
    }
  }

  // ETA = last event time + avg interval (simple linear projection)
  const etaMs = avgIntervalMs ? lastEventAt.getTime() + avgIntervalMs : null;

  // Delay check: time since last event > maxIntervalHours
  const now = Date.now();
  const msSinceLastEvent = now - lastEventAt.getTime();
  const maxIntervalMs = maxIntervalHours * 60 * 60 * 1000;
  const isDelayed = msSinceLastEvent > maxIntervalMs;
  const delayMinutes = isDelayed
    ? Math.floor((msSinceLastEvent - maxIntervalMs) / 60_000)
    : 0;

  return { lastEventAt, etaMs, isDelayed, delayMinutes, avgIntervalMs, recentLocation };
}

// ── Component ─────────────────────────────────────────────────────────────────

export function TrackingEnhanced({
  productId,
  events,
  maxIntervalHours = 24,
  className,
}: Readonly<TrackingEnhancedProps>) {
  const status = React.useMemo(
    () => computeTrackingStatus(events, maxIntervalHours),
    [events, maxIntervalHours],
  );

  const trackingUrl = React.useMemo(() => {
    if (typeof window === "undefined") return "";
    return `${window.location.origin}/verify?product=${encodeURIComponent(productId)}`;
  }, [productId]);

  const [copied, setCopied] = React.useState(false);
  const handleCopy = React.useCallback(async () => {
    await navigator.clipboard.writeText(trackingUrl);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }, [trackingUrl]);

  return (
    <div className={cn("space-y-4", className)}>
      {/* ── Delay notification ── */}
      {status.isDelayed && (
        <div
          role="alert"
          aria-live="assertive"
          className="flex items-start gap-3 rounded-xl border border-amber-200 bg-amber-50 p-4"
        >
          <AlertTriangle className="mt-0.5 h-5 w-5 shrink-0 text-amber-600" />
          <div>
            <p className="font-semibold text-amber-900">Shipment Delay Detected</p>
            <p className="mt-0.5 text-sm text-amber-700">
              No update for{" "}
              <strong>
                {Math.floor(
                  (Date.now() - (status.lastEventAt?.getTime() ?? 0)) / 3_600_000,
                )}
              </strong>{" "}
              hours — {status.delayMinutes} min past the expected{" "}
              {maxIntervalHours}h interval. Verify with the carrier.
            </p>
          </div>
        </div>
      )}

      {/* ── Status card ── */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
        {/* Last event */}
        <div className="flex items-start gap-3 rounded-xl border border-zinc-200 bg-white p-4 shadow-sm">
          <Clock className="mt-0.5 h-5 w-5 shrink-0 text-zinc-400" />
          <div>
            <p className="text-xs font-medium text-zinc-500 uppercase tracking-wide">
              Last Update
            </p>
            <p className="mt-1 text-sm font-semibold text-zinc-900">
              {status.lastEventAt
                ? status.lastEventAt.toLocaleString()
                : "No events yet"}
            </p>
          </div>
        </div>

        {/* ETA */}
        <div className="flex items-start gap-3 rounded-xl border border-zinc-200 bg-white p-4 shadow-sm">
          <RefreshCw className="mt-0.5 h-5 w-5 shrink-0 text-zinc-400" />
          <div>
            <p className="text-xs font-medium text-zinc-500 uppercase tracking-wide">
              Predicted Next Update
            </p>
            <p className="mt-1 text-sm font-semibold text-zinc-900">
              {status.etaMs
                ? new Date(status.etaMs).toLocaleString()
                : "Insufficient data"}
            </p>
            {status.avgIntervalMs && (
              <p className="text-xs text-zinc-400 mt-0.5">
                Avg interval:{" "}
                {Math.round(status.avgIntervalMs / 60_000)} min
              </p>
            )}
          </div>
        </div>

        {/* Location */}
        <div className="flex items-start gap-3 rounded-xl border border-zinc-200 bg-white p-4 shadow-sm">
          <MapPin className="mt-0.5 h-5 w-5 shrink-0 text-zinc-400" />
          <div>
            <p className="text-xs font-medium text-zinc-500 uppercase tracking-wide">
              Last Known Location
            </p>
            <p className="mt-1 text-sm font-semibold text-zinc-900 truncate max-w-[160px]">
              {status.recentLocation || "Unknown"}
            </p>
            <Badge
              variant={status.isDelayed ? "destructive" : "default"}
              className="mt-1 text-xs"
            >
              {status.isDelayed ? "Delayed" : "On Track"}
            </Badge>
          </div>
        </div>
      </div>

      {/* ── Customer tracking portal ── */}
      <div className="rounded-xl border border-zinc-200 bg-white p-4 shadow-sm">
        <div className="flex items-center gap-2 mb-3">
          <ExternalLink className="h-4 w-4 text-zinc-500" />
          <h3 className="text-sm font-semibold text-zinc-800">Customer Tracking Portal</h3>
        </div>
        <p className="text-xs text-zinc-500 mb-3">
          Share this link with your customer for read-only shipment tracking.
        </p>
        <div className="flex flex-col gap-2 sm:flex-row sm:items-center">
          <input
            readOnly
            value={trackingUrl}
            aria-label="Customer tracking URL"
            className="flex-1 rounded-lg border border-zinc-200 bg-zinc-50 px-3 py-2 text-xs font-mono text-zinc-700 focus:outline-none"
          />
          <Button variant="outline" size="sm" onClick={handleCopy} className="shrink-0">
            {copied ? (
              <CheckCircle2 className="mr-1.5 h-3.5 w-3.5 text-green-600" />
            ) : (
              <Copy className="mr-1.5 h-3.5 w-3.5" />
            )}
            {copied ? "Copied!" : "Copy link"}
          </Button>
        </div>
      </div>
    </div>
  );
}
