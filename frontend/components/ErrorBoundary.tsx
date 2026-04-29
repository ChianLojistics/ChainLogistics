"use client";

import * as React from "react";
import { trackError } from "@/lib/analytics";
import { classifyError, type ClassifiedError } from "@/lib/errors";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

type ErrorBoundaryProps = {
  children: React.ReactNode;
  title?: string;
  description?: string;
  onReset?: () => void;
  resetLabel?: string;
  onError?: (classified: ClassifiedError) => void;
};

type ErrorBoundaryState = {
  hasError: boolean;
  error: Error | null;
  classified: ClassifiedError | null;
};

export class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { hasError: false, error: null, classified: null };

  private boundHandleUnhandledRejection: ((event: PromiseRejectionEvent) => void) | null = null;
  private boundHandleWindowError: ((event: ErrorEvent) => void) | null = null;

  componentDidMount() {
    this.boundHandleUnhandledRejection = this.handleUnhandledRejection.bind(this);
    this.boundHandleWindowError = this.handleWindowError.bind(this);

    window.addEventListener("unhandledrejection", this.boundHandleUnhandledRejection);
    window.addEventListener("error", this.boundHandleWindowError);
  }

  componentWillUnmount() {
    if (this.boundHandleUnhandledRejection) {
      window.removeEventListener("unhandledrejection", this.boundHandleUnhandledRejection);
    }
    if (this.boundHandleWindowError) {
      window.removeEventListener("error", this.boundHandleWindowError);
    }
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    const classified = classifyError(error);
    return { hasError: true, error, classified };
  }

  componentDidCatch(error: Error) {
    const classified = classifyError(error);
    trackError(error, {
      source: "ErrorBoundary.componentDidCatch",
      component: this.constructor.name,
      category: classified.category,
    });
    this.props.onError?.(classified);
  }

  private handleUnhandledRejection(event: PromiseRejectionEvent) {
    const error = event.reason instanceof Error
      ? event.reason
      : new Error(String(event.reason));

    const classified = classifyError(error);
    trackError(error, {
      source: "unhandledrejection",
      category: classified.category,
    });

    this.setState({ hasError: true, error, classified });
    this.props.onError?.(classified);
    event.preventDefault();
  }

  private handleWindowError(event: ErrorEvent) {
    const error = event.error instanceof Error
      ? event.error
      : new Error(event.message || "An unexpected error occurred.");

    const classified = classifyError(error);
    trackError(error, {
      source: "window.error",
      category: classified.category,
      filename: event.filename,
      lineno: event.lineno,
    });

    this.setState({ hasError: true, error, classified });
    this.props.onError?.(classified);
  }

  private handleReset = () => {
    this.setState({ hasError: false, error: null, classified: null });
    this.props.onReset?.();
  };

  render() {
    if (!this.state.hasError) return this.props.children;

    const classified = this.state.classified;
    const title = this.props.title ?? classified?.title ?? "Something went wrong";
    const description =
      this.props.description ??
      classified?.message ??
      "The page had an unexpected problem. You can try again.";

    const categoryStyles: Record<string, string> = {
      network: "border-yellow-200 bg-yellow-50",
      contract: "border-orange-200 bg-orange-50",
      wallet: "border-blue-200 bg-blue-50",
      validation: "border-amber-200 bg-amber-50",
      authentication: "border-purple-200 bg-purple-50",
      business_logic: "border-indigo-200 bg-indigo-50",
      system: "border-red-200 bg-red-50",
      user: "border-gray-200 bg-gray-50",
      unknown: "border-red-200 bg-red-50",
    };

    const categoryTextStyles: Record<string, string> = {
      network: "text-yellow-900",
      contract: "text-orange-900",
      wallet: "text-blue-900",
      validation: "text-amber-900",
      authentication: "text-purple-900",
      business_logic: "text-indigo-900",
      system: "text-red-900",
      user: "text-gray-900",
      unknown: "text-red-900",
    };

    const category = classified?.category ?? "unknown";

    return (
      <div className={cn("rounded-xl border p-6 text-center", categoryStyles[category])}>
        <p className={cn("text-sm font-semibold", categoryTextStyles[category])}>{title}</p>
        <p className={cn("mt-1 text-sm", categoryTextStyles[category], "opacity-80")}>
          {description}
        </p>

        {classified && classified.recoverySteps.length > 0 && (
          <ul className={cn("mt-3 space-y-1 text-left text-xs", categoryTextStyles[category], "opacity-70")}>
            {classified.recoverySteps.map((step, idx) => (
              <li key={idx} className="flex items-start gap-1.5">
                <span className="mt-0.5 block h-1 w-1 flex-shrink-0 rounded-full bg-current" />
                {step}
              </li>
            ))}
          </ul>
        )}

        <div className="mt-5 flex items-center justify-center gap-2">
          <Button
            type="button"
            onClick={this.handleReset}
            size="sm"
          >
            {this.props.resetLabel ?? "Try again"}
          </Button>
        </div>
      </div>
    );
  }
}
