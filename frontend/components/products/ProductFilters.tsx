"use client";

import { useMemo, useState, useEffect } from "react";
import { SearchInput, type SearchInputSuggestion } from "@/components/ui/SearchInput";
import {
  loadSavedSearches,
  persistSavedSearches,
  type SavedSearch,
} from "@/lib/search/savedSearches";

export type FilterState = {
  search: string;
  owner: string;
  category: string;
  status: "all" | "active" | "inactive";
  dateFrom: string;
  dateTo: string;
};

 type SavedFilters = FilterState;

type ProductFiltersProps = {
  filters: FilterState;
  onFiltersChange: (filters: FilterState) => void;
  availableCategories: string[];
  availableOwners: string[];
  suggestions?: readonly SearchInputSuggestion[];
};

export function ProductFilters({
  filters,
  onFiltersChange,
  availableCategories,
  availableOwners,
  suggestions = [],
}: ProductFiltersProps) {
  const [localSearch, setLocalSearch] = useState(filters.search);

  const storageKey = "chainlogistics.products.savedSearches";
  const [saved, setSaved] = useState<SavedSearch<SavedFilters>[]>([]);
  const [saveName, setSaveName] = useState("");

  // Debounce search input
  useEffect(() => {
    const timer = setTimeout(() => {
      onFiltersChange({ ...filters, search: localSearch });
    }, 300);

    return () => clearTimeout(timer);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [localSearch]);

  useEffect(() => {
    setSaved(loadSavedSearches<SavedFilters>(storageKey));
  }, []);

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

  const canSave = useMemo(() => {
    const name = saveName.trim();
    if (!name) return false;
    return true;
  }, [saveName]);

  const saveCurrent = () => {
    const name = saveName.trim();
    if (!name) return;

    const next: SavedSearch<SavedFilters> = {
      id: `${Date.now()}-${Math.random().toString(16).slice(2)}`,
      name,
      filters,
      createdAt: Date.now(),
    };

    const nextSaved = [next, ...saved].slice(0, 20);
    setSaved(nextSaved);
    persistSavedSearches(storageKey, nextSaved);
    setSaveName("");
  };

  const applySaved = (item: SavedSearch<SavedFilters>) => {
    setLocalSearch(item.filters.search);
    onFiltersChange(item.filters);
  };

  const deleteSaved = (id: string) => {
    const nextSaved = saved.filter((s) => s.id !== id);
    setSaved(nextSaved);
    persistSavedSearches(storageKey, nextSaved);
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
          <SearchInput
            id="search"
            value={localSearch}
            onValueChange={setLocalSearch}
            suggestions={suggestions}
            placeholder="Search by name, ID, or origin..."
            minQueryLength={1}
            maxSuggestions={8}
            emptyText="No suggestions"
          />
        </div>

        {/* Saved Searches */}
        <div className="lg:col-span-3">
          <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
            <div className="flex-1">
              <label className="block text-sm font-medium text-zinc-700 mb-2" htmlFor="saveSearchName">
                Saved searches
              </label>
              <div className="flex gap-2">
                <input
                  id="saveSearchName"
                  type="text"
                  value={saveName}
                  onChange={(e) => setSaveName(e.target.value)}
                  placeholder="Name this search..."
                  className="flex-1 px-4 py-2 border border-zinc-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                />
                <button
                  type="button"
                  onClick={saveCurrent}
                  disabled={!canSave}
                  className="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-semibold hover:bg-blue-700 disabled:opacity-50"
                >
                  Save
                </button>
              </div>
            </div>
          </div>

          {saved.length > 0 ? (
            <div className="mt-3 flex flex-wrap gap-2">
              {saved.map((s) => (
                <div key={s.id} className="inline-flex items-center overflow-hidden rounded-lg border border-zinc-200 bg-zinc-50">
                  <button
                    type="button"
                    onClick={() => applySaved(s)}
                    className="px-3 py-1.5 text-sm text-zinc-900 hover:bg-zinc-100"
                    title="Apply saved search"
                  >
                    {s.name}
                  </button>
                  <button
                    type="button"
                    onClick={() => deleteSaved(s.id)}
                    className="px-2 py-1.5 text-sm text-zinc-500 hover:bg-zinc-100 hover:text-zinc-900"
                    aria-label={`Delete saved search ${s.name}`}
                    title="Delete saved search"
                  >
                    ×
                  </button>
                </div>
              ))}
            </div>
          ) : null}
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
