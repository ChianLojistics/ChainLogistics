"use client";

import * as React from "react";
import { AlertCircle, AlertTriangle, WifiOff, Wallet, RefreshCw, Info } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { type ErrorCategory } from "@/lib/errors";

export type InlineErrorVariant = "error" | "warning" | "info";

export interface InlineErrorStateProps {
  /** Short headline shown in bold */
  title: string;
  /** Detailed message providing context */
  message: string;
  /** Optional extra detail (e.g., raw error message for devs) */
  detail?: string;
  /** Visual severity level */
  variant?: InlineErrorVariant;
  /** Error category drives the icon selection */
  category?: ErrorCategory;
  /** List of recovery steps to display */
  recoverySteps?: string[];
  /** When true a "Try again" button is rendered */
  canRetry?: boolean;
  /** Called when the retry button is clicked */
  onRetry?: () => void;
  /** Label for the retry button */
  retryLabel?: string;
  /** Whether the retry action is currently in progress */
  isRetrying?: boolean;
  className?: string;
}

const variantStyles: Record<InlineErrorVariant, string> = {
  error: "border-red-200 bg-red-50 text-red-900",
  warning: "border-amber-200 bg-amber-50 text-amber-900",
  info: "border-blue-200 bg-blue-50 text-blue-900",
};

const retryButtonStyles: Record<InlineErrorVariant, string> = {
  error: "bg-red-600 text-white hover:bg-red-700 focus-visible:ring-red-500",
  warning: "bg-amber-600 text-white hover:bg-amber-700 focus-visible:ring-amber-500",
  info: "bg-blue-600 text-white hover:bg-blue-700 focus-visible:ring-blue-500",
};

function CategoryIcon({
  category,
  variant,
  className,
}: {
  category: ErrorCategory | undefined;
  variant: InlineErrorVariant;
  className?: string;
}) {
  const iconClass = cn("h-5 w-5 flex-shrink-0", className);

  switch (category) {
    case "network":
      return <WifiOff className={iconClass} aria-hidden="true" />;
    case "wallet":
      return <Wallet className={iconClass} aria-hidden="true" />;
    case "validation":
      return <Info className={iconClass} aria-hidden="true" />;
    default:
      return variant === "warning"
        ? <AlertTriangle className={iconClass} aria-hidden="true" />
        : <AlertCircle className={iconClass} aria-hidden="true" />;
  }
}

/**
 * A consistent inline error/warning/info banner for async data-fetch or form
 * submission failures. Pairs naturally with the class-based `ErrorBoundary`
 * which handles React render-time errors.
 */
export function InlineErrorState({
  title,
  message,
  detail,
  variant = "error",
  category,
  recoverySteps,
  canRetry = false,
  onRetry,
  retryLabel = "Try again",
  isRetrying = false,
  className,
}: Readonly<InlineErrorStateProps>) {
  const styles = variantStyles[variant];

  return (
    <div
      role="alert"
      aria-live="assertive"
      className={cn("rounded-xl border p-5", styles, className)}
    >
      <div className="flex items-start gap-3">
        <CategoryIcon category={category} variant={variant} />

        <div className="min-w-0 flex-1">
          <p className="text-sm font-semibold leading-snug">{title}</p>
          <p className="mt-1 text-sm opacity-80">{message}</p>

          {detail && (
            <p className="mt-1 font-mono text-xs opacity-60 break-all">{detail}</p>
          )}

          {recoverySteps && recoverySteps.length > 0 && (
            <ul className="mt-3 space-y-1 text-xs opacity-70">
              {recoverySteps.map((step, idx) => (
                <li key={idx} className="flex items-start gap-1.5">
                  <span className="mt-1 block h-1 w-1 flex-shrink-0 rounded-full bg-current" />
                  {step}
                </li>
              ))}
            </ul>
          )}

          {canRetry && onRetry && (
            <div className="mt-4">
              <Button
                type="button"
                size="sm"
                disabled={isRetrying}
                onClick={onRetry}
                className={cn(
                  "inline-flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs font-semibold",
                  retryButtonStyles[variant]
                )}
              >
                {isRetrying && (
                  <RefreshCw className="h-3.5 w-3.5 animate-spin" aria-hidden="true" />
                )}
                {retryLabel}
              </Button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
