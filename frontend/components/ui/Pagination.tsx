"use client";

import * as React from "react";

import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

type PaginationItem = number | "ellipsis";

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}

function range(start: number, end: number) {
  const result: number[] = [];
  for (let i = start; i <= end; i += 1) result.push(i);
  return result;
}

function getPaginationItems(params: {
  page: number;
  totalPages: number;
  siblingCount: number;
  boundaryCount: number;
}): PaginationItem[] {
  const { page, totalPages, siblingCount, boundaryCount } = params;

  if (totalPages <= 0) return [];

  const safePage = clamp(page, 1, totalPages);

  const totalNumbers = boundaryCount * 2 + siblingCount * 2 + 1;
  const totalBlocks = totalNumbers + 2; // 2 for ellipsis

  // If the number of pages is small, show all pages.
  if (totalPages <= totalBlocks) {
    return range(1, totalPages);
  }

  const leftSiblingIndex = Math.max(safePage - siblingCount, boundaryCount + 2);
  const rightSiblingIndex = Math.min(
    safePage + siblingCount,
    totalPages - boundaryCount - 1
  );

  const showLeftEllipsis = leftSiblingIndex > boundaryCount + 2;
  const showRightEllipsis = rightSiblingIndex < totalPages - boundaryCount - 1;

  const firstPages = range(1, boundaryCount);
  const lastPages = range(totalPages - boundaryCount + 1, totalPages);

  if (!showLeftEllipsis && showRightEllipsis) {
    // No left ellipsis, but right ellipsis
    const leftItemCount = boundaryCount + 1 + 2 * siblingCount;
    const leftRange = range(1, leftItemCount);
    return [...leftRange, "ellipsis", ...lastPages];
  }

  if (showLeftEllipsis && !showRightEllipsis) {
    // Left ellipsis, but no right ellipsis
    const rightItemCount = boundaryCount + 1 + 2 * siblingCount;
    const rightRange = range(totalPages - rightItemCount + 1, totalPages);
    return [...firstPages, "ellipsis", ...rightRange];
  }

  // Both left and right ellipsis
  const middleRange = range(leftSiblingIndex, rightSiblingIndex);
  return [...firstPages, "ellipsis", ...middleRange, "ellipsis", ...lastPages];
}

export type PaginationProps = {
  page: number;
  totalPages?: number;
  totalItems?: number;
  pageSize?: number;
  pageSizeOptions?: number[];
  siblingCount?: number;
  boundaryCount?: number;
  onPageChange: (page: number) => void;
  onPageSizeChange?: (pageSize: number) => void;
  className?: string;
  disabled?: boolean;
  showPageSizeSelector?: boolean;
  pageSizeLabel?: string;
};

export function Pagination({
  page,
  totalPages: totalPagesProp,
  totalItems,
  pageSize,
  pageSizeOptions = [10, 20, 50, 100],
  siblingCount = 1,
  boundaryCount = 1,
  onPageChange,
  onPageSizeChange,
  className,
  disabled = false,
  showPageSizeSelector = true,
  pageSizeLabel = "Rows per page",
}: PaginationProps) {
  const computedTotalPages = React.useMemo(() => {
    if (typeof totalPagesProp === "number") return totalPagesProp;
    if (typeof totalItems === "number" && typeof pageSize === "number" && pageSize > 0) {
      return Math.max(1, Math.ceil(totalItems / pageSize));
    }
    return 1;
  }, [totalPagesProp, totalItems, pageSize]);

  const totalPages = Math.max(1, computedTotalPages);
  const safePage = clamp(page, 1, totalPages);

  const canGoPrev = safePage > 1;
  const canGoNext = safePage < totalPages;

  const items = React.useMemo(
    () =>
      getPaginationItems({
        page: safePage,
        totalPages,
        siblingCount,
        boundaryCount,
      }),
    [safePage, totalPages, siblingCount, boundaryCount]
  );

  const handlePrev = () => {
    if (disabled || !canGoPrev) return;
    onPageChange(safePage - 1);
  };

  const handleNext = () => {
    if (disabled || !canGoNext) return;
    onPageChange(safePage + 1);
  };

  const handlePageClick = (targetPage: number) => {
    if (disabled) return;
    if (targetPage === safePage) return;
    onPageChange(targetPage);
  };

  const showSizeSelector =
    showPageSizeSelector && typeof pageSize === "number" && !!onPageSizeChange;

  return (
    <div
      className={cn(
        "flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between",
        className
      )}
    >
      <nav aria-label="Pagination" className="w-full">
        <ul className="flex flex-wrap items-center justify-center gap-1">
          <li>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={handlePrev}
              disabled={disabled || !canGoPrev}
              aria-label="Go to previous page"
            >
              Previous
            </Button>
          </li>

          {items.map((item: PaginationItem, idx: number) => {
            if (item === "ellipsis") {
              return (
                <li key={`ellipsis-${idx}`} aria-hidden="true">
                  <span className="px-2 text-sm text-muted-foreground">…</span>
                </li>
              );
            }

            const isActive = item === safePage;

            return (
              <li key={item}>
                <Button
                  type="button"
                  variant={isActive ? "default" : "outline"}
                  size="sm"
                  onClick={() => handlePageClick(item)}
                  disabled={disabled}
                  aria-current={isActive ? "page" : undefined}
                  aria-label={isActive ? `Page ${item}, current page` : `Go to page ${item}`}
                >
                  {item}
                </Button>
              </li>
            );
          })}

          <li>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={handleNext}
              disabled={disabled || !canGoNext}
              aria-label="Go to next page"
            >
              Next
            </Button>
          </li>
        </ul>
      </nav>

      {showSizeSelector && (
        <div className="flex items-center justify-center gap-2 sm:justify-end">
          <span className="text-sm text-muted-foreground">{pageSizeLabel}</span>
          <Select
            value={String(pageSize)}
            onValueChange={(v: string) => {
              const nextSize = Number(v);
              if (!Number.isFinite(nextSize)) return;
              onPageSizeChange(nextSize);
            }}
            disabled={disabled}
          >
            <SelectTrigger className="h-9 w-[110px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {pageSizeOptions.map((opt) => (
                <SelectItem key={opt} value={String(opt)}>
                  {opt}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      )}
    </div>
  );
}
