"use client";

import { useEffect } from "react";
import { subscribeToBudgetAlerts } from "@/lib/analytics";
import { toast } from "sonner";

export function PerformanceBudgetAlerts() {
  useEffect(() => {
    const unsubscribe = subscribeToBudgetAlerts((alert) => {
      const prefix = alert.severity === "critical" ? "Critical" : "Warning";
      toast.warning(`${prefix}: ${alert.metric} budget exceeded`, {
        description: `Observed ${alert.value.toFixed(2)} (warn ${alert.warningThreshold}, critical ${alert.criticalThreshold}).`,
      });
    });

    return unsubscribe;
  }, []);

  return null;
}

