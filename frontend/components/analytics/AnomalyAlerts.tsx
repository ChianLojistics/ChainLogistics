"use client";

import * as React from "react";
import { AlertTriangle, CheckCircle2, ChevronDown, ChevronUp, ShieldAlert, XCircle } from "lucide-react";
import { cn } from "@/lib/utils";

// ── Types ─────────────────────────────────────────────────────────────────────

export type RiskLevel = "low" | "medium" | "high" | "critical";
export type AlertStatus = "open" | "acknowledged" | "resolved" | "suppressed";
export type AlertType =
  | "delay_predicted"
  | "route_disruption"
  | "env_anomaly"
  | "critical_risk";

export interface RoutingAlert {
  id: string;
  product_id: string;
  alert_type: AlertType;
  severity: RiskLevel;
  title: string;
  description: string;
  recommendations: string[];
  status: AlertStatus;
  triggered_score: number;
  created_at: string;
  acknowledged_by?: string;
  acknowledged_at?: string;
}

export interface AnomalyAlertsProps {
  alerts: RoutingAlert[];
  isLoading?: boolean;
  onAcknowledge?: (alertId: string) => void;
  onResolve?: (alertId: string) => void;
  className?: string;
}

// ── Helpers ───────────────────────────────────────────────────────────────────

const SEVERITY_CONFIG: Record<
  RiskLevel,
  { label: string; bgClass: string; borderClass: string; textClass: string; Icon: React.ElementType }
> = {
  critical: {
    label: "Critical",
    bgClass: "bg-red-50",
    borderClass: "border-red-300",
    textClass: "text-red-700",
    Icon: XCircle,
  },
  high: {
    label: "High",
    bgClass: "bg-orange-50",
    borderClass: "border-orange-300",
    textClass: "text-orange-700",
    Icon: ShieldAlert,
  },
  medium: {
    label: "Medium",
    bgClass: "bg-amber-50",
    borderClass: "border-amber-300",
    textClass: "text-amber-700",
    Icon: AlertTriangle,
  },
  low: {
    label: "Low",
    bgClass: "bg-blue-50",
    borderClass: "border-blue-300",
    textClass: "text-blue-700",
    Icon: AlertTriangle,
  },
};

const ALERT_TYPE_LABELS: Record<AlertType, string> = {
  delay_predicted: "Delay Predicted",
  route_disruption: "Route Disruption",
  env_anomaly: "Environmental Anomaly",
  critical_risk: "Critical Risk",
};

function formatRelativeTime(isoDate: string): string {
  const diff = Date.now() - new Date(isoDate).getTime();
  const mins = Math.floor(diff / 60_000);
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins}m ago`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h ago`;
  return `${Math.floor(hrs / 24)}d ago`;
}

function ScorePill({ score }: { score: number }) {
  const pct = Math.round(score * 100);
  const color =
    pct >= 85 ? "bg-red-100 text-red-700" :
    pct >= 60 ? "bg-orange-100 text-orange-700" :
    pct >= 35 ? "bg-amber-100 text-amber-700" :
    "bg-green-100 text-green-700";

  return (
    <span className={cn("rounded-full px-2 py-0.5 text-xs font-semibold tabular-nums", color)}>
      {pct}%
    </span>
  );
}

// ── Alert row ─────────────────────────────────────────────────────────────────

interface AlertRowProps {
  alert: RoutingAlert;
  onAcknowledge?: (id: string) => void;
  onResolve?: (id: string) => void;
}

function AlertRow({ alert, onAcknowledge, onResolve }: Readonly<AlertRowProps>) {
  const [expanded, setExpanded] = React.useState(false);
  const cfg = SEVERITY_CONFIG[alert.severity] ?? SEVERITY_CONFIG.medium;
  const { Icon } = cfg;

  return (
    <li
      className={cn(
        "rounded-lg border p-4 transition-colors",
        cfg.bgClass,
        cfg.borderClass,
        alert.status === "acknowledged" && "opacity-70"
      )}
    >
      <div className="flex items-start gap-3">
        <Icon
          className={cn("mt-0.5 h-5 w-5 shrink-0", cfg.textClass)}
          aria-hidden="true"
        />

        <div className="min-w-0 flex-1">
          {/* Header row */}
          <div className="flex flex-wrap items-center gap-2">
            <span
              className={cn(
                "rounded-full px-2 py-0.5 text-xs font-semibold uppercase tracking-wide",
                cfg.textClass
              )}
            >
              {cfg.label}
            </span>
            <span className="text-xs text-zinc-500">
              {ALERT_TYPE_LABELS[alert.alert_type]}
            </span>
            <ScorePill score={alert.triggered_score} />
            <span className="ml-auto text-xs text-zinc-400">
              {formatRelativeTime(alert.created_at)}
            </span>
          </div>

          {/* Title */}
          <p className={cn("mt-1 text-sm font-semibold", cfg.textClass)}>
            {alert.title}
          </p>

          {/* Description */}
          <p className="mt-0.5 text-sm text-zinc-600">{alert.description}</p>

          {/* Expandable recommendations */}
          {alert.recommendations.length > 0 ? (
            <div className="mt-2">
              <button
                type="button"
                onClick={() => setExpanded((v) => !v)}
                className={cn(
                  "inline-flex items-center gap-1 text-xs font-medium",
                  cfg.textClass
                )}
                aria-expanded={expanded}
              >
                {expanded ? (
                  <>
                    <ChevronUp className="h-3 w-3" aria-hidden="true" />
                    Hide recommendations
                  </>
                ) : (
                  <>
                    <ChevronDown className="h-3 w-3" aria-hidden="true" />
                    {alert.recommendations.length} recommendation
                    {alert.recommendations.length > 1 ? "s" : ""}
                  </>
                )}
              </button>

              {expanded ? (
                <ul className="mt-2 space-y-1">
                  {alert.recommendations.map((rec, i) => (
                    <li
                      key={i}
                      className="flex gap-2 text-xs text-zinc-700"
                    >
                      <span className="mt-0.5 h-3.5 w-3.5 shrink-0 text-zinc-400">→</span>
                      {rec}
                    </li>
                  ))}
                </ul>
              ) : null}
            </div>
          ) : null}

          {/* Actions */}
          {alert.status === "open" ? (
            <div className="mt-3 flex flex-wrap gap-2">
              {onAcknowledge ? (
                <button
                  type="button"
                  onClick={() => onAcknowledge(alert.id)}
                  className="rounded-md bg-white px-3 py-1 text-xs font-semibold text-zinc-700 ring-1 ring-zinc-300 hover:bg-zinc-50"
                >
                  Acknowledge
                </button>
              ) : null}
              {onResolve ? (
                <button
                  type="button"
                  onClick={() => onResolve(alert.id)}
                  className="inline-flex items-center gap-1 rounded-md bg-white px-3 py-1 text-xs font-semibold text-green-700 ring-1 ring-green-300 hover:bg-green-50"
                >
                  <CheckCircle2 className="h-3 w-3" aria-hidden="true" />
                  Resolve
                </button>
              ) : null}
            </div>
          ) : null}

          {alert.status === "acknowledged" && alert.acknowledged_by ? (
            <p className="mt-2 text-xs text-zinc-400">
              Acknowledged by {alert.acknowledged_by}
              {alert.acknowledged_at
                ? ` · ${formatRelativeTime(alert.acknowledged_at)}`
                : null}
            </p>
          ) : null}
        </div>
      </div>
    </li>
  );
}

