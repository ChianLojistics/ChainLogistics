"use client";
/* eslint-disable react-hooks/set-state-in-effect */
import { useMemo, useState, useEffect } from "react";
import type { Product } from "@/lib/types/product";
import { normalizeUnixSeconds } from "@/lib/types/product";
import { fuzzyScore } from "@/lib/search/fuzzy";
import type { FilterState } from "./ProductFilters";
import { ProductCard } from "./ProductCard";
import { Skeleton } from "@/components/ui";

export type SortOption = "newest" | "oldest" | "name" | "relevance";

type ProductListProps = {
  products: Product[];
  filters: FilterState;
  isLoading?: boolean;
};

const ITEMS_PER_PAGE = 12;

export function ProductList({
  products,
  filters,
  isLoading = false,
}: ProductListProps) {
  const [sortBy, setSortBy] = useState<SortOption>("newest");
  const [currentPage, setCurrentPage] = useState(1);

  // Filter products
  const filteredProducts = useMemo(() => {
    const q = filters.search.trim();

    return products
      .map((product) => {
        if (!q) return { product, matchScore: 0 };

        const fields = [
          product.name,
          product.id,
          product.description ?? "",
          product.origin.location,
        ];

        let best = 0;
        let matchedAny = false;

        for (const field of fields) {
          const res = fuzzyScore(q, field);
          if (res.matched) {
            matchedAny = true;
            best = Math.max(best, res.score);
          }
        }

        return { product, matchScore: matchedAny ? best : -1 };
      })
      .filter(({ product, matchScore }) => {
        if (q && matchScore < 0) return false;

        // Owner filter
        if (filters.owner && product.owner !== filters.owner) {
          return false;
        }

        // Category filter
        if (filters.category && product.category !== filters.category) {
          return false;
        }

        // Status filter
        if (filters.status !== "all") {
          const isActive = filters.status === "active";
          const productIsActive =
            product.active ?? product.is_active ?? product.isActive ?? false;
          if (productIsActive !== isActive) return false;
        }

        // Date range filter
        const createdAt = normalizeUnixSeconds(product.created_at ?? product.createdAt ?? 0);
        if (filters.dateFrom) {
          const fromDate = new Date(filters.dateFrom).getTime() / 1000;
          if (createdAt < fromDate) return false;
        }
        if (filters.dateTo) {
          const toDate = new Date(filters.dateTo).getTime() / 1000;
          // Add one day to include the entire end date
          const toDateEnd = toDate + 24 * 60 * 60;
          if (createdAt > toDateEnd) return false;
        }

        return true;
      })
      .map((x) => x);
  }, [products, filters]);

  // Sort products
  const sortedProducts = useMemo(() => {
    const sorted = [...filteredProducts];
    switch (sortBy) {
      case "relevance":
        return sorted
          .sort((a, b) => b.matchScore - a.matchScore)
          .map((x) => x.product);
      case "newest":
        return sorted
          .map((x) => x.product)
          .sort(
            (a, b) =>
              normalizeUnixSeconds(b.created_at ?? b.createdAt ?? 0) -
              normalizeUnixSeconds(a.created_at ?? a.createdAt ?? 0)
          );
      case "oldest":
        return sorted
          .map((x) => x.product)
          .sort(
            (a, b) =>
              normalizeUnixSeconds(a.created_at ?? a.createdAt ?? 0) -
              normalizeUnixSeconds(b.created_at ?? b.createdAt ?? 0)
          );
      case "name":
        return sorted
          .map((x) => x.product)
          .sort((a, b) => a.name.localeCompare(b.name));
      default:
        return sorted.map((x) => x.product);
    }
  }, [filteredProducts, sortBy]);

  useEffect(() => {
    if (filters.search.trim() && sortBy === "newest") {
      setSortBy("relevance");
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [filters.search]);

  // Pagination
  const totalPages = Math.ceil(sortedProducts.length / ITEMS_PER_PAGE);
  const startIndex = (currentPage - 1) * ITEMS_PER_PAGE;
  const endIndex = startIndex + ITEMS_PER_PAGE;
  const paginatedProducts = sortedProducts.slice(startIndex, endIndex);

  // Reset to page 1 when filters or sort change
  useEffect(() => {
     
    setCurrentPage(1);
  }, [filters, sortBy]);

  if (isLoading) {
    return (
      <div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-8">
          {Array.from({ length: 6 }, (_, idx) => (
            <div
              key={idx}
              className="rounded-xl border bg-white shadow overflow-hidden"
              aria-hidden="true"
            >
              <div className="p-6 pb-4">
                <div className="flex items-start justify-between gap-4">
                  <div className="flex-1 space-y-2">
                    <Skeleton className="h-5 w-3/4" />
                    <Skeleton className="h-4 w-1/2" />
                  </div>
                  <Skeleton className="h-6 w-16 rounded-full" />
                </div>
              </div>
              <div className="px-6 pb-4 space-y-3">
                <Skeleton className="h-4 w-full" />
                <Skeleton className="h-4 w-5/6" />
                <Skeleton className="h-4 w-4/6" />
                <Skeleton className="h-4 w-3/6" />
              </div>
              <div className="px-6 pt-4 pb-6 border-t border-zinc-100 flex items-center justify-between gap-3">
                <Skeleton className="h-6 w-24 rounded-full" />
                <div className="flex items-center gap-2">
                  <Skeleton className="h-8 w-24" />
                  <Skeleton className="h-8 w-16" />
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    );
  }

  if (sortedProducts.length === 0) {
    return (
      <div className="bg-white rounded-lg border border-zinc-200 p-12 text-center">
        <svg
          className="w-16 h-16 mx-auto text-zinc-400 mb-4"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
          aria-hidden="true"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4"
          />
        </svg>
        <h3 className="text-lg font-semibold text-zinc-900 mb-2">
          No products found
        </h3>
        <p className="text-zinc-600">
          {filteredProducts.length === 0 && products.length > 0
            ? "Try adjusting your filters to see more results."
            : "No products have been registered yet."}
        </p>
      </div>
    );
  }

  return (
    <div>
      {/* Sort and results count */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between mb-6 gap-4">
        <p className="text-sm text-zinc-600" aria-live="polite" aria-atomic="true">
          Showing {startIndex + 1}-{Math.min(endIndex, sortedProducts.length)}{" "}
          of {sortedProducts.length} product{sortedProducts.length !== 1 ? "s" : ""}
        </p>
        <div className="flex items-center gap-2">
          <label
            htmlFor="sort"
            className="text-sm font-medium text-zinc-700 whitespace-nowrap"
          >
            Sort by:
          </label>
          <select
            id="sort"
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as SortOption)}
            className="px-4 py-2 border border-zinc-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none bg-white text-sm"
          >
            {filters.search.trim() ? <option value="relevance">Relevance</option> : null}
            <option value="newest">Newest first</option>
            <option value="oldest">Oldest first</option>
            <option value="name">Name (A-Z)</option>
          </select>
        </div>
      </div>

      {/* Product grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-8">
        {paginatedProducts.map((product) => (
          <ProductCard key={product.id} product={product} />
        ))}
      </div>

      {/* Pagination */}
      {totalPages > 1 && (
        <nav aria-label="Product list pagination" className="flex flex-col sm:flex-row items-center justify-center gap-2">
          <button
            onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
            disabled={currentPage === 1}
            aria-label="Go to previous page"
            className="px-4 py-2 border border-zinc-300 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed hover:bg-zinc-50 transition-colors text-sm"
          >
            Previous
          </button>
          <div className="flex items-center gap-1 flex-wrap justify-center" role="list">
            {Array.from({ length: totalPages }, (_, i) => i + 1).map((page) => {
              if (
                page === 1 ||
                page === totalPages ||
                (page >= currentPage - 1 && page <= currentPage + 1)
              ) {
                return (
                  <button
                    key={page}
                    onClick={() => setCurrentPage(page)}
                    aria-label={currentPage === page ? `Page ${page}, current page` : `Go to page ${page}`}
                    aria-current={currentPage === page ? "page" : undefined}
                    className={`px-3 py-2 border rounded-lg transition-colors text-sm ${currentPage === page
                      ? "bg-blue-500 text-white border-blue-500"
                      : "border-zinc-300 hover:bg-zinc-50"
                      }`}
                  >
                    {page}
                  </button>
                );
              } else if (
                page === currentPage - 2 ||
                page === currentPage + 2
              ) {
                return (
                  <span key={page} className="px-2 text-zinc-400 text-sm" aria-hidden="true">
                    ...
                  </span>
                );
              }
              return null;
            })}
          </div>
          <button
            onClick={() => setCurrentPage((p) => Math.min(totalPages, p + 1))}
            disabled={currentPage === totalPages}
            aria-label="Go to next page"
            className="px-4 py-2 border border-zinc-300 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed hover:bg-zinc-50 transition-colors text-sm"
          >
            Next
          </button>
        </nav>
      )}
    </div>
  );
}
