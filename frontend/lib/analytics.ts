import type { Metric } from "web-vitals";

type MonitoringEventType =
  | "web_vital"
  | "contract_interaction"
  | "error"
  | "performance_budget_breach";

type MonitoringPayload = {
  type: MonitoringEventType;
  timestamp: string;
  route?: string;
  environment: string;
  data: Record<string, unknown>;
};

export type PerformanceBudgetAlert = {
  metric: string;
  severity: "warning" | "critical";
  value: number;
  warningThreshold: number;
  criticalThreshold: number;
  context?: Record<string, unknown>;
};

const WEB_VITAL_BUDGETS = {
  CLS: { warning: 0.1, critical: 0.25 },
  FCP: { warning: 1800, critical: 3000 },
  LCP: { warning: 2500, critical: 4000 },
  INP: { warning: 200, critical: 500 },
  TTFB: { warning: 800, critical: 1800 },
} as const;

const CONTRACT_INTERACTION_BUDGET_MS = { warning: 1200, critical: 2500 } as const;

const budgetAlertSubscribers = new Set<(alert: PerformanceBudgetAlert) => void>();
let monitoringInitialized = false;

function getEnvironment(): string {
  return process.env.NODE_ENV ?? "development";
}

function getCurrentRoute(): string | undefined {
  if (typeof window === "undefined") return undefined;
  return window.location.pathname;
}

function getMonitoringEndpoint(): string | undefined {
  const endpoint = process.env.NEXT_PUBLIC_MONITORING_ENDPOINT;
  if (!endpoint || !endpoint.trim()) return undefined;
  return endpoint.trim();
}

function notifyBudgetAlert(alert: PerformanceBudgetAlert): void {
  budgetAlertSubscribers.forEach((callback) => callback(alert));
}

function checkBudget(
  metric: string,
  value: number,
  thresholds: { warning: number; critical: number },
  context?: Record<string, unknown>
): void {
  let severity: "warning" | "critical" | null = null;

  if (value >= thresholds.critical) {
    severity = "critical";
  } else if (value >= thresholds.warning) {
    severity = "warning";
  }

  if (!severity) return;

  const alert: PerformanceBudgetAlert = {
    metric,
    severity,
    value,
    warningThreshold: thresholds.warning,
    criticalThreshold: thresholds.critical,
    context,
  };

  notifyBudgetAlert(alert);
  void sendMonitoringPayload({
    type: "performance_budget_breach",
    data: {
      ...alert,
    },
  });
}

async function sendMonitoringPayload(event: {
  type: MonitoringEventType;
  data: Record<string, unknown>;
}): Promise<void> {
  const payload: MonitoringPayload = {
    type: event.type,
    data: event.data,
    timestamp: new Date().toISOString(),
    environment: getEnvironment(),
    route: getCurrentRoute(),
  };

  const endpoint = getMonitoringEndpoint();
  if (!endpoint) {
    if (getEnvironment() !== "production") {
      console.info("[monitoring]", payload);
    }
    return;
  }

  try {
    await fetch(endpoint, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
      keepalive: true,
    });
  } catch (error) {
    if (getEnvironment() !== "production") {
      console.error("[monitoring] failed to send payload", error);
    }
  }
}

export function subscribeToBudgetAlerts(
  callback: (alert: PerformanceBudgetAlert) => void
): () => void {
  budgetAlertSubscribers.add(callback);
  return () => {
    budgetAlertSubscribers.delete(callback);
  };
}

export function trackWebVital(metric: Metric): void {
  const safeValue = Number(metric.value ?? 0);
  void sendMonitoringPayload({
    type: "web_vital",
    data: {
      name: metric.name,
      id: metric.id,
      value: safeValue,
      rating: metric.rating,
      delta: metric.delta,
      navigationType: metric.navigationType,
    },
  });

  const budget = WEB_VITAL_BUDGETS[metric.name as keyof typeof WEB_VITAL_BUDGETS];
  if (budget) {
    checkBudget(metric.name, safeValue, budget);
  }
}

export function trackContractInteraction(params: {
  method: string;
  durationMs: number;
  success: boolean;
  errorMessage?: string;
  context?: Record<string, unknown>;
}): void {
  const durationMs = Number(params.durationMs);
  void sendMonitoringPayload({
    type: "contract_interaction",
    data: {
      method: params.method,
      durationMs,
      success: params.success,
      errorMessage: params.errorMessage,
      ...params.context,
    },
  });

  checkBudget("CONTRACT_INTERACTION_MS", durationMs, CONTRACT_INTERACTION_BUDGET_MS, {
    method: params.method,
    success: params.success,
  });
}

export function trackError(error: unknown, context?: Record<string, unknown>): void {
  const normalizedError =
    error instanceof Error
      ? {
          message: error.message,
          stack: error.stack,
          name: error.name,
        }
      : {
          message: String(error),
        };

  void sendMonitoringPayload({
    type: "error",
    data: {
      ...normalizedError,
      ...context,
    },
  });
}

export function initMonitoring(): void {
  if (monitoringInitialized || typeof window === "undefined") return;
  monitoringInitialized = true;

  window.addEventListener("error", (event) => {
    trackError(event.error ?? event.message, {
      source: "window.error",
      filename: event.filename,
      lineno: event.lineno,
      colno: event.colno,
    });
  });

  window.addEventListener("unhandledrejection", (event) => {
    trackError(event.reason, {
      source: "window.unhandledrejection",
    });
  });
}

export async function startWebVitalsTracking(): Promise<void> {
  if (typeof window === "undefined") return;

  const { onCLS, onFCP, onINP, onLCP, onTTFB } = await import("web-vitals");
  onCLS(trackWebVital);
  onFCP(trackWebVital);
  onINP(trackWebVital);
  onLCP(trackWebVital);
  onTTFB(trackWebVital);
}