// ── Summary bar ───────────────────────────────────────────────────────────────

interface SummaryBarProps {
  alerts: RoutingAlert[];
}

function SummaryBar({ alerts }: Readonly<SummaryBarProps>) {
  const counts = React.useMemo(() => {
    return alerts.reduce(
      (acc, a) => {
        acc[a.severity] = (acc[a.severity] ?? 0) + 1;
        return acc;
      },
      {} as Record<string, number>
    );
  }, [alerts]);

  const chips: Array<{ level: RiskLevel; label: string; color: string }> = [
    { level: "critical", label: "Critical", color: "bg-red-100 text-red-700" },
    { level: "high", label: "High", color: "bg-orange-100 text-orange-700" },
    { level: "medium", label: "Medium", color: "bg-amber-100 text-amber-700" },
    { level: "low", label: "Low", color: "bg-blue-100 text-blue-700" },
  ];

  return (
    <div className="flex flex-wrap gap-2">
      {chips.map(({ level, label, color }) =>
        counts[level] ? (
          <span
            key={level}
            className={cn(
              "rounded-full px-2.5 py-0.5 text-xs font-semibold",
              color
            )}
          >
            {counts[level]} {label}
          </span>
        ) : null
      )}
    </div>
  );
}

// ── Main component ────────────────────────────────────────────────────────────

const SKELETON_COUNT = 3;

export function AnomalyAlerts({
  alerts,
  isLoading = false,
  onAcknowledge,
  onResolve,
  className,
}: Readonly<AnomalyAlertsProps>) {
  // Sort: critical first, then by recency
  const sorted = React.useMemo(() => {
    const order: Record<RiskLevel, number> = {
      critical: 0,
      high: 1,
      medium: 2,
      low: 3,
    };
    return [...alerts].sort((a, b) => {
      const diff = order[a.severity] - order[b.severity];
      if (diff !== 0) return diff;
      return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
    });
  }, [alerts]);

  return (
    <section
      className={cn(
        "rounded-xl border border-zinc-200 bg-white p-5 shadow-sm",
        className
      )}
      aria-label="Proactive routing alerts"
    >
      {/* Header */}
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div>
          <h2 className="text-sm font-semibold text-zinc-900">
            Routing Anomaly Alerts
          </h2>
          <p className="mt-0.5 text-xs text-zinc-500">
            Proactive alerts from the GNN anomaly detection engine
          </p>
        </div>
        {!isLoading && alerts.length > 0 ? (
          <SummaryBar alerts={alerts} />
        ) : null}
      </div>

      {/* Body */}
      <div className="mt-4">
        {isLoading ? (
          <ul className="space-y-3" aria-label="Loading alerts">
            {Array.from({ length: SKELETON_COUNT }, (_, i) => (
              <li
                key={i}
                className="h-20 animate-pulse rounded-lg bg-zinc-100"
                aria-hidden="true"
              />
            ))}
          </ul>
        ) : sorted.length === 0 ? (
          <div className="flex flex-col items-center gap-2 rounded-lg border border-dashed border-zinc-200 bg-zinc-50 py-10 text-center">
            <CheckCircle2
              className="h-8 w-8 text-green-400"
              aria-hidden="true"
            />
            <p className="text-sm font-medium text-zinc-700">
              No active routing alerts
            </p>
            <p className="text-xs text-zinc-400">
              All shipments are within expected parameters.
            </p>
          </div>
        ) : (
          <ul className="space-y-3">
            {sorted.map((alert) => (
              <AlertRow
                key={alert.id}
                alert={alert}
                onAcknowledge={onAcknowledge}
                onResolve={onResolve}
              />
            ))}
          </ul>
        )}
      </div>
    </section>
  );
}
