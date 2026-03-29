"use client";

import * as React from "react";
import {
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import {
  Activity,
  Boxes,
  MapPinned,
  RefreshCw,
  TrendingUp,
} from "lucide-react";

import type { Product } from "@/lib/types/product";
import type { TimelineEvent } from "@/lib/types/tracking";
import { useWalletStore } from "@/lib/state/wallet.store";
import { getProductsByOwner } from "@/lib/contract/products";
import { fetchProductEvents } from "@/lib/contract/events";
import { cn } from "@/lib/utils";
import { DASHBOARD_REFRESH_INTERVAL_MS, DASHBOARD_RECENT_EVENTS_LIMIT } from "@/lib/constants";

import { StatCard } from "@/components/analytics/StatCard";
import { EventsChart } from "@/components/analytics/EventsChart";
import { ActivityFeed } from "@/components/analytics/ActivityFeed";

type EventsByTypeDatum = { type: string; count: number };
type ActivityDatum = { date: string; count: number };

function formatDay(tsSeconds: number) {
  const d = new Date(tsSeconds * 1000);
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

export default function DashboardPage() {
  const { publicKey, status } = useWalletStore();

  const [products, setProducts] = React.useState<Product[]>([]);
  const [events, setEvents] = React.useState<TimelineEvent[]>([]);
  const [isLoading, setIsLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);
  const [lastUpdatedAt, setLastUpdatedAt] = React.useState<number | null>(null);

  const load = React.useCallback(async () => {
    if (!publicKey) return;

    setIsLoading(true);
    setError(null);

    try {
      const fetchedProducts = await getProductsByOwner(publicKey);
      setProducts(fetchedProducts);

      if (fetchedProducts.length === 0) {
        setEvents([]);
        setLastUpdatedAt(Date.now());
        return;
      }

      const settled = await Promise.allSettled(
        fetchedProducts.map((p) => fetchProductEvents(p.id))
      );
      const all = settled
        .filter((r): r is PromiseFulfilledResult<TimelineEvent[]> => r.status === "fulfilled")
        .flatMap((r) => r.value);

      all.sort((a, b) => b.timestamp - a.timestamp);
      setEvents(all);
      setLastUpdatedAt(Date.now());

      const rejectedCount = settled.filter((r) => r.status === "rejected").length;
      if (rejectedCount > 0 && all.length === 0) {
        setError(
          "Events could not be loaded (contract not configured or unavailable). Showing product-only insights."
        );
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load dashboard data");
    } finally {
      setIsLoading(false);
    }
  }, [publicKey]);

  React.useEffect(() => {
    if (status !== "connected" || !publicKey) return;
    load();
  }, [status, publicKey, load]);

  React.useEffect(() => {
    if (status !== "connected" || !publicKey) return;

    let intervalId: ReturnType<typeof setInterval> | null = null;

    const startPolling = () => {
      if (intervalId) return;
      intervalId = setInterval(() => {
        void load();
      }, DASHBOARD_REFRESH_INTERVAL_MS);
    };

    const stopPolling = () => {
      if (intervalId) {
        clearInterval(intervalId);
        intervalId = null;
      }
    };

    const handleVisibilityChange = () => {
      if (document.hidden) {
        stopPolling();
      } else {
        // Refresh immediately when the user returns, then resume polling.
        void load();
        startPolling();
      }
    };

    // Only poll while the tab is visible.
    if (!document.hidden) {
      startPolling();
    }

    document.addEventListener("visibilitychange", handleVisibilityChange);

    return () => {
      stopPolling();
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, [status, publicKey, load]);

  const totalProducts = products.length;
  const totalEvents = events.length;

  const eventsByType: EventsByTypeDatum[] = React.useMemo(() => {
    const map = new Map<string, number>();
    for (const e of events) {
      const t = e.event_type || "UNKNOWN";
      map.set(t, (map.get(t) ?? 0) + 1);
    }
    return Array.from(map.entries()).map(([type, count]) => ({ type, count }));
  }, [events]);

  const activityOverTime: ActivityDatum[] = React.useMemo(() => {
    const map = new Map<string, number>();
    for (const e of events) {
      const day = formatDay(e.timestamp);
      map.set(day, (map.get(day) ?? 0) + 1);
    }
    return Array.from(map.entries())
      .sort((a, b) => a[0].localeCompare(b[0]))
      .map(([date, count]) => ({ date, count }));
  }, [events]);

  const topOrigins = React.useMemo(() => {
    const map = new Map<string, number>();
    for (const p of products) {
      const origin = p.origin?.location || "Unknown";
      map.set(origin, (map.get(origin) ?? 0) + 1);
    }
    return Array.from(map.entries())
      .map(([origin, count]) => ({ origin, count }))
      .sort((a, b) => b.count - a.count)
      .slice(0, 5);
  }, [products]);

  const recentEvents = React.useMemo(() => events.slice(0, DASHBOARD_RECENT_EVENTS_LIMIT), [events]);

  const topOriginDescription = React.useMemo(() => {
    if (isLoading) return undefined;
    const first = topOrigins[0];
    if (!first) return "No products yet.";
    return `${first.count} product${first.count === 1 ? "" : "s"}`;
  }, [isLoading, topOrigins]);

  const canLoad = status === "connected" && Boolean(publicKey);

  return (
    <main className="mx-auto max-w-6xl px-6 py-10">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <h1 className="text-2xl font-semibold text-zinc-900">Dashboard</h1>
          <p className="mt-1 text-sm text-zinc-500">
            Supply chain analytics for your registered products.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => void load()}
            disabled={!canLoad || isLoading}
            className={cn(
              "inline-flex items-center gap-2 rounded-lg border px-4 py-2 text-sm font-semibold",
              canLoad
                ? "border-zinc-200 bg-white text-zinc-900 hover:bg-zinc-50"
                : "border-zinc-200 bg-zinc-50 text-zinc-400",
              isLoading && "opacity-50"
            )}
          >
            <RefreshCw className={cn("h-4 w-4", isLoading && "animate-spin")} />
            Refresh
          </button>
        </div>
      </div>

      {!canLoad ? (
        <div className="mt-8 rounded-xl border border-zinc-200 bg-white p-8 shadow-sm">
          <h2 className="text-lg font-semibold text-zinc-900">Connect your wallet</h2>
          <p className="mt-1 text-sm text-zinc-600">
            Connect a wallet to load products and events for analytics.
          </p>
        </div>
      ) : null}

      {error ? (
        <div className="mt-6 rounded-xl border border-amber-200 bg-amber-50 p-4 text-sm text-amber-900">
          {error}
        </div>
      ) : null}

      <div className="mt-8 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <StatCard
          label="Total products"
          value={isLoading ? "—" : totalProducts}
          description="Products currently visible to your wallet."
          icon={<Boxes className="h-6 w-6" />}
        />
        <StatCard
          label="Total events"
          value={isLoading ? "—" : totalEvents}
          description="All loaded tracking events."
          icon={<Activity className="h-6 w-6" />}
        />
        <StatCard
          label="Top origin"
          value={isLoading ? "—" : (topOrigins[0]?.origin ?? "—")}
          description={topOriginDescription}
          icon={<MapPinned className="h-6 w-6" />}
        />
        <StatCard
          label="Last updated"
          value={lastUpdatedAt ? new Date(lastUpdatedAt).toLocaleTimeString() : "—"}
          description="Auto-refreshes while tab is visible."
          icon={<TrendingUp className="h-6 w-6" />}
        />
      </div>

      <div className="mt-6 grid grid-cols-1 gap-6 lg:grid-cols-2">
        <EventsChart data={eventsByType} />

        <div className="rounded-xl border border-zinc-200 bg-white p-5 shadow-sm">
          <div>
            <h2 className="text-sm font-semibold text-zinc-900">Activity over time</h2>
            <p className="mt-1 text-sm text-zinc-500">
              Events grouped by day.
            </p>
          </div>
          <div className="mt-4 h-72">
            {isLoading ? (
              <div className="h-full rounded-lg bg-zinc-100 animate-pulse" aria-hidden="true" />
            ) : activityOverTime.length === 0 ? (
              <div className="h-full rounded-lg border border-dashed border-zinc-200 bg-zinc-50 flex items-center justify-center text-sm text-zinc-500">
                No activity yet.
              </div>
            ) : (
              <ResponsiveContainer width="100%" height="100%">
                <LineChart data={activityOverTime} margin={{ top: 10, right: 10, left: 0, bottom: 10 }}>
                  <XAxis dataKey="date" tick={{ fontSize: 12 }} />
                  <YAxis allowDecimals={false} tick={{ fontSize: 12 }} />
                  <Tooltip />
                  <Line type="monotone" dataKey="count" stroke="#2563eb" strokeWidth={2} dot={false} />
                </LineChart>
              </ResponsiveContainer>
            )}
          </div>
        </div>
      </div>

      <div className="mt-6 grid grid-cols-1 gap-6 lg:grid-cols-3">
        <div className="rounded-xl border border-zinc-200 bg-white p-5 shadow-sm lg:col-span-1">
          <div>
            <h2 className="text-sm font-semibold text-zinc-900">Top origins</h2>
            <p className="mt-1 text-sm text-zinc-500">Most common product origins.</p>
          </div>

          <div className="mt-4">
            {isLoading ? (
              <div className="space-y-3" aria-hidden="true">
                {Array.from({ length: 5 }, (_, i) => (
                  <div key={i} className="h-10 rounded-lg bg-zinc-100 animate-pulse" />
                ))}
              </div>
            ) : topOrigins.length === 0 ? (
              <div className="rounded-lg border border-dashed border-zinc-200 bg-zinc-50 p-6 text-sm text-zinc-500">
                No products yet.
              </div>
            ) : (
              <ul className="space-y-3">
                {topOrigins.map((o) => (
                  <li key={o.origin} className="flex items-center justify-between gap-3">
                    <span className="text-sm font-medium text-zinc-900 truncate">{o.origin}</span>
                    <span className="text-sm text-zinc-600">{o.count}</span>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </div>

        <ActivityFeed events={recentEvents} isLoading={isLoading} className="lg:col-span-2" />
      </div>
    </main>
  );
}
