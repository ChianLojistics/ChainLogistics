/**
 * React hooks for search functionality
 */

import { useCallback, useEffect, useMemo, useState } from "react";
import type {
  SearchCategory,
  SearchFilters,
  SearchQuery,
  SearchSortOption,
  SavedSearch,
  SearchHistoryItem,
} from "@/lib/types/search";
import {
  fuzzySearch,
  applyFilters,
  sortResults,
  generateAutocompleteSuggestions,
  debounce,
  paginateResults,
} from "@/lib/utils/search";

/**
 * Hook for search functionality with debouncing and caching
 */
export function useSearch<T extends { title: string; description?: string; tags?: string[] }>(
  items: T[],
  options: {
    threshold?: number;
    debounceMs?: number;
    itemAccessor?: (item: T) => {
      category?: string;
      date?: Date;
      location?: string;
      eventType?: string;
      status?: string;
      tags?: string[];
      [key: string]: any;
    };
    dateAccessor?: (item: T) => Date | undefined;
    nameAccessor?: (item: T) => string;
  } = {}
) {
  const {
    threshold = 0.3,
    debounceMs = 300,
    itemAccessor = (item) => item as any,
    dateAccessor,
    nameAccessor,
  } = options;

  const [query, setQuery] = useState("");
  const [filters, setFilters] = useState<SearchFilters>({});
  const [sortBy, setSortBy] = useState<SearchSortOption>("relevance");
  const [category, setCategory] = useState<SearchCategory>("product");
  const [page, setPage] = useState(0);
  const [isLoading, setIsLoading] = useState(false);

  // Perform search
  const searchResults = useMemo(() => {
    let results = fuzzySearch(items, query, threshold);

    // Apply filters
    if (Object.keys(filters).length > 0) {
      const filteredItems = applyFilters(items, filters, itemAccessor);
      results = fuzzySearch(filteredItems, query, threshold);
    }

    // Sort results
    results = sortResults(results, sortBy, dateAccessor, nameAccessor);

    // Convert to SearchResult format
    return results.map(({ item, score }) => ({
      id: (item as { id?: string }).id || String(Math.random()),
      type: category,
      title: item.title,
      description: item.description,
      relevanceScore: score,
      data: item,
    }));
  }, [items, query, threshold, filters, sortBy, category, itemAccessor, dateAccessor, nameAccessor]);

  // Debounced search function
  const debouncedSearch = useMemo(
    () =>
      debounce((searchQuery: string) => {
        setIsLoading(true);
        setTimeout(() => setIsLoading(false), 100); // Simulate async
      }, debounceMs),
    [debounceMs]
  );

  // Paginate results
  const paginatedResults = useMemo(() => {
    return paginateResults(searchResults, page * 20, 20);
  }, [searchResults, page]);

  // Update debounced search when query changes
  useEffect(() => {
    if (query) {
      debouncedSearch(query);
    }
  }, [query, debouncedSearch]);

  const handleSearch = useCallback((newQuery: string) => {
    setQuery(newQuery);
    setPage(0);
  }, []);

  const handleFilterChange = useCallback((newFilters: SearchFilters) => {
    setFilters(newFilters);
    setPage(0);
  }, []);

  const handleSortChange = useCallback((newSortBy: SearchSortOption) => {
    setSortBy(newSortBy);
  }, []);

  const handleCategoryChange = useCallback((newCategory: SearchCategory) => {
    setCategory(newCategory);
    setPage(0);
  }, []);

  const handlePageChange = useCallback((newPage: number) => {
    setPage(newPage);
  }, []);

  const clearFilters = useCallback(() => {
    setFilters({});
    setPage(0);
  }, []);

  const clearSearch = useCallback(() => {
    setQuery("");
    setFilters({});
    setPage(0);
  }, []);

  return {
    query,
    setQuery: handleSearch,
    filters,
    setFilters: handleFilterChange,
    sortBy,
    setSortBy: handleSortChange,
    category,
    setCategory: handleCategoryChange,
    page,
    setPage: handlePageChange,
    results: searchResults,
    paginatedResults: paginatedResults.results,
    totalResults: paginatedResults.total,
    hasMore: paginatedResults.hasMore,
    isLoading,
    clearFilters,
    clearSearch,
  };
}

/**
 * Hook for autocomplete suggestions
 */
