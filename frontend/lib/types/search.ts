/**
 * Search and filtering types for the ChainLogistics application
 */

export type SearchCategory = "product" | "event" | "partner" | "document" | "analytics";

export interface SearchQuery {
  query: string;
  category: SearchCategory;
  filters: SearchFilters;
  sortBy?: SearchSortOption;
  limit?: number;
  offset?: number;
}

export interface SearchFilters {
  categories?: string[];
  dateRange?: {
    start: Date;
    end: Date;
  };
  locations?: string[];
  eventTypes?: string[];
  status?: string[];
  tags?: string[];
  customFilters?: Record<string, string | number | boolean>;
}

export type SearchSortOption =
  | "relevance"
  | "date_desc"
  | "date_asc"
  | "name_asc"
  | "name_desc"
  | "created_desc"
  | "created_asc";

export interface SearchResult<T = unknown> {
  id: string;
  type: SearchCategory;
  title: string;
  description?: string;
  relevanceScore: number;
  highlight?: string;
  data: T;
  metadata?: SearchMetadata;
}

export interface SearchMetadata {
  createdAt: Date;
  updatedAt: Date;
  category?: string;
  tags?: string[];
  [key: string]: string | number | Date | string[] | undefined;
}

export interface SearchSuggestions {
  query: string;
  suggestions: string[];
  categorySuggestions: SearchCategory[];
  recentSearches: string[];
}

export interface SavedSearch {
  id: string;
  name: string;
  query: SearchQuery;
  createdAt: Date;
  lastUsedAt?: Date;
  useCount: number;
}

export interface SearchHistoryItem {
  id: string;
  query: string;
  category: SearchCategory;
  timestamp: Date;
  resultCount: number;
}

export interface SearchAnalytics {
  searchId: string;
  query: string;
  category: SearchCategory;
  timestamp: Date;
  resultCount: number;
  clickedResult?: string;
  timeToFirstClick?: number;
  filtersApplied: boolean;
}

// Product-specific search types
export interface ProductSearchFilters extends SearchFilters {
  categories?: string[];
  origins?: string[];
  owners?: string[];
  active?: boolean;
  tags?: string[];
}

// Event-specific search types
export interface EventSearchFilters extends SearchFilters {
  eventTypes?: string[];
  actors?: string[];
  locations?: string[];
  dateRange?: {
    start: Date;
    end: Date;
  };
}

// Partner-specific search types
export interface PartnerSearchFilters extends SearchFilters {
  partnerTypes?: string[];
  locations?: string[];
  status?: string[];
  ratings?: number[];
}

// Document-specific search types
export interface DocumentSearchFilters extends SearchFilters {
  documentTypes?: string[];
  uploadDateRange?: {
    start: Date;
    end: Date;
  };
  fileTypes?: string[];
}

// Analytics-specific search types
export interface AnalyticsSearchFilters extends SearchFilters {
  reportTypes?: string[];
  dateRange?: {
    start: Date;
    end: Date;
  };
  metrics?: string[];
}
