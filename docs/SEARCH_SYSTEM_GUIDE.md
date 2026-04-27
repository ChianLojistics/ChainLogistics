# Search System Guide

This guide provides comprehensive documentation for the ChainLogistics frontend search and filtering system.

## Overview

The search system provides advanced search capabilities with fuzzy matching, faceted filtering, autocomplete suggestions, saved searches, search history, and analytics tracking. It's designed to handle large datasets efficiently while providing an excellent user experience.

## Architecture

### Core Components

1. **Types & Interfaces** (`lib/types/search.ts`)
   - Defines all search-related TypeScript types
   - Search queries, filters, results, and analytics types
   - Category-specific filter types

2. **Search Utilities** (`lib/utils/search.ts`)
   - Fuzzy search with Levenshtein distance
   - Relevance scoring algorithm
   - Filter application logic
   - Result sorting and pagination
   - Autocomplete suggestions
   - Text highlighting

3. **React Hooks** (`lib/hooks/useSearch.ts`)
   - `useSearch`: Main search hook with debouncing
   - `useAutocomplete`: Autocomplete suggestions
   - `useSavedSearches`: Saved search management
   - `useSearchHistory`: Search history tracking

4. **UI Components**
   - `SearchBar`: Search input with autocomplete and history
   - `FacetedFilter`: Multi-category filtering with collapsible groups
   - `SearchResults`: Result display with pagination
   - `SavedSearchesPanel`: Saved search management

5. **Analytics** (`lib/analytics/searchAnalytics.ts`)
   - Search session tracking
   - Click tracking and time-to-click metrics
   - Statistics and reporting

## Usage Guide

### Basic Search

```typescript
import { useSearch } from "@/lib/hooks/useSearch";

function ProductSearch() {
  const products = [
    { id: "1", title: "Organic Coffee", description: "Premium coffee beans", category: "Beverages" },
    // ... more products
  ];

  const {
    query,
    setQuery,
    results,
    paginatedResults,
    totalResults,
    isLoading,
    clearSearch,
  } = useSearch(products, {
    threshold: 0.3,
    debounceMs: 300,
    itemAccessor: (item) => ({
      category: item.category,
      tags: [],
    }),
  });

  return (
    <div>
      <input
        type="text"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder="Search products..."
      />
      <div>
        {paginatedResults.map((result) => (
          <div key={result.id}>{result.title}</div>
        ))}
      </div>
    </div>
  );
}
```

### Advanced Search with Filters

```typescript
import { useSearch } from "@/lib/hooks/useSearch";
import { SearchBar } from "@/components/search/SearchBar";
import { FacetedFilter } from "@/components/search/FacetedFilter";
import type { SearchFilters, SearchSortOption } from "@/lib/types/search";

function AdvancedSearch() {
  const [filters, setFilters] = useState<SearchFilters>({});
  const [sortBy, setSortBy] = useState<SearchSortOption>("relevance");
  const [showFilters, setShowFilters] = useState(false);

  const { query, setQuery, results, paginatedResults } = useSearch(products);

  const filterGroups = [
    {
      id: "categories",
      label: "Categories",
      type: "checkbox" as const,
      options: [
        { label: "Beverages", value: "beverages", count: 45 },
        { label: "Food", value: "food", count: 32 },
      ],
      collapsible: true,
    },
    {
      id: "dateRange",
      label: "Date Range",
      type: "date-range" as const,
      options: [],
      collapsible: true,
    },
  ];

  return (
    <div className="flex gap-6">
      <div className="w-1/3">
        <SearchBar
          query={query}
          onQueryChange={setQuery}
          category="product"
          onCategoryChange={() => {}}
          onFilterClick={() => setShowFilters(!showFilters)}
          onSearch={() => {}}
        />
        {showFilters && (
          <FacetedFilter
            filters={filters}
            onFiltersChange={setFilters}
            sortBy={sortBy}
            onSortChange={setSortBy}
            filterGroups={filterGroups}
            isOpen={showFilters}
            onToggle={() => setShowFilters(!showFilters)}
          />
        )}
      </div>
      <div className="w-2/3">
        <SearchResults
          results={paginatedResults}
          totalResults={results.length}
          isLoading={false}
          onResultClick={(result) => console.log(result)}
        />
      </div>
    </div>
  );
}
```

