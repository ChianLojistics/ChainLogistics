"use client";

import { useEffect } from "react";
import { initMonitoring, startWebVitalsTracking, trackError } from "@/lib/analytics";

export function MonitoringBootstrap() {
  useEffect(() => {
    initMonitoring();
    const connection = (navigator as unknown as { connection?: { saveData?: boolean; effectiveType?: string } })
      .connection;
    const saveData = connection?.saveData;
    const effectiveType = connection?.effectiveType;
    const slowConnection = effectiveType === "slow-2g" || effectiveType === "2g";

    if (saveData || slowConnection) return;

    startWebVitalsTracking().catch((error) => {
      trackError(error, {
        source: "monitoring.startWebVitalsTracking",
      });
    });
  }, []);

  return null;
}
