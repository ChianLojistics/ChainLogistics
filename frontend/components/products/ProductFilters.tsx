"use client";

import { useState, useEffect } from "react";
import type { Product } from "@/lib/types/product";

export type FilterState = {
  search: string;
  owner: string;
  category: string;
  status: "all" | "active" | "inactive";
  dateFrom: string;
  dateTo: string;
};

type ProductFiltersProps = {
  filters: FilterState;
  onFiltersChange: (filters: FilterState) => void;
  availableCategories: string[];
  availableOwners: string[];
};

export function ProductFilters({
  filters,
  onFiltersChange,
  availableCategories,
  availableOwners,
}: ProductFiltersProps) {
  const [localSearch, setLocalSearch] = useState(filters.search);

  // Debounce search input
  useEffect(() => {
    const timer = setTimeout(() => {
      onFiltersChange({ ...filters, search: localSearch });
    }, 300);

    return () => clearTimeout(timer);
  }, [localSearch]);

  const updateFilter = (key: keyof FilterState, value: string) => {
    onFiltersChange({ ...filters, [key]: value });
  };

  const clearFilters = () => {
    const cleared: FilterState = {
      search: "",
      owner: "",
      category: "",
      status: "all",
      dateFrom: "",
      dateTo: "",
    };
    setLocalSearch("");
    onFiltersChange(cleared);
  };

  const hasActiveFilters =
    filters.search ||
    filters.owner ||
    filters.category ||
    filters.status !== "all" ||
    filters.dateFrom ||
    filters.dateTo;

  return (
    <div className="bg-white rounded-lg border border-zinc-200 p-6 mb-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-lg font-semibold text-zinc-900">Filters</h2>
        {hasActiveFilters && (
          <button
            onClick={clearFilters}
            className="text-sm text-zinc-600 hover:text-zinc-900 underline"
          >
            Clear all
          </button>
        )}
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {/* Search */}
        <div className="lg:col-span-3">
          <label
            htmlFor="search"
            className="block text-sm font-medium text-zinc-700 mb-2"
          >
            Search
          </label>
          <input
            id="search"
            type="text"
            value={localSearch}
            onChange={(e) => setLocalSearch(e.target.value)}
            placeholder="Search by name, ID, or description..."
            className="w-full px-4 py-2 border border-zinc-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
          />
        </div>

        {/* Owner Filter */}
        <div>
          <label
            htmlFor="owner"
            className="block text-sm font-medium text-zinc-700 mb-2"
          >
            Owner
          </label>
          <select
            id="owner"
            value={filters.owner}
            onChange={(e) => updateFilter("owner", e.target.value)}
            className="w-full px-4 py-2 border border-zinc-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none bg-white"
          >
            <option value="">All owners</option>
            {availableOwners.map((owner) => (
              <option key={owner} value={owner}>
                {owner.slice(0, 8)}...{owner.slice(-6)}
              </option>
            ))}
          </select>
        </div>

        {/* Category Filter */}
        <div>
          <label
            htmlFor="category"
            className="block text-sm font-medium text-zinc-700 mb-2"
          >
            Category
          </label>
          <select
            id="category"
            value={filters.category}
            onChange={(e) => updateFilter("category", e.target.value)}
            className="w-full px-4 py-2 border border-zinc-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none bg-white"
          >
            <option value="">All categories</option>
            {availableCategories.map((category) => (
              <option key={category} value={category}>
                {category}
              </option>
            ))}
          </select>
        </div>

        {/* Status Filter */}
        <div>
          <label
            htmlFor="status"
            className="block text-sm font-medium text-zinc-700 mb-2"
          >
            Status
          </label>
          <select
            id="status"
            value={filters.status}
            onChange={(e) =>
              updateFilter("status", e.target.value as FilterState["status"])
            }
            className="w-full px-4 py-2 border border-zinc-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none bg-white"
          >
            <option value="all">All statuses</option>
            <option value="active">Active</option>
            <option value="inactive">Inactive</option>
          </select>
        </div>

        {/* Date From */}
        <div>
          <label
            htmlFor="dateFrom"
            className="block text-sm font-medium text-zinc-700 mb-2"
          >
            Date From
          </label>
          <input
            id="dateFrom"
            type="date"
            value={filters.dateFrom}
            onChange={(e) => updateFilter("dateFrom", e.target.value)}
            className="w-full px-4 py-2 border border-zinc-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
          />
        </div>

        {/* Date To */}
        <div>
          <label
            htmlFor="dateTo"
            className="block text-sm font-medium text-zinc-700 mb-2"
          >
            Date To
          </label>
          <input
            id="dateTo"
            type="date"
            value={filters.dateTo}
            onChange={(e) => updateFilter("dateTo", e.target.value)}
            className="w-full px-4 py-2 border border-zinc-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
          />
        </div>
      </div>
    </div>
  );
}
