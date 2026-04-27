"use client";

import { useState, useRef, useEffect } from "react";
import { Search, X, Clock, Filter, Save } from "lucide-react";
import { useAutocomplete, useSearchHistory } from "@/lib/hooks/useSearch";
import type { SearchCategory } from "@/lib/types/search";

interface SearchBarProps {
  query: string;
  onQueryChange: (query: string) => void;
  category: SearchCategory;
  onCategoryChange: (category: SearchCategory) => void;
  suggestions?: string[];
  onSearch?: () => void;
  onFilterClick?: () => void;
  onSaveSearch?: () => void;
  placeholder?: string;
}

const CATEGORIES: { value: SearchCategory; label: string }[] = [
  { value: "product", label: "Products" },
  { value: "event", label: "Events" },
  { value: "partner", label: "Partners" },
  { value: "document", label: "Documents" },
  { value: "analytics", label: "Analytics" },
];

export function SearchBar({
  query,
  onQueryChange,
  category,
  onCategoryChange,
  suggestions = [],
  onSearch,
  onFilterClick,
  onSaveSearch,
  placeholder = "Search...",
}: SearchBarProps) {
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [showHistory, setShowHistory] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const { suggestions: autocompleteSuggestions, showSuggestions: showAutocomplete } = useAutocomplete(
    query,
    suggestions,
    5
  );
  const { history, getRecentSearches, addToHistory } = useSearchHistory();
  const recentSearches = getRecentSearches(category, 5);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setShowSuggestions(false);
        setShowHistory(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const handleInputChange = (value: string) => {
    onQueryChange(value);
    setShowHistory(value.length === 0);
    setShowSuggestions(value.length > 0);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && query) {
      onSearch?.();
      addToHistory(query, category, 0);
      setShowSuggestions(false);
      setShowHistory(false);
    } else if (e.key === "Escape") {
      setShowSuggestions(false);
      setShowHistory(false);
    }
  };

  const handleSuggestionClick = (suggestion: string) => {
    onQueryChange(suggestion);
    onSearch?.();
    addToHistory(suggestion, category, 0);
    setShowSuggestions(false);
    setShowHistory(false);
  };

  const handleHistoryClick = (historyItem: typeof recentSearches[0]) => {
    onQueryChange(historyItem.query);
    onSearch?.();
    setShowHistory(false);
  };

  const handleClear = () => {
    onQueryChange("");
    inputRef.current?.focus();
    setShowHistory(true);
  };

  return (
    <div className="relative w-full">
      {/* Search Input Container */}
      <div className="flex items-center gap-2">
        {/* Category Selector */}
        <select
          value={category}
          onChange={(e) => onCategoryChange(e.target.value as SearchCategory)}
          className="px-3 py-2 text-sm border border-gray-300 rounded-lg bg-white focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          {CATEGORIES.map((cat) => (
            <option key={cat.value} value={cat.value}>
              {cat.label}
            </option>
          ))}
        </select>

        {/* Search Input */}
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-gray-400" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => handleInputChange(e.target.value)}
            onKeyDown={handleKeyDown}
            onFocus={() => {
              if (query.length === 0) setShowHistory(true);
              else setShowSuggestions(true);
            }}
            placeholder={placeholder}
            className="w-full pl-10 pr-10 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          {query && (
            <button
              onClick={handleClear}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600"
            >
              <X className="h-5 w-5" />
            </button>
          )}
        </div>

        {/* Action Buttons */}
        <button
          onClick={onFilterClick}
          className="p-2 border border-gray-300 rounded-lg hover:bg-gray-50"
          title="Filters"
        >
          <Filter className="h-5 w-5 text-gray-600" />
        </button>
        {onSaveSearch && (
          <button
            onClick={onSaveSearch}
            className="p-2 border border-gray-300 rounded-lg hover:bg-gray-50"
            title="Save Search"
          >
            <Save className="h-5 w-5 text-gray-600" />
          </button>
        )}
        <button
          onClick={() => {
            if (query) {
              onSearch?.();
              addToHistory(query, category, 0);
            }
          }}
          disabled={!query}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Search
        </button>
      </div>

      {/* Autocomplete Suggestions */}
      {showSuggestions && showAutocomplete && autocompleteSuggestions.length > 0 && (
        <div
          ref={dropdownRef}
          className="absolute z-50 w-full mt-2 bg-white border border-gray-300 rounded-lg shadow-lg max-h-60 overflow-y-auto"
        >
          {autocompleteSuggestions.map((suggestion, index) => (
            <button
              key={index}
              onClick={() => handleSuggestionClick(suggestion)}
              className="w-full px-4 py-2 text-left hover:bg-gray-50 focus:outline-none focus:bg-gray-50"
            >
              <Search className="inline h-4 w-4 mr-2 text-gray-400" />
              {suggestion}
            </button>
          ))}
        </div>
      )}

      {/* Search History */}
      {showHistory && recentSearches.length > 0 && (
        <div
          ref={dropdownRef}
          className="absolute z-50 w-full mt-2 bg-white border border-gray-300 rounded-lg shadow-lg"
        >
          <div className="px-4 py-2 border-b border-gray-200">
            <span className="text-sm font-medium text-gray-700">Recent Searches</span>
          </div>
          {recentSearches.map((item) => (
            <button
              key={item.id}
              onClick={() => handleHistoryClick(item)}
              className="w-full px-4 py-2 text-left hover:bg-gray-50 focus:outline-none focus:bg-gray-50 flex items-center gap-2"
            >
              <Clock className="h-4 w-4 text-gray-400" />
              <span className="text-sm">{item.query}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
