/**
 * Fuzzy search utility with autocomplete and relevance scoring
 */

import type { SearchFilters, SearchSortOption } from "@/lib/types/search";

/**
 * Levenshtein distance calculation for fuzzy matching
 */
function levenshteinDistance(a: string, b: string): number {
  const matrix = Array(b.length + 1)
    .fill(null)
    .map(() => Array(a.length + 1).fill(null));

  for (let i = 0; i <= a.length; i++) matrix[0][i] = i;
  for (let j = 0; j <= b.length; j++) matrix[j][0] = j;

  for (let j = 1; j <= b.length; j++) {
    for (let i = 1; i <= a.length; i++) {
      const indicator = a[i - 1] === b[j - 1] ? 0 : 1;
      matrix[j][i] = Math.min(
        matrix[j][i - 1] + 1,
        matrix[j - 1][i] + 1,
        matrix[j - 1][i - 1] + indicator
      );
    }
  }

  return matrix[b.length][a.length];
}

/**
 * Calculate similarity score between two strings (0-1, where 1 is exact match)
 */
export function calculateSimilarity(query: string, target: string): number {
  if (!query || !target) return 0;
  if (query === target) return 1;

  const queryLower = query.toLowerCase();
  const targetLower = target.toLowerCase();

  // Exact match (case-insensitive)
  if (queryLower === targetLower) return 1;

  // Contains match
  if (targetLower.includes(queryLower)) return 0.9;

  // Fuzzy match using Levenshtein distance
  const distance = levenshteinDistance(queryLower, targetLower);
  const maxLength = Math.max(query.length, target.length);
  const similarity = 1 - distance / maxLength;

  return Math.max(0, similarity);
}

/**
 * Calculate relevance score for a search result
 */
export function calculateRelevanceScore(
  query: string,
  result: { title: string; description?: string; tags?: string[] }
): number {
  let score = 0;
  const queryLower = query.toLowerCase();

  // Title match (highest weight)
  const titleSimilarity = calculateSimilarity(query, result.title);
  score += titleSimilarity * 0.5;

  // Description match (medium weight)
  if (result.description) {
    const descSimilarity = calculateSimilarity(query, result.description);
    score += descSimilarity * 0.3;
  }

  // Tags match (lower weight)
  if (result.tags && result.tags.length > 0) {
    const tagScores = result.tags.map((tag) => calculateSimilarity(query, tag));
    const maxTagScore = Math.max(...tagScores);
    score += maxTagScore * 0.2;
  }

  // Exact word match bonus
  const queryWords = queryLower.split(/\s+/);
  const titleWords = result.title.toLowerCase().split(/\s+/);
  const exactMatches = queryWords.filter((word) => titleWords.includes(word));
  score += exactMatches.length * 0.1;

  return Math.min(1, score);
}

/**
 * Perform fuzzy search on an array of items
 */
export function fuzzySearch<T extends { title: string; description?: string; tags?: string[] }>(
  items: T[],
  query: string,
  threshold: number = 0.3
): Array<{ item: T; score: number }> {
  if (!query || query.trim().length === 0) {
    return items.map((item) => ({ item, score: 1 }));
  }

  return items
    .map((item) => ({
      item,
      score: calculateRelevanceScore(query, item),
    }))
    .filter(({ score }) => score >= threshold)
    .sort((a, b) => b.score - a.score);
}

/**
 * Filter items based on search filters
 */
export function applyFilters<T>(
  items: T[],
  filters: SearchFilters,
  itemAccessor: (item: T) => {
    category?: string;
    date?: Date;
    location?: string;
    eventType?: string;
    status?: string;
    tags?: string[];
    [key: string]: string | number | boolean | Date | string[] | undefined;
  }
): T[] {
  return items.filter((item) => {
    const data = itemAccessor(item);

    // Category filter
    if (filters.categories && filters.categories.length > 0) {
      if (!data.category || !filters.categories.includes(data.category)) {
        return false;
      }
    }

    // Date range filter
    if (filters.dateRange) {
      if (!data.date) return false;
      const itemDate = new Date(data.date);
      if (itemDate < filters.dateRange.start || itemDate > filters.dateRange.end) {
        return false;
      }
    }

    // Location filter
    if (filters.locations && filters.locations.length > 0) {
      if (!data.location || !filters.locations.includes(data.location)) {
        return false;
      }
    }

    // Event type filter
    if (filters.eventTypes && filters.eventTypes.length > 0) {
      if (!data.eventType || !filters.eventTypes.includes(data.eventType)) {
        return false;
      }
    }

    // Status filter
    if (filters.status && filters.status.length > 0) {
      if (!data.status || !filters.status.includes(data.status)) {
        return false;
      }
    }

    // Tags filter
    if (filters.tags && filters.tags.length > 0) {
      if (!data.tags || !filters.tags.some((tag) => data.tags?.includes(tag))) {
        return false;
      }
    }

    // Custom filters
    if (filters.customFilters) {
      for (const [key, value] of Object.entries(filters.customFilters)) {
        if (data[key] !== value) {
          return false;
        }
      }
    }

    return true;
  });
}

