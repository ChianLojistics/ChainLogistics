"use client";

import * as React from "react";
import { trackError } from "@/lib/analytics";

type ErrorBoundaryProps = {
  children: React.ReactNode;
  title?: string;
  description?: string;
  onReset?: () => void;
  resetLabel?: string;
};

type ErrorBoundaryState = {
  hasError: boolean;
  error: Error | null;
};

export class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { hasError: false, error: null };

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error) {
    console.error(error);
    trackError(error, {
      source: "ErrorBoundary.componentDidCatch",
      component: this.constructor.name,
    });
  }

  private handleReset = () => {
    this.setState({ hasError: false, error: null });
    this.props.onReset?.();
  };

  render() {
    if (!this.state.hasError) return this.props.children;

    const title = this.props.title ?? "Something went wrong";
    const description =
      this.props.description ??
      "The page had an unexpected problem. You can try again.";

    return (
      <div className="rounded-xl border border-red-200 bg-red-50 p-6 text-center">
        <p className="text-sm font-semibold text-red-900">{title}</p>
        <p className="mt-1 text-sm text-red-700">{description}</p>
        {this.state.error?.message ? (
          <p className="mt-2 text-xs text-red-700 break-words">{this.state.error.message}</p>
        ) : null}
        <div className="mt-5 flex items-center justify-center gap-2">
          <button
            type="button"
            onClick={this.handleReset}
            className="rounded-lg bg-red-600 px-4 py-2 text-sm font-semibold text-white hover:bg-red-700"
          >
            {this.props.resetLabel ?? "Try again"}
          </button>
        </div>
      </div>
    );
  }
}
