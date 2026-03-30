/**
 * Centralized configuration constants for the frontend application.
 *
 * Collecting magic numbers and hardcoded values here improves
 * maintainability and makes configuration changes straightforward.
 */

// ---------------------------------------------------------------------------
// UI Timing (milliseconds)
// ---------------------------------------------------------------------------

/** Duration a "copied" tooltip stays visible after clipboard copy. */
export const COPY_FEEDBACK_DURATION_MS = 2000;

/** Simulated delay for product registration placeholder. */
export const PRODUCT_REGISTRATION_DELAY_MS = 2000;

/** Simulated delay for event tracking form submission placeholder. */
export const EVENT_TRACKING_SUBMIT_DELAY_MS = 1500;

// ---------------------------------------------------------------------------
// Dashboard
// ---------------------------------------------------------------------------

/** Default polling interval for the dashboard auto-refresh (ms). */
export const DASHBOARD_REFRESH_INTERVAL_MS = 30_000;

/** Number of recent events shown in the dashboard activity feed. */
export const DASHBOARD_RECENT_EVENTS_LIMIT = 12;

// ---------------------------------------------------------------------------
// Wallet
// ---------------------------------------------------------------------------

/** Timeout for wallet connection requests (ms). */
export const WALLET_CONNECTION_TIMEOUT_MS = 20_000;