/**
 * Sort search results based on sort option
 */
export function sortResults<T>(
  results: Array<{ item: T; score: number }>,
  sortBy: SearchSortOption,
  dateAccessor?: (item: T) => Date | undefined,
  nameAccessor?: (item: T) => string
): Array<{ item: T; score: number }> {
  const sorted = [...results];

  switch (sortBy) {
    case "relevance":
      return sorted.sort((a, b) => b.score - a.score);

    case "date_desc":
      if (dateAccessor) {
        return sorted.sort((a, b) => {
          const dateA = dateAccessor(a.item);
          const dateB = dateAccessor(b.item);
          if (!dateA) return 1;
          if (!dateB) return -1;
          return dateB.getTime() - dateA.getTime();
        });
      }
      return sorted;

    case "date_asc":
      if (dateAccessor) {
        return sorted.sort((a, b) => {
          const dateA = dateAccessor(a.item);
          const dateB = dateAccessor(b.item);
          if (!dateA) return 1;
          if (!dateB) return -1;
          return dateA.getTime() - dateB.getTime();
        });
      }
      return sorted;

    case "name_asc":
      if (nameAccessor) {
        return sorted.sort((a, b) => {
          const nameA = nameAccessor(a.item).toLowerCase();
          const nameB = nameAccessor(b.item).toLowerCase();
          return nameA.localeCompare(nameB);
        });
      }
      return sorted;

    case "name_desc":
      if (nameAccessor) {
        return sorted.sort((a, b) => {
          const nameA = nameAccessor(a.item).toLowerCase();
          const nameB = nameAccessor(b.item).toLowerCase();
          return nameB.localeCompare(nameA);
        });
      }
      return sorted;

    default:
      return sorted.sort((a, b) => b.score - a.score);
  }
}

/**
 * Generate autocomplete suggestions
 */
export function generateAutocompleteSuggestions(
  query: string,
  items: string[],
  maxSuggestions: number = 5
): string[] {
  if (!query || query.trim().length === 0) {
    return items.slice(0, maxSuggestions);
  }

  const scored = items
    .map((item) => ({
      item,
      score: calculateSimilarity(query, item),
    }))
    .filter(({ score }) => score >= 0.5)
    .sort((a, b) => b.score - a.score)
    .slice(0, maxSuggestions);

  return scored.map(({ item }) => item);
}

/**
 * Highlight matching text in search results
 */
export function highlightMatch(text: string, query: string): string {
  if (!query || query.trim().length === 0) {
    return text;
  }

  const regex = new RegExp(`(${escapeRegExp(query)})`, "gi");
  return text.replace(regex, "<mark>$1</mark>");
}

/**
 * Escape special regex characters
 */
function escapeRegExp(string: string): string {
  return string.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/**
 * Paginate search results
 */
export function paginateResults<T>(
  results: T[],
  offset: number = 0,
  limit: number = 20
): { results: T[]; total: number; hasMore: boolean } {
  const total = results.length;
  const paginatedResults = results.slice(offset, offset + limit);
  const hasMore = offset + limit < total;

  return {
    results: paginatedResults,
    total,
    hasMore,
  };
}

/**
 * Debounce function for search input
 */
export function debounce<T extends (...args: unknown[]) => void>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: ReturnType<typeof setTimeout> | null = null;

  return function executedFunction(...args: Parameters<T>) {
    const later = () => {
      timeout = null;
      func(...args);
    };

    if (timeout) clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };
}
