import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";

import { AnomalyAlerts } from "@/components/analytics/AnomalyAlerts";
import type { RoutingAlert } from "@/components/analytics/AnomalyAlerts";

// ── Fixtures ──────────────────────────────────────────────────────────────────

function makeAlert(overrides: Partial<RoutingAlert> = {}): RoutingAlert {
  return {
    id: "alert-001",
    product_id: "PROD-001",
    alert_type: "delay_predicted",
    severity: "high",
    title: "Delay Predicted for shipment of PROD-001",
    description: "Temporal anomaly score 0.72: shipment is at risk of missing planned arrival.",
    recommendations: [
      "Transit time is 2.5σ above historical mean – notify downstream recipient.",
      "Monitor shipment closely over the next 6 hours.",
    ],
    status: "open",
    triggered_score: 0.72,
    created_at: new Date(Date.now() - 5 * 60 * 1000).toISOString(), // 5 min ago
    ...overrides,
  };
}

const CRITICAL_ALERT = makeAlert({
  id: "alert-crit",
  severity: "critical",
  alert_type: "critical_risk",
  title: "Critical risk detected for PROD-002",
  triggered_score: 0.91,
});

const MEDIUM_ALERT = makeAlert({
  id: "alert-med",
  severity: "medium",
  alert_type: "env_anomaly",
  title: "Environmental anomaly on shipment of PROD-003",
  triggered_score: 0.45,
});

// ── Tests ──────────────────────────────────────────────────────────────────────

describe("AnomalyAlerts", () => {
  describe("empty state", () => {
    it("renders empty state when no alerts are provided", () => {
      render(<AnomalyAlerts alerts={[]} />);
      expect(
        screen.getByText("No active routing alerts")
      ).toBeDefined();
    });

    it("renders loading skeleton when isLoading is true", () => {
      render(<AnomalyAlerts alerts={[]} isLoading />);
      const skeletons = screen.getByLabelText("Loading alerts");
      expect(skeletons).toBeDefined();
      // No alert text should appear while loading
      expect(screen.queryByText("No active routing alerts")).toBeNull();
    });
  });

  describe("alert rendering", () => {
    it("renders alert title and description", () => {
      render(<AnomalyAlerts alerts={[makeAlert()]} />);
      expect(screen.getByText("Delay Predicted for shipment of PROD-001")).toBeDefined();
      expect(
        screen.getByText(
          "Temporal anomaly score 0.72: shipment is at risk of missing planned arrival."
        )
      ).toBeDefined();
    });

    it("shows score pill with correct percentage", () => {
      render(<AnomalyAlerts alerts={[makeAlert()]} />);
      // triggered_score = 0.72 → 72%
      expect(screen.getByText("72%")).toBeDefined();
    });

    it("renders severity badge", () => {
      render(<AnomalyAlerts alerts={[makeAlert({ severity: "critical" })]} />);
      expect(screen.getAllByText("Critical").length).toBeGreaterThan(0);
    });

    it("renders alert type label", () => {
      render(<AnomalyAlerts alerts={[makeAlert()]} />);
      expect(screen.getByText("Delay Predicted")).toBeDefined();
    });
  });

  describe("recommendations", () => {
    it("shows recommendation count toggle", () => {
      render(<AnomalyAlerts alerts={[makeAlert()]} />);
      expect(screen.getByText(/2 recommendations/)).toBeDefined();
    });

    it("expands recommendations on click", () => {
      render(<AnomalyAlerts alerts={[makeAlert()]} />);
      const toggle = screen.getByText(/2 recommendations/);
      fireEvent.click(toggle);
      expect(
        screen.getByText(/Transit time is 2\.5σ above historical mean/)
      ).toBeDefined();
    });

    it("collapses recommendations on second click", () => {
      render(<AnomalyAlerts alerts={[makeAlert()]} />);
      const toggle = screen.getByText(/2 recommendations/);
      fireEvent.click(toggle); // expand
      fireEvent.click(toggle); // collapse
      expect(
        screen.queryByText(/Transit time is 2\.5σ above historical mean/)
      ).toBeNull();
    });

    it("hides recommendation toggle when none exist", () => {
      render(<AnomalyAlerts alerts={[makeAlert({ recommendations: [] })]} />);
      expect(screen.queryByText(/recommendations/)).toBeNull();
    });
  });

  describe("alert actions", () => {
    it("calls onAcknowledge with the alert id", () => {
      const onAcknowledge = vi.fn();
      render(<AnomalyAlerts alerts={[makeAlert()]} onAcknowledge={onAcknowledge} />);
      fireEvent.click(screen.getByText("Acknowledge"));
      expect(onAcknowledge).toHaveBeenCalledWith("alert-001");
    });

    it("calls onResolve with the alert id", () => {
      const onResolve = vi.fn();
      render(<AnomalyAlerts alerts={[makeAlert()]} onResolve={onResolve} />);
      fireEvent.click(screen.getByText("Resolve"));
      expect(onResolve).toHaveBeenCalledWith("alert-001");
    });

    it("hides action buttons for acknowledged alerts", () => {
      render(
        <AnomalyAlerts
          alerts={[makeAlert({ status: "acknowledged", acknowledged_by: "ops-team" })]}
          onAcknowledge={vi.fn()}
          onResolve={vi.fn()}
        />
      );
      expect(screen.queryByText("Acknowledge")).toBeNull();
      expect(screen.queryByText("Resolve")).toBeNull();
      expect(screen.getByText(/Acknowledged by ops-team/)).toBeDefined();
    });
  });

  describe("sort order", () => {
    it("renders critical alerts before high severity", () => {
      render(
        <AnomalyAlerts alerts={[makeAlert(), CRITICAL_ALERT]} />
      );
      const titles = screen.getAllByRole("paragraph").map((el) => el.textContent ?? "");
      const criticalIdx = titles.findIndex((t) => t.includes("Critical risk"));
      const highIdx = titles.findIndex((t) => t.includes("Delay Predicted"));
      expect(criticalIdx).toBeLessThan(highIdx);
    });
  });

  describe("summary bar", () => {
    it("shows summary counts when alerts exist", () => {
      render(<AnomalyAlerts alerts={[makeAlert(), CRITICAL_ALERT, MEDIUM_ALERT]} />);
      // Should show "1 Critical", "1 High", "1 Medium"
      expect(screen.getByText("1 Critical")).toBeDefined();
      expect(screen.getByText("1 High")).toBeDefined();
      expect(screen.getByText("1 Medium")).toBeDefined();
    });

    it("does not show summary bar when no alerts", () => {
      render(<AnomalyAlerts alerts={[]} />);
      expect(screen.queryByText(/Critical/)).toBeNull();
    });
  });

  describe("accessibility", () => {
    it("section has aria-label", () => {
      render(<AnomalyAlerts alerts={[]} />);
      expect(
        screen.getByRole("region", { name: "Proactive routing alerts" })
      ).toBeDefined();
    });

    it("expand button has aria-expanded attribute", () => {
      render(<AnomalyAlerts alerts={[makeAlert()]} />);
      const toggle = screen.getByRole("button", { name: /recommendations/ });
      expect(toggle.getAttribute("aria-expanded")).toBe("false");
      fireEvent.click(toggle);
      expect(toggle.getAttribute("aria-expanded")).toBe("true");
    });
  });
});
