/**
 * Simple client-side rate limiter for API and contract calls.
 * Tracks call timestamps per key and rejects calls that exceed the limit
 * within the configured time window.
 */
export class RateLimiter {
  private readonly maxCalls: number;
  private readonly windowMs: number;
  private readonly timestamps: Map<string, number[]> = new Map();

  constructor(maxCalls: number, windowMs: number) {
    this.maxCalls = maxCalls;
    this.windowMs = windowMs;
  }

  /**
   * Returns true if the call is allowed, false if rate-limited.
   * Automatically prunes expired timestamps.
   */
  check(key: string): boolean {
    const now = Date.now();
    const calls = this.timestamps.get(key) ?? [];
    const recent = calls.filter((t) => now - t < this.windowMs);

    if (recent.length >= this.maxCalls) {
      this.timestamps.set(key, recent);
      return false;
    }

    recent.push(now);
    this.timestamps.set(key, recent);
    return true;
  }

  /**
   * Resets the rate limiter state for a specific key, or all keys if none provided.
   */
  reset(key?: string): void {
    if (key) {
      this.timestamps.delete(key);
    } else {
      this.timestamps.clear();
    }
  }
}

/** Default limiter: 5 calls per 60 seconds per action. */
export const apiRateLimiter = new RateLimiter(5, 60_000);
