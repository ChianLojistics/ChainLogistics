import { ReactNode } from "react";
import { LoadingSpinner } from "./loading-spinner";
import { cn } from "@/lib/utils";

interface AsyncStateProps {
  /**
   * Whether data is currently loading
   */
  isLoading: boolean;
  /**
   * Error if one occurred
   */
  error: Error | null;
  /**
   * Data is available
   */
  data?: unknown;
  /**
   * Content to render when loading
   */
  loadingFallback?: ReactNode;
  /**
   * Content to render when error occurs
   */
  errorFallback?: (error: Error, retry?: () => void) => ReactNode;
  /**
   * Content to render when data is empty
   */
  emptyFallback?: ReactNode;
  /**
   * Main content to render
   */
  children: ReactNode;
  /**
   * Optional retry function
   */
  onRetry?: () => void;
}

/**
 * Wrapper component for displaying async operation states
 * Handles loading, error, and empty states automatically
 */
export function AsyncState({
  isLoading,
  error,
  data,
  loadingFallback,
  errorFallback,
  emptyFallback,
  children,
  onRetry,
}: AsyncStateProps) {
  if (isLoading) {
    return (
      loadingFallback || (
        <div
          className="flex items-center justify-center py-12"
          role="status"
          aria-label="Loading content"
        >
          <div className="text-center">
            <LoadingSpinner size={40} className="mx-auto mb-4" />
            <p className="text-sm text-gray-600">Loading content...</p>
          </div>
        </div>
      )
    );
  }

  if (error) {
    return (
      errorFallback?.(error, onRetry) || (
        <div
          className="flex items-center justify-center py-12 px-4"
          role="alert"
        >
          <div className="text-center">
            <div className="text-red-600 mb-2">
              <svg
                className="w-12 h-12 mx-auto"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M12 8v4m0 4v.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
            </div>
            <h3 className="font-semibold text-gray-900 mb-1">
              Something went wrong
            </h3>
            <p className="text-sm text-gray-600 mb-4">{error.message}</p>
            {onRetry && (
              <button
                onClick={onRetry}
                className="text-blue-600 hover:text-blue-700 font-medium text-sm"
                aria-label="Retry loading"
              >
                Try Again
              </button>
            )}
          </div>
        </div>
      )
    );
  }

  if (!data && emptyFallback) {
    return <div role="status">{emptyFallback}</div>;
  }

  return <>{children}</>;
}

/**
 * Loading overlay component for blocking UI during operations
 */
export interface LoadingOverlayProps {
  /**
   * Show the overlay
   */
  isVisible: boolean;
  /**
   * Loading message
   */
  message?: string;
  /**
   * Show indeterminate progress
   */
  indeterminate?: boolean;
  /**
   * Allow cancellation
   */
  onCancel?: () => void;
  /**
   * Custom className
   */
  className?: string;
}

export function LoadingOverlay({
  isVisible,
  message = "Loading...",
  indeterminate = true,
  onCancel,
  className,
}: LoadingOverlayProps) {
  if (!isVisible) return null;

  return (
    <div
      className={cn(
        "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
        className
      )}
      role="status"
      aria-label={message}
      aria-modal="true"
    >
      <div className="bg-white rounded-lg shadow-lg p-8 max-w-sm w-full mx-4">
        <div className="text-center">
          <LoadingSpinner size={48} className="mx-auto mb-4" />
          <p className="text-gray-900 font-semibold mb-4">{message}</p>
          {indeterminate && (
            <div
              className="w-full h-1 bg-gray-200 rounded-full overflow-hidden"
              role="progressbar"
              aria-label="Loading progress"
            >
              <div className="h-full bg-blue-500 animate-pulse" />
            </div>
          )}
          {onCancel && (
            <button
              onClick={onCancel}
              className="mt-6 px-4 py-2 text-gray-700 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
              aria-label="Cancel operation"
            >
              Cancel
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
