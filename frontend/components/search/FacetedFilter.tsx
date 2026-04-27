"use client";

import { useState } from "react";
import { X, ChevronDown, ChevronUp, Filter } from "lucide-react";
import type { SearchFilters, SearchSortOption } from "@/lib/types/search";

interface FilterOption {
  label: string;
  value: string;
  count?: number;
}

interface FilterGroup {
  id: string;
  label: string;
  options: FilterOption[];
  type: "checkbox" | "radio" | "date-range";
  collapsible?: boolean;
}

interface FacetedFilterProps {
  filters: SearchFilters;
  onFiltersChange: (filters: SearchFilters) => void;
  sortBy: SearchSortOption;
  onSortChange: (sortBy: SearchSortOption) => void;
  filterGroups: FilterGroup[];
  isOpen?: boolean;
  onToggle?: () => void;
}

const SORT_OPTIONS: { value: SearchSortOption; label: string }[] = [
  { value: "relevance", label: "Relevance" },
  { value: "date_desc", label: "Newest First" },
  { value: "date_asc", label: "Oldest First" },
  { value: "name_asc", label: "Name A-Z" },
  { value: "name_desc", label: "Name Z-A" },
];

export function FacetedFilter({
  filters,
  onFiltersChange,
  sortBy,
  onSortChange,
  filterGroups,
  isOpen = false,
  onToggle,
}: FacetedFilterProps) {
  const [expandedGroups, setExpandedGroups] = useState<Set<string>>(
    new Set(filterGroups.map((g) => g.id))
  );

  const toggleGroup = (groupId: string) => {
    setExpandedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(groupId)) {
        next.delete(groupId);
      } else {
        next.add(groupId);
      }
      return next;
    });
  };

  const handleCheckboxChange = (groupId: string, value: string, checked: boolean) => {
    const newFilters = { ...filters };
    const group = filterGroups.find((g) => g.id === groupId);

    if (group) {
      const filterKey = groupId as keyof SearchFilters;
      const currentValues = Array.isArray(newFilters[filterKey]) ? (newFilters[filterKey] as string[]) : [];

      if (checked) {
        (newFilters[filterKey] as string[]) = [...currentValues, value];
      } else {
        (newFilters[filterKey] as string[]) = currentValues.filter((v) => v !== value);
      }

      onFiltersChange(newFilters);
    }
  };

  const handleDateRangeChange = (start: Date | null, end: Date | null) => {
    const newFilters = { ...filters };

    if (start && end) {
      newFilters.dateRange = { start, end };
    } else {
      delete newFilters.dateRange;
    }

    onFiltersChange(newFilters);
  };

  const clearAllFilters = () => {
    onFiltersChange({});
  };

  const activeFilterCount = Object.values(filters).filter(
    (value) =>
      value !== undefined &&
      value !== null &&
      (Array.isArray(value) ? value.length > 0 : true)
  ).length;

  return (
    <div className="bg-white border border-gray-200 rounded-lg">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-200">
        <div className="flex items-center gap-2">
          <Filter className="h-5 w-5 text-gray-600" />
          <span className="font-semibold text-gray-900">Filters</span>
          {activeFilterCount > 0 && (
            <span className="px-2 py-1 text-xs font-medium bg-blue-100 text-blue-800 rounded-full">
              {activeFilterCount}
            </span>
          )}
        </div>
        <div className="flex items-center gap-2">
          {activeFilterCount > 0 && (
            <button
              onClick={clearAllFilters}
              className="text-sm text-blue-600 hover:text-blue-800"
            >
              Clear All
            </button>
          )}
          {onToggle && (
            <button onClick={onToggle} className="p-1 hover:bg-gray-100 rounded">
              {isOpen ? (
                <ChevronUp className="h-5 w-5 text-gray-600" />
              ) : (
                <ChevronDown className="h-5 w-5 text-gray-600" />
              )}
            </button>
          )}
        </div>
      </div>

      {/* Sort Options */}
      <div className="p-4 border-b border-gray-200">
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Sort By
        </label>
        <select
          value={sortBy}
          onChange={(e) => onSortChange(e.target.value as SearchSortOption)}
          className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          {SORT_OPTIONS.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      </div>

      {/* Filter Groups */}
      {isOpen &&
        filterGroups.map((group) => (
          <div key={group.id} className="border-b border-gray-200 last:border-b-0">
            <button
              onClick={() => group.collapsible && toggleGroup(group.id)}
              className="w-full flex items-center justify-between p-4 hover:bg-gray-50"
            >
              <span className="font-medium text-gray-900">{group.label}</span>
              {group.collapsible && (
                <span className="p-1">
                  {expandedGroups.has(group.id) ? (
                    <ChevronUp className="h-4 w-4 text-gray-600" />
                  ) : (
                    <ChevronDown className="h-4 w-4 text-gray-600" />
                  )}
                </span>
              )}
            </button>

            {(!group.collapsible || expandedGroups.has(group.id)) && (
              <div className="px-4 pb-4">
                {group.type === "checkbox" && (
                  <div className="space-y-2">
                    {group.options.map((option) => {
                      const isChecked = (
                        filters[group.id as keyof SearchFilters] as string[]
                      )?.includes(option.value);

                      return (
                        <label
                          key={option.value}
                          className="flex items-center gap-2 cursor-pointer"
                        >
                          <input
                            type="checkbox"
                            checked={isChecked}
                            onChange={(e) =>
                              handleCheckboxChange(group.id, option.value, e.target.checked)
                            }
                            className="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                          />
                          <span className="text-sm text-gray-700">{option.label}</span>
                          {option.count !== undefined && (
                            <span className="text-xs text-gray-500">({option.count})</span>
                          )}
                        </label>
                      );
                    })}
                  </div>
                )}

                {group.type === "radio" && (
                  <div className="space-y-2">
                    {group.options.map((option) => {
                      const filterValue = filters[group.id as keyof SearchFilters];
                      const isChecked = typeof filterValue === "string" && filterValue === option.value;

                      return (
                        <label
                          key={option.value}
                          className="flex items-center gap-2 cursor-pointer"
                        >
                          <input
                            type="radio"
                            name={group.id}
                            checked={isChecked}
                            onChange={() => {
                              const newFilters = { ...filters };
                              (newFilters as Record<string, unknown>)[group.id] = option.value;
                              onFiltersChange(newFilters);
                            }}
                            className="w-4 h-4 text-blue-600 border-gray-300 focus:ring-blue-500"
                          />
                          <span className="text-sm text-gray-700">{option.label}</span>
                        </label>
                      );
                    })}
                  </div>
                )}

                {group.type === "date-range" && (
                  <div className="space-y-2">
                    <div>
                      <label className="block text-sm text-gray-600 mb-1">From</label>
                      <input
                        type="date"
                        value={filters.dateRange?.start.toISOString().split("T")[0] || ""}
                        onChange={(e) => {
                          const start = e.target.value ? new Date(e.target.value) : null;
                          handleDateRangeChange(start, filters.dateRange?.end || null);
                        }}
                        className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                      />
                    </div>
                    <div>
                      <label className="block text-sm text-gray-600 mb-1">To</label>
                      <input
                        type="date"
                        value={filters.dateRange?.end.toISOString().split("T")[0] || ""}
                        onChange={(e) => {
                          const end = e.target.value ? new Date(e.target.value) : null;
                          handleDateRangeChange(filters.dateRange?.start || null, end);
                        }}
                        className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                      />
                    </div>
                  </div>
                )}
              </div>
            )}
          </div>
        ))}

      {/* Active Filters Display */}
      {activeFilterCount > 0 && (
        <div className="p-4 border-t border-gray-200">
          <div className="flex flex-wrap gap-2">
            {Object.entries(filters).map(([key, value]) => {
              if (!value || (Array.isArray(value) && value.length === 0)) return null;

              if (Array.isArray(value)) {
                return value.map((v) => (
                  <span
                    key={`${key}-${v}`}
                    className="inline-flex items-center gap-1 px-3 py-1 bg-gray-100 text-gray-700 rounded-full text-sm"
                  >
                    {v}
                    <button
                      onClick={() => handleCheckboxChange(key, v, false)}
                      className="hover:text-gray-900"
                    >
                      <X className="h-3 w-3" />
                    </button>
                  </span>
                ));
              }

              return null;
            })}
          </div>
        </div>
      )}
    </div>
  );
}
