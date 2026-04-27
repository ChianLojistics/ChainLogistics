"use client";

import { useState } from "react";
import { ChevronRight, ExternalLink, Star, X } from "lucide-react";
import type { SearchResult } from "@/lib/types/search";

interface SearchResultsProps {
  results: SearchResult[];
  totalResults: number;
  isLoading: boolean;
  onResultClick?: (result: SearchResult) => void;
  renderResult?: (result: SearchResult) => React.ReactNode;
  emptyMessage?: string;
  showRelevance?: boolean;
}

export function SearchResults({
  results,
  totalResults,
  isLoading,
  onResultClick,
  renderResult,
  emptyMessage = "No results found",
  showRelevance = false,
}: Readonly<SearchResultsProps>) {
  const [hoveredResult, setHoveredResult] = useState<string | null>(null);

  if (isLoading) {
    return (
      <div className="space-y-4">
        {[...Array(5)].map((_, i) => (
          <div key={i} className="animate-pulse">
            <div className="h-20 bg-gray-200 rounded-lg"></div>
          </div>
        ))}
      </div>
    );
  }

  if (results.length === 0) {
    return (
      <div className="text-center py-12">
        <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-gray-100 mb-4">
          <ChevronRight className="h-8 w-8 text-gray-400" />
        </div>
        <p className="text-gray-600">{emptyMessage}</p>
      </div>
    );
  }

  return (
    <div>
      {/* Results Summary */}
      <div className="mb-4 text-sm text-gray-600">
        {totalResults} result{totalResults !== 1 ? "s" : ""} found
      </div>

      {/* Results List */}
      <div className="space-y-3">
        {results.map((result) => (
          <div
            key={result.id}
            onMouseEnter={() => setHoveredResult(result.id)}
            onMouseLeave={() => setHoveredResult(null)}
            onClick={() => onResultClick?.(result)}
            className={`
              p-4 border border-gray-200 rounded-lg hover:border-blue-300 hover:shadow-md transition-all cursor-pointer
              ${hoveredResult === result.id ? "border-blue-300 shadow-md" : ""}
            `}
          >
            {renderResult ? (
              renderResult(result)
            ) : (
              <DefaultResultCard result={result} showRelevance={showRelevance} />
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

interface DefaultResultCardProps {
  result: SearchResult;
  showRelevance?: boolean;
}

function DefaultResultCard({ result, showRelevance }: DefaultResultCardProps) {
  return (
    <div className="flex items-start gap-3">
      {/* Relevance Score */}
      {showRelevance && (
        <div className="flex-shrink-0">
          <div
            className="w-8 h-8 rounded-full flex items-center justify-center text-xs font-medium"
            style={{
              backgroundColor: `rgba(59, 130, 246, ${result.relevanceScore})`,
              color: result.relevanceScore > 0.5 ? "white" : "#1e40af",
            }}
          >
            {Math.round(result.relevanceScore * 100)}%
          </div>
        </div>
      )}

      {/* Result Content */}
      <div className="flex-1 min-w-0">
        <div className="flex items-start justify-between gap-2">
          <h3 className="font-semibold text-gray-900 truncate">{result.title}</h3>
          <span className="px-2 py-1 text-xs font-medium bg-gray-100 text-gray-600 rounded">
            {result.type}
          </span>
        </div>

        {result.description && (
          <p className="mt-1 text-sm text-gray-600 line-clamp-2">{result.description}</p>
        )}

        {result.highlight && (
          <p
            className="mt-2 text-sm text-gray-700"
            dangerouslySetInnerHTML={{ __html: result.highlight }}
          />
        )}

        {result.metadata && (
          <div className="mt-2 flex flex-wrap gap-2">
            {result.metadata.category && (
              <span className="px-2 py-1 text-xs bg-blue-50 text-blue-700 rounded">
                {result.metadata.category}
              </span>
            )}
            {result.metadata.tags &&
              result.metadata.tags.slice(0, 3).map((tag) => (
                <span
                  key={tag}
                  className="px-2 py-1 text-xs bg-gray-50 text-gray-600 rounded"
                >
                  {tag}
                </span>
              ))}
          </div>
        )}
      </div>

      {/* Action Icon */}
      <div className="flex-shrink-0">
        <ExternalLink className="h-5 w-5 text-gray-400" />
      </div>
    </div>
  );
}

interface SearchResultsPaginationProps {
  currentPage: number;
  totalPages: number;
  onPageChange: (page: number) => void;
  totalResults: number;
  pageSize: number;
}

export function SearchResultsPagination({
  currentPage,
  totalPages,
  onPageChange,
  totalResults,
  pageSize,
}: Readonly<SearchResultsPaginationProps>) {
  const startResult = currentPage * pageSize + 1;
  const endResult = Math.min((currentPage + 1) * pageSize, totalResults);

  const pages = [];
  const maxVisiblePages = 5;
  let startPage = Math.max(0, currentPage - Math.floor(maxVisiblePages / 2));
  const endPage = Math.min(totalPages - 1, startPage + maxVisiblePages - 1);

  if (endPage - startPage < maxVisiblePages - 1) {
    startPage = Math.max(0, endPage - maxVisiblePages + 1);
  }

  for (let i = startPage; i <= endPage; i++) {
    pages.push(i);
  }

  return (
    <div className="flex items-center justify-between mt-6 pt-4 border-t border-gray-200">
      <div className="text-sm text-gray-600">
        Showing {startResult}-{endResult} of {totalResults} results
      </div>

      <div className="flex items-center gap-2">
        <button
          onClick={() => onPageChange(currentPage - 1)}
          disabled={currentPage === 0}
          className="px-3 py-1 border border-gray-300 rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Previous
        </button>

        {startPage > 0 && (
          <>
            <button
              onClick={() => onPageChange(0)}
              className="px-3 py-1 border border-gray-300 rounded hover:bg-gray-50"
            >
              1
            </button>
            {startPage > 1 && <span className="px-2">...</span>}
          </>
        )}

        {pages.map((page) => (
          <button
            key={page}
            onClick={() => onPageChange(page)}
            className={`px-3 py-1 border rounded ${
              page === currentPage
                ? "border-blue-500 bg-blue-50 text-blue-700"
                : "border-gray-300 hover:bg-gray-50"
            }`}
          >
            {page + 1}
          </button>
        ))}

        {endPage < totalPages - 1 && (
          <>
            {endPage < totalPages - 2 && <span className="px-2">...</span>}
            <button
              onClick={() => onPageChange(totalPages - 1)}
              className="px-3 py-1 border border-gray-300 rounded hover:bg-gray-50"
            >
              {totalPages}
            </button>
          </>
        )}

        <button
          onClick={() => onPageChange(currentPage + 1)}
          disabled={currentPage === totalPages - 1}
          className="px-3 py-1 border border-gray-300 rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Next
        </button>
      </div>
    </div>
  );
}

interface SavedSearchesPanelProps {
  savedSearches: Array<{ id: string; name: string; useCount: number }>;
  onLoadSearch: (id: string) => void;
  onDeleteSearch: (id: string) => void;
}

export function SavedSearchesPanel({
  savedSearches,
  onLoadSearch,
  onDeleteSearch,
}: Readonly<SavedSearchesPanelProps>) {
  const [isExpanded, setIsExpanded] = useState(false);

  if (savedSearches.length === 0) {
    return null;
  }

  return (
    <div className="mb-6 bg-white border border-gray-200 rounded-lg">
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full flex items-center justify-between p-4 hover:bg-gray-50"
      >
        <div className="flex items-center gap-2">
          <Star className="h-5 w-5 text-yellow-500" />
          <span className="font-semibold text-gray-900">Saved Searches</span>
          <span className="px-2 py-1 text-xs bg-gray-100 text-gray-600 rounded-full">
            {savedSearches.length}
          </span>
        </div>
        <span className="p-1">
          {isExpanded ? (
            <ChevronRight className="h-5 w-5 text-gray-600 rotate-90" />
          ) : (
            <ChevronRight className="h-5 w-5 text-gray-600" />
          )}
        </span>
      </button>

      {isExpanded && (
        <div className="px-4 pb-4 border-t border-gray-200">
          <div className="space-y-2">
            {savedSearches.map((search) => (
              <div
                key={search.id}
                className="flex items-center justify-between p-2 hover:bg-gray-50 rounded"
              >
                <div className="flex-1">
                  <button
                    onClick={() => onLoadSearch(search.id)}
                    className="text-left font-medium text-gray-900 hover:text-blue-600"
                  >
                    {search.name}
                  </button>
                  <span className="text-xs text-gray-500">
                    Used {search.useCount} time{search.useCount !== 1 ? "s" : ""}
                  </span>
                </div>
                <button
                  onClick={() => onDeleteSearch(search.id)}
                  className="p-1 hover:bg-gray-200 rounded"
                  title="Delete saved search"
                >
                  <X className="h-4 w-4 text-gray-400 hover:text-red-500" />
                </button>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
