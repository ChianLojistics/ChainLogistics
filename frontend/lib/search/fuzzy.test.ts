import { describe, expect, it } from "vitest";

import { fuzzyScore } from "./fuzzy";

describe("fuzzyScore", () => {
  it("matches exact substring", () => {
    const res = fuzzyScore("app", "apple");
    expect(res.matched).toBe(true);
    expect(res.score).toBeGreaterThan(0);
  });

  it("matches non-contiguous characters", () => {
    const res = fuzzyScore("apl", "apple");
    expect(res.matched).toBe(true);
  });

  it("does not match if characters are missing", () => {
    const res = fuzzyScore("zzz", "apple");
    expect(res.matched).toBe(false);
    expect(res.score).toBe(0);
  });

  it("prefers word-start matches", () => {
    const start = fuzzyScore("app", "apple").score;
    const mid = fuzzyScore("app", "snapple").score;
    expect(start).toBeGreaterThan(mid);
  });
});
