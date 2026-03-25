# Monitoring and Performance

This project includes frontend monitoring for user experience and blockchain call reliability.

## What is tracked

- Core Web Vitals: `CLS`, `FCP`, `LCP`, `INP`, `TTFB`
- Contract interaction analytics:
  - method name
  - latency (`durationMs`)
  - success/failure
  - context (e.g. product/event id, result count)
- Error events:
  - runtime errors (`window.error`)
  - unhandled promise rejections
  - React error boundaries (`ErrorBoundary`, `app/error.tsx`)

## Implementation points

- Monitoring core: `frontend/lib/analytics.ts`
- Auto-bootstrap in app root: `frontend/components/analytics/MonitoringBootstrap.tsx`
- Budget alert notifications: `frontend/components/analytics/PerformanceBudgetAlerts.tsx`
- Contract instrumentation:
  - `frontend/lib/stellar/contractClient.ts`
  - `frontend/lib/contract/events.ts`

## Performance budgets

### Core Web Vitals

- `CLS`: warning `0.10`, critical `0.25`
- `FCP`: warning `1800ms`, critical `3000ms`
- `LCP`: warning `2500ms`, critical `4000ms`
- `INP`: warning `200ms`, critical `500ms`
- `TTFB`: warning `800ms`, critical `1800ms`

### Contract interactions

- `CONTRACT_INTERACTION_MS`: warning `1200ms`, critical `2500ms`

When a metric crosses a threshold, the system emits a `performance_budget_breach` event and raises a client alert.

## Dashboard integration

Set `NEXT_PUBLIC_MONITORING_ENDPOINT` to stream events to your monitoring backend.

Example:

```bash
NEXT_PUBLIC_MONITORING_ENDPOINT=https://monitoring.example.com/events
```

Payload format:

```json
{
  "type": "web_vital | contract_interaction | error | performance_budget_breach",
  "timestamp": "2026-03-25T12:00:00.000Z",
  "route": "/products/PROD-1",
  "environment": "production",
  "data": {}
}
```

If no endpoint is configured, events are logged locally in non-production environments.

