import { describe, expect, it, beforeEach } from "vitest";

import { loadSavedSearches, persistSavedSearches } from "./savedSearches";

describe("savedSearches", () => {
  const storageKey = "test.savedSearches";

  beforeEach(() => {
    localStorage.clear();
  });

  it("returns empty array when nothing stored", () => {
    expect(loadSavedSearches(storageKey)).toEqual([]);
  });

  it("persists and loads searches", () => {
    const items = [
      {
        id: "1",
        name: "My search",
        createdAt: 1,
        filters: { q: "abc", category: "food" },
      },
    ];

    persistSavedSearches(storageKey, items);
    expect(loadSavedSearches(storageKey)).toEqual(items);
  });

  it("handles invalid JSON gracefully", () => {
    localStorage.setItem(storageKey, "not-json");
    expect(loadSavedSearches(storageKey)).toEqual([]);
  });
});
