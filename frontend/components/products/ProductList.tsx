"use client";

import { useMemo, useState, useEffect } from "react";
import type { Product } from "@/lib/types/product";
import type { FilterState } from "./ProductFilters";
import { ProductCard } from "./ProductCard";

export type SortOption = "newest" | "oldest" | "name";

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
    return products.filter((product) => {
      // Search filter
      if (filters.search) {
        const searchLower = filters.search.toLowerCase();
        const matchesSearch =
          product.name.toLowerCase().includes(searchLower) ||
          product.id.toLowerCase().includes(searchLower) ||
          product.description?.toLowerCase().includes(searchLower) ||
          product.origin.location.toLowerCase().includes(searchLower);
        if (!matchesSearch) return false;
      }

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
        if (product.active !== isActive) return false;
      }

      // Date range filter
      if (filters.dateFrom) {
        const fromDate = new Date(filters.dateFrom).getTime() / 1000;
        if (product.created_at < fromDate) return false;
      }
      if (filters.dateTo) {
        const toDate = new Date(filters.dateTo).getTime() / 1000;
        // Add one day to include the entire end date
        const toDateEnd = toDate + 24 * 60 * 60;
        if (product.created_at > toDateEnd) return false;
      }

      return true;
    });
  }, [products, filters]);

  // Sort products
  const sortedProducts = useMemo(() => {
    const sorted = [...filteredProducts];
    switch (sortBy) {
      case "newest":
        return sorted.sort((a, b) => b.created_at - a.created_at);
      case "oldest":
        return sorted.sort((a, b) => a.created_at - b.created_at);
      case "name":
        return sorted.sort((a, b) => a.name.localeCompare(b.name));
      default:
        return sorted;
    }
  }, [filteredProducts, sortBy]);

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
      <div className="flex items-center justify-center py-12">
        <div className="flex flex-col items-center gap-4">
          <div className="w-8 h-8 border-4 border-blue-500 border-t-transparent rounded-full animate-spin" />
          <p className="text-zinc-600">Loading products...</p>
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
        <p className="text-sm text-zinc-600">
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
        <div className="flex flex-col sm:flex-row items-center justify-center gap-2">
          <button
            onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
            disabled={currentPage === 1}
            className="px-4 py-2 border border-zinc-300 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed hover:bg-zinc-50 transition-colors text-sm"
          >
            Previous
          </button>
          <div className="flex items-center gap-1 flex-wrap justify-center">
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
                    className={`px-3 py-2 border rounded-lg transition-colors text-sm ${
                      currentPage === page
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
                  <span key={page} className="px-2 text-zinc-400 text-sm">
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
            className="px-4 py-2 border border-zinc-300 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed hover:bg-zinc-50 transition-colors text-sm"
          >
            Next
          </button>
        </div>
      )}
    </div>
  );
}