### Saved Searches

```typescript
import { useSavedSearches } from "@/lib/hooks/useSearch";
import type { SearchQuery } from "@/lib/types/search";

function SavedSearches() {
  const { savedSearches, saveSearch, loadSearch, deleteSearch } = useSavedSearches();

  const handleSaveSearch = (name: string) => {
    const query: SearchQuery = {
      query: "coffee",
      category: "product",
      filters: { categories: ["beverages"] },
      sortBy: "relevance",
    };
    saveSearch(name, query);
  };

  const handleLoadSearch = (id: string) => {
    const search = savedSearches.find((s) => s.id === id);
    if (search) {
      const loadedQuery = loadSearch(search);
      // Apply loaded query to search state
    }
  };

  return (
    <div>
      <button onClick={() => handleSaveSearch("My Coffee Search")}>
        Save Current Search
      </button>
      <div>
        {savedSearches.map((search) => (
          <div key={search.id}>
            <span>{search.name}</span>
            <button onClick={() => handleLoadSearch(search.id)}>Load</button>
            <button onClick={() => deleteSearch(search.id)}>Delete</button>
          </div>
        ))}
      </div>
    </div>
  );
}
```

### Search Analytics

```typescript
import { useSearchAnalytics } from "@/lib/analytics/searchAnalytics";

function SearchWithAnalytics() {
  const { trackSearch, trackResults, trackClick, getStatistics } = useSearchAnalytics();

  const handleSearch = (query: string) => {
    const searchId = trackSearch(query, "product", false);
    // Perform search...
    const resultCount = results.length;
    trackResults(searchId, resultCount);
  };

  const handleResultClick = (searchId: string, resultId: string) => {
    trackClick(searchId, resultId);
  };

  const stats = getStatistics();
  console.log("Search statistics:", stats);

  return <div>...</div>;
}
```

## API Reference

### Types

#### SearchQuery
```typescript
interface SearchQuery {
  query: string;
  category: SearchCategory;
  filters: SearchFilters;
  sortBy?: SearchSortOption;
  limit?: number;
  offset?: number;
}
```

#### SearchFilters
```typescript
interface SearchFilters {
  categories?: string[];
  dateRange?: { start: Date; end: Date };
  locations?: string[];
  eventTypes?: string[];
  status?: string[];
  tags?: string[];
  customFilters?: Record<string, string | number | boolean>;
}
```

#### SearchResult
```typescript
interface SearchResult<T = any> {
  id: string;
  type: SearchCategory;
  title: string;
  description?: string;
  relevanceScore: number;
  highlight?: string;
  data: T;
  metadata?: SearchMetadata;
}
```

### Hooks

#### useSearch
```typescript
function useSearch<T>(
  items: T[],
  options: {
    threshold?: number;
    debounceMs?: number;
    itemAccessor?: (item: T) => any;
    dateAccessor?: (item: T) => Date | undefined;
    nameAccessor?: (item: T) => string;
  }
): {
  query: string;
  setQuery: (query: string) => void;
  filters: SearchFilters;
  setFilters: (filters: SearchFilters) => void;
  sortBy: SearchSortOption;
  setSortBy: (sortBy: SearchSortOption) => void;
  category: SearchCategory;
  setCategory: (category: SearchCategory) => void;
  page: number;
  setPage: (page: number) => void;
  results: SearchResult<T>[];
  paginatedResults: SearchResult<T>[];
  totalResults: number;
  hasMore: boolean;
  isLoading: boolean;
  clearFilters: () => void;
  clearSearch: () => void;
}
```

#### useAutocomplete
```typescript
function useAutocomplete(
  query: string,
  suggestions: string[],
  maxSuggestions?: number
): {
  suggestions: string[];
  showSuggestions: boolean;
  selectSuggestion: (suggestion: string) => string;
  hideSuggestions: () => void;
}
```

#### useSavedSearches
```typescript
function useSavedSearches(): {
  savedSearches: SavedSearch[];
  saveSearch: (name: string, query: SearchQuery) => void;
  loadSearch: (savedSearch: SavedSearch) => SearchQuery;
  deleteSearch: (id: string) => void;
  clearAllSearches: () => void;
}
```

