"use client";

import { useEffect } from "react";
import { initMonitoring, startWebVitalsTracking, trackError } from "@/lib/analytics";

export function MonitoringBootstrap() {
  useEffect(() => {
    initMonitoring();
    startWebVitalsTracking().catch((error) => {
      trackError(error, {
        source: "monitoring.startWebVitalsTracking",
      });
    });
  }, []);

  return null;
}

