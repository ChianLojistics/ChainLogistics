"use client";

import * as React from "react";
import { Search } from "lucide-react";

import { cn } from "@/lib/utils";

type BaseSuggestion = {
  id: string;
  label: string;
};

export type SearchInputSuggestion<TSuggestion extends BaseSuggestion = BaseSuggestion> =
  TSuggestion;

export interface SearchInputProps<
  TSuggestion extends BaseSuggestion = BaseSuggestion,
> extends Omit<React.InputHTMLAttributes<HTMLInputElement>, "value" | "defaultValue"> {
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;

  suggestions: readonly TSuggestion[];
  onSelectSuggestion?: (suggestion: TSuggestion) => void;

  getSuggestionLabel?: (suggestion: TSuggestion) => string;
  getSuggestionValue?: (suggestion: TSuggestion) => string;

  minQueryLength?: number;
  maxSuggestions?: number;

  emptyText?: string;

  containerClassName?: string;
  inputClassName?: string;
  dropdownClassName?: string;

  highlightMatches?: boolean;
}

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function HighlightedText({
  text,
  query,
}: {
  text: string;
  query: string;
}) {
  const q = query.trim();
  if (!q) return <>{text}</>;

  const re = new RegExp(`(${escapeRegExp(q)})`, "ig");
  const parts = text.split(re);

  return (
    <>
      {parts.map((part, idx) => {
        const isMatch = part.toLowerCase() === q.toLowerCase();
        return isMatch ? (
          <mark
             
            key={idx}
            className="rounded-sm bg-primary/10 px-0.5 text-foreground"
          >
            {part}
          </mark>
        ) : (
           
          <React.Fragment key={idx}>{part}</React.Fragment>
        );
      })}
    </>
  );
}

export function SearchInput<
  TSuggestion extends BaseSuggestion = BaseSuggestion,