#### useSearchHistory
```typescript
function useSearchHistory(): {
  history: SearchHistoryItem[];
  addToHistory: (query: string, category: SearchCategory, resultCount: number) => void;
  clearHistory: () => void;
  removeFromHistory: (id: string) => void;
  getRecentSearches: (category?: SearchCategory, limit?: number) => SearchHistoryItem[];
}
```

### Utilities

#### fuzzySearch
Performs fuzzy search on an array of items using Levenshtein distance.

```typescript
function fuzzySearch<T>(
  items: T[],
  query: string,
  threshold?: number
): Array<{ item: T; score: number }>
```

#### applyFilters
Filters items based on search filters.

```typescript
function applyFilters<T>(
  items: T[],
  filters: SearchFilters,
  itemAccessor: (item: T) => any
): T[]
```

#### sortResults
Sorts search results based on sort option.

```typescript
function sortResults<T>(
  results: Array<{ item: T; score: number }>,
  sortBy: SearchSortOption,
  dateAccessor?: (item: T) => Date | undefined,
  nameAccessor?: (item: T) => string
): Array<{ item: T; score: number }>
```

#### calculateRelevanceScore
Calculates relevance score for a search result.

```typescript
function calculateRelevanceScore(
  query: string,
  result: { title: string; description?: string; tags?: string[] }
): number
```

## Performance Optimization

### Debouncing
Search input is debounced by default (300ms) to reduce unnecessary searches while typing.

### Pagination
Results are paginated with a default page size of 20 items. Use the `page` and `setPage` from `useSearch` to navigate pages.

### Threshold Tuning
Adjust the `threshold` parameter in `useSearch` to control fuzzy matching sensitivity:
- Lower threshold (0.1-0.2): More permissive, more results
- Higher threshold (0.4-0.5): More strict, fewer results

### Memoization
The search hooks use React's `useMemo` to optimize performance. Search results are only recalculated when dependencies change.

## Best Practices

1. **Use meaningful item accessors**: Provide proper `itemAccessor` to enable filtering on custom fields
2. **Implement proper date accessors**: For date-based sorting, provide `dateAccessor`
3. **Set appropriate thresholds**: Balance between precision and recall based on your use case
4. **Track analytics**: Use search analytics to understand user behavior and improve search relevance
5. **Save popular searches**: Encourage users to save frequently used searches for quick access
6. **Clear filters properly**: Provide clear UI for users to reset filters
7. **Handle empty states**: Show helpful messages when no results are found
8. **Optimize for mobile**: Ensure search components work well on mobile devices

## Search Categories

The system supports the following search categories:

- **product**: Search products by name, category, origin, attributes
- **event**: Search supply chain events by type, location, date
- **partner**: Find supply chain partners and suppliers
- **document**: Search uploaded documents and certificates
- **analytics**: Find specific analytics and reports

## Integration Examples

### Product Search Page
```typescript
// app/products/search/page.tsx
import { SearchBar } from "@/components/search/SearchBar";
import { FacetedFilter } from "@/components/search/FacetedFilter";
import { SearchResults } from "@/components/search/SearchResults";

export default function ProductSearchPage() {
  // Implementation
}
```

### Global Search Component
```typescript
// components/GlobalSearch.tsx
import { SearchBar } from "@/components/search/SearchBar";

export function GlobalSearch() {
  // Implementation for site-wide search
}
```

## Troubleshooting

### Search Returns No Results
- Check if the query is too specific (try lowering the threshold)
- Verify that items have proper title/description fields
- Ensure filters aren't too restrictive

### Slow Search Performance
- Reduce the dataset size or implement server-side search
- Increase debounce time to reduce search frequency
- Consider implementing search indexing

### Autocomplete Not Working
- Verify suggestions array is populated
- Check that query length is sufficient for matching
- Ensure threshold isn't too high

## Future Enhancements

- Server-side search with Elasticsearch integration
- Voice search support
- Image-based search
- Advanced query syntax (AND, OR, NOT operators)
- Search result clustering
- Personalized search ranking
- Real-time search suggestions
- Search result preview