export function useAutocomplete(
  query: string,
  allSuggestions: string[],
  maxSuggestions: number = 5
) {
  const [autocompleteSuggestions, setAutocompleteSuggestions] = useState<string[]>([]);
  const [showSuggestions, setShowSuggestions] = useState(false);

  useEffect(() => {
    if (query.length > 0) {
      const generated = generateAutocompleteSuggestions(query, allSuggestions, maxSuggestions);
      setAutocompleteSuggestions(generated);
      setShowSuggestions(generated.length > 0);
    } else {
      setAutocompleteSuggestions([]);
      setShowSuggestions(false);
    }
  }, [query, allSuggestions, maxSuggestions]);

  const selectSuggestion = useCallback((suggestion: string) => {
    setAutocompleteSuggestions([]);
    setShowSuggestions(false);
    return suggestion;
  }, []);

  const hideSuggestions = useCallback(() => {
    setShowSuggestions(false);
  }, []);

  return {
    suggestions: autocompleteSuggestions,
    showSuggestions,
    selectSuggestion,
    hideSuggestions,
  };
}

/**
 * Hook for saved searches
 */
export function useSavedSearches() {
  const [savedSearches, setSavedSearches] = useState<SavedSearch[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  // Load saved searches from localStorage
  useEffect(() => {
    const stored = localStorage.getItem("savedSearches");
    if (stored) {
      try {
        setSavedSearches(JSON.parse(stored));
      } catch (e) {
        console.error("Failed to load saved searches:", e);
      }
    }
  }, []);

  const saveSearch = useCallback((name: string, query: SearchQuery) => {
    const newSavedSearch: SavedSearch = {
      id: Date.now().toString(),
      name,
      query,
      createdAt: new Date(),
      useCount: 0,
    };

    setSavedSearches((prev: SavedSearch[]) => {
      const updated = [newSavedSearch, ...prev];
      localStorage.setItem("savedSearches", JSON.stringify(updated));
      return updated;
    });
  }, []);

  const loadSearch = useCallback((savedSearch: SavedSearch) => {
    // Update last used and use count
    setSavedSearches((prev: SavedSearch[]) => {
      const updated = prev.map((search: SavedSearch) =>
        search.id === savedSearch.id
          ? {
              ...search,
              lastUsedAt: new Date(),
              useCount: search.useCount + 1,
            }
          : search
      );
      localStorage.setItem("savedSearches", JSON.stringify(updated));
      return updated;
    });

    return savedSearch.query;
  }, []);

  const deleteSearch = useCallback((id: string) => {
    setSavedSearches((prev: SavedSearch[]) => {
      const updated = prev.filter((search: SavedSearch) => search.id !== id);
      localStorage.setItem("savedSearches", JSON.stringify(updated));
      return updated;
    });
  }, []);

  const clearAllSearches = useCallback(() => {
    setSavedSearches([]);
    localStorage.removeItem("savedSearches");
  }, []);

  return {
    savedSearches,
    saveSearch,
    loadSearch,
    deleteSearch,
    clearAllSearches,
  };
}

/**
 * Hook for search history
 */
export function useSearchHistory() {
  const [history, setHistory] = useState<SearchHistoryItem[]>([]);
  const MAX_HISTORY = 50;

  // Load history from localStorage
  useEffect(() => {
    const stored = localStorage.getItem("searchHistory");
    if (stored) {
      try {
        setHistory(JSON.parse(stored));
      } catch (e) {
        console.error("Failed to load search history:", e);
      }
    }
  }, []);

  const addToHistory = useCallback((query: string, category: SearchCategory, resultCount: number) => {
    setHistory((prev) => {
      // Remove duplicate if exists
      const filtered = prev.filter(
        (item) => item.query.toLowerCase() === query.toLowerCase() && item.category === category
      );

      // Add new item to front
      const newHistory = [
        {
          id: Date.now().toString(),
          query,
          category,
          timestamp: new Date(),
          resultCount,
        },
        ...filtered,
      ].slice(0, MAX_HISTORY);

      localStorage.setItem("searchHistory", JSON.stringify(newHistory));
      return newHistory;
    });
  }, []);

  const clearHistory = useCallback(() => {
    setHistory([]);
    localStorage.removeItem("searchHistory");
  }, []);

  const removeFromHistory = useCallback((id: string) => {
    setHistory((prev: SearchHistoryItem[]) => {
      const updated = prev.filter((item: SearchHistoryItem) => item.id !== id);
      localStorage.setItem("searchHistory", JSON.stringify(updated));
      return updated;
    });
  }, []);

  const getRecentSearches = useCallback((category?: SearchCategory, limit: number = 10) => {
    let filtered = history;
    if (category) {
      filtered = history.filter((item: SearchHistoryItem) => item.category === category);
    }
    return filtered.slice(0, limit);
  }, [history]);

  return {
    history,
    addToHistory,
    clearHistory,
    removeFromHistory,
    getRecentSearches,
  };
}