>({
  value,
  defaultValue,
  onValueChange,
  suggestions,
  onSelectSuggestion,
  getSuggestionLabel,
  getSuggestionValue,
  minQueryLength = 1,
  maxSuggestions = 8,
  emptyText = "No results.",
  containerClassName,
  inputClassName,
  dropdownClassName,
  highlightMatches = true,
  placeholder = "Search…",
  disabled,
  id,
  ...inputProps
}: Readonly<SearchInputProps<TSuggestion>>) {
  const isControlled = value != null;
  const [uncontrolledValue, setUncontrolledValue] = React.useState(
    defaultValue ?? ""
  );

  const inputValue = isControlled ? (value as string) : uncontrolledValue;

  const setValue = React.useCallback(
    (next: string) => {
      if (!isControlled) setUncontrolledValue(next);
      onValueChange?.(next);
    },
    [isControlled, onValueChange]
  );

  const rootRef = React.useRef<HTMLDivElement | null>(null);
  const inputRef = React.useRef<HTMLInputElement | null>(null);

  const [open, setOpen] = React.useState(false);
  const [activeIndex, setActiveIndex] = React.useState<number>(-1);

  const labelOf = React.useCallback(
    (s: TSuggestion) => (getSuggestionLabel ? getSuggestionLabel(s) : s.label),
    [getSuggestionLabel]
  );

  const valueOf = React.useCallback(
    (s: TSuggestion) => (getSuggestionValue ? getSuggestionValue(s) : s.label),
    [getSuggestionValue]
  );

  const filtered = React.useMemo(() => {
    const q = inputValue.trim().toLowerCase();
    if (q.length < minQueryLength) return [] as TSuggestion[];

    const res = suggestions.filter((s) =>
      labelOf(s).toLowerCase().includes(q)
    );

    return res.slice(0, maxSuggestions);
  }, [inputValue, labelOf, maxSuggestions, minQueryLength, suggestions]);

  const listboxId = React.useId();

  const openIfPossible = React.useCallback(() => {
    if (disabled) return;
    const q = inputValue.trim();
    if (q.length < minQueryLength) return;
    setOpen(true);
  }, [disabled, inputValue, minQueryLength]);

  React.useEffect(() => {
    // Keep activeIndex in bounds when the filtered list changes.
    if (!open) {
      setActiveIndex(-1);
      return;
    }

    if (filtered.length === 0) {
      setActiveIndex(-1);
      return;
    }

    setActiveIndex((prev) => {
      if (prev < 0) return 0;
      return Math.min(prev, filtered.length - 1);
    });
  }, [filtered.length, open]);

  React.useEffect(() => {
    function onPointerDown(e: MouseEvent | TouchEvent) {
      const el = rootRef.current;
      if (!el) return;
      const target = e.target as Node | null;
      if (target && el.contains(target)) return;
      setOpen(false);
      setActiveIndex(-1);
    }

    document.addEventListener("mousedown", onPointerDown);
    document.addEventListener("touchstart", onPointerDown);

    return () => {
      document.removeEventListener("mousedown", onPointerDown);
      document.removeEventListener("touchstart", onPointerDown);
    };
  }, []);

  const selectSuggestion = React.useCallback(
    (s: TSuggestion) => {
      const nextValue = valueOf(s);
      setValue(nextValue);
      onSelectSuggestion?.(s);
      setOpen(false);
      setActiveIndex(-1);
      // keep focus in the input for quick follow-up actions
      requestAnimationFrame(() => inputRef.current?.focus());
    },
    [onSelectSuggestion, setValue, valueOf]
  );

  const activeDescendantId =
    open && activeIndex >= 0 && activeIndex < filtered.length
      ? `${listboxId}-option-${filtered[activeIndex]!.id}`
      : undefined;

  return (
    <div
      ref={rootRef}
      className={cn("relative w-full", containerClassName)}
    >
      <div className="relative">
        <div className="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3 text-muted-foreground">
          <Search className="h-4 w-4" />
        </div>

        <input
          {...inputProps}
          id={id}
          ref={(node) => {
            inputRef.current = node;
          }}
          value={inputValue}
          disabled={disabled}
          placeholder={placeholder}
          role="combobox"
          aria-autocomplete="list"
          aria-expanded={open}
          aria-controls={listboxId}
          aria-activedescendant={activeDescendantId}
          className={cn(
            "flex h-9 w-full rounded-md border border-input bg-transparent py-2 pl-9 pr-3 text-sm shadow-sm ring-offset-background",
            "placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring",
            "disabled:cursor-not-allowed disabled:opacity-50",
            inputClassName
          )}
          onFocus={(e) => {
            inputProps.onFocus?.(e);
            openIfPossible();
          }}
          onChange={(e) => {
            setValue(e.target.value);
            setOpen(true);
            inputProps.onChange?.(e);
          }}
          onKeyDown={(e) => {
            inputProps.onKeyDown?.(e);
            if (e.defaultPrevented) return;

            if (e.key === "ArrowDown") {
              e.preventDefault();
              if (!open) setOpen(true);
              if (filtered.length === 0) return;
              setActiveIndex((idx) => (idx + 1) % filtered.length);
              return;
            }

            if (e.key === "ArrowUp") {
              e.preventDefault();
              if (!open) setOpen(true);
              if (filtered.length === 0) return;
              setActiveIndex((idx) => {
                const next = idx <= 0 ? filtered.length - 1 : idx - 1;
                return next;
              });
              return;
            }

            if (e.key === "Enter") {
              if (!open) return;
              if (activeIndex < 0 || activeIndex >= filtered.length) return;
              e.preventDefault();
              selectSuggestion(filtered[activeIndex]!);
              return;
            }

            if (e.key === "Escape") {
              if (!open) return;
              e.preventDefault();
              setOpen(false);
              setActiveIndex(-1);
              return;
            }
          }}
        />
      </div>

      {open ? (
        <div
          className={cn(
            "absolute z-50 mt-1 w-full overflow-hidden rounded-md border bg-popover text-popover-foreground shadow-md",
            dropdownClassName
          )}
        >
          <ul
            id={listboxId}
            role="listbox"
            className="max-h-60 overflow-auto p-1"
          >
            {filtered.length === 0 ? (
              <li
                role="option"
                aria-disabled="true"
                aria-selected="false"
                className="px-2 py-2 text-sm text-muted-foreground"
              >
                {emptyText}
              </li>
            ) : (
              filtered.map((s, idx) => {
                const isActive = idx === activeIndex;
                const label = labelOf(s);

                return (
                  <li
                    id={`${listboxId}-option-${s.id}`}
                    key={s.id}
                    role="option"
                    aria-selected={isActive}
                    className={cn(
                      "flex cursor-pointer select-none items-center rounded-sm px-2 py-2 text-sm outline-none",
                      isActive
                        ? "bg-accent text-accent-foreground"
                        : "hover:bg-accent/50"
                    )}
                    // Use onMouseDown to avoid input blur before click fires.
                    onMouseDown={(e) => {
                      e.preventDefault();
                      selectSuggestion(s);
                    }}
                    onMouseEnter={() => setActiveIndex(idx)}
                  >
                    <span className="truncate">
                      {highlightMatches ? (
                        <HighlightedText text={label} query={inputValue} />
                      ) : (
                        label
                      )}
                    </span>
                  </li>
                );
              })
            )}
          </ul>
        </div>
      ) : null}
    </div>
  );
}
