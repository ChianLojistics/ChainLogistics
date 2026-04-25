import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { InlineErrorState } from "./InlineErrorState";

describe("InlineErrorState component", () => {
  it("renders the title and message", () => {
    render(<InlineErrorState title="Something went wrong" message="Please try again later." />);
    expect(screen.getByText("Something went wrong")).toBeInTheDocument();
    expect(screen.getByText("Please try again later.")).toBeInTheDocument();
  });

  it("has role=alert for screen reader announcements", () => {
    render(<InlineErrorState title="Error" message="An error occurred." />);
    expect(screen.getByRole("alert")).toBeInTheDocument();
  });

  it("renders retry button when canRetry and onRetry are provided", () => {
    const onRetry = vi.fn();
    render(
      <InlineErrorState
        title="Network Error"
        message="Failed to load."
        canRetry
        onRetry={onRetry}
      />
    );
    const button = screen.getByRole("button", { name: /try again/i });
    expect(button).toBeInTheDocument();
    fireEvent.click(button);
    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it("does not render retry button when canRetry is false", () => {
    render(<InlineErrorState title="Error" message="Failed." canRetry={false} />);
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });

  it("renders recovery steps when provided", () => {
    render(
      <InlineErrorState
        title="Wallet Error"
        message="Could not connect."
        recoverySteps={["Check your wallet extension", "Try reconnecting"]}
      />
    );
    expect(screen.getByText("Check your wallet extension")).toBeInTheDocument();
    expect(screen.getByText("Try reconnecting")).toBeInTheDocument();
  });

  it("renders optional detail when provided", () => {
    render(
      <InlineErrorState
        title="Error"
        message="Failed."
        detail="Raw error: timeout after 30s"
      />
    );
    expect(screen.getByText(/Raw error: timeout/)).toBeInTheDocument();
  });

  it("disables retry button while isRetrying", () => {
    const onRetry = vi.fn();
    render(
      <InlineErrorState
        title="Error"
        message="Failed."
        canRetry
        onRetry={onRetry}
        isRetrying
      />
    );
    const button = screen.getByRole("button", { name: /try again/i });
    expect(button).toBeDisabled();
  });
});
