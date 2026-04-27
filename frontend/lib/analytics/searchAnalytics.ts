/**
 * Search analytics tracking for user behavior insights
 */

import type { SearchAnalytics } from "@/lib/types/search";

class SearchAnalyticsTracker {
  private analytics: SearchAnalytics[] = [];
  private currentSearchId: string | null = null;
  private startTime: number = 0;

  /**
   * Start tracking a new search session
   */
  startSearch(query: string, category: string, filtersApplied: boolean): string {
    const searchId = `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    this.currentSearchId = searchId;
    this.startTime = Date.now();

    const analytics: SearchAnalytics = {
      searchId,
      query,
      category: category as "product" | "event" | "partner" | "document" | "analytics",
      timestamp: new Date(),
      resultCount: 0,
      filtersApplied,
    };

    this.analytics.push(analytics);
    this.saveToStorage();

    return searchId;
  }

  /**
   * Record search results
   */
  recordResults(searchId: string, resultCount: number): void {
    const analytics = this.analytics.find((a) => a.searchId === searchId);
    if (analytics) {
      analytics.resultCount = resultCount;
      this.saveToStorage();
    }
  }

  /**
   * Record a result click
   */
  recordClick(searchId: string, resultId: string): void {
    const analytics = this.analytics.find((a) => a.searchId === searchId);
    if (analytics) {
      analytics.clickedResult = resultId;
      analytics.timeToFirstClick = Date.now() - this.startTime;
      this.saveToStorage();
    }
  }

  /**
   * Get analytics data
   */
  getAnalytics(): SearchAnalytics[] {
    return this.analytics;
  }

  /**
   * Get analytics for a specific time range
   */
  getAnalyticsInRange(startDate: Date, endDate: Date): SearchAnalytics[] {
    return this.analytics.filter(
      (a) => a.timestamp >= startDate && a.timestamp <= endDate
    );
  }

  /**
   * Get search statistics
   */
  getStatistics() {
    const totalSearches = this.analytics.length;
    const searchesWithResults = this.analytics.filter((a) => a.resultCount > 0).length;
    const searchesWithClicks = this.analytics.filter((a) => a.clickedResult).length;
    const avgResults =
      totalSearches > 0
        ? this.analytics.reduce((sum, a) => sum + a.resultCount, 0) / totalSearches
        : 0;
    const avgTimeToClick =
      searchesWithClicks > 0
        ? this.analytics
            .filter((a) => a.timeToFirstClick)
            .reduce((sum, a) => sum + (a.timeToFirstClick || 0), 0) / searchesWithClicks
        : 0;

    // Get top queries
    const queryCounts = new Map<string, number>();
    this.analytics.forEach((a) => {
      queryCounts.set(a.query, (queryCounts.get(a.query) || 0) + 1);
    });
    const topQueries = Array.from(queryCounts.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, 10)
      .map(([query, count]) => ({ query, count }));

    // Get category distribution
    const categoryCounts = new Map<string, number>();
    this.analytics.forEach((a) => {
      categoryCounts.set(a.category, (categoryCounts.get(a.category) || 0) + 1);
    });
    const categoryDistribution = Array.from(categoryCounts.entries()).map(
      ([category, count]) => ({ category, count })
    );

    return {
      totalSearches,
      searchesWithResults,
      searchesWithClicks,
      avgResults: Math.round(avgResults * 100) / 100,
      avgTimeToClick: Math.round(avgTimeToClick),
      topQueries,
      categoryDistribution,
    };
  }

  /**
   * Clear analytics data
   */
  clearAnalytics(): void {
    this.analytics = [];
    this.currentSearchId = null;
    localStorage.removeItem("searchAnalytics");
  }

  /**
   * Save analytics to localStorage
   */
  private saveToStorage(): void {
    try {
      localStorage.setItem("searchAnalytics", JSON.stringify(this.analytics));
    } catch (e) {
      console.error("Failed to save search analytics:", e);
    }
  }

  /**
   * Load analytics from localStorage
   */
  loadFromStorage(): void {
    try {
      const stored = localStorage.getItem("searchAnalytics");
      if (stored) {
        this.analytics = JSON.parse(stored);
      }
    } catch (e) {
      console.error("Failed to load search analytics:", e);
    }
  }
}

// Singleton instance
export const searchAnalyticsTracker = new SearchAnalyticsTracker();

// Load analytics on initialization
if (typeof window !== "undefined") {
  searchAnalyticsTracker.loadFromStorage();
}

/**
 * Hook for search analytics
 */
export function useSearchAnalytics() {
  const trackSearch = (query: string, category: string, filtersApplied: boolean) => {
    return searchAnalyticsTracker.startSearch(query, category, filtersApplied);
  };

  const trackResults = (searchId: string, resultCount: number) => {
    searchAnalyticsTracker.recordResults(searchId, resultCount);
  };

  const trackClick = (searchId: string, resultId: string) => {
    searchAnalyticsTracker.recordClick(searchId, resultId);
  };

  const getAnalytics = () => {
    return searchAnalyticsTracker.getAnalytics();
  };

  const getStatistics = () => {
    return searchAnalyticsTracker.getStatistics();
  };

  const clearAnalytics = () => {
    searchAnalyticsTracker.clearAnalytics();
  };

  return {
    trackSearch,
    trackResults,
    trackClick,
    getAnalytics,
    getStatistics,
    clearAnalytics,
  };
}
