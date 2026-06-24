import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { AsyncState, LoadingOverlay } from './async-state';

describe('AsyncState', () => {
  it('renders loading state by default', () => {
    render(
      <AsyncState isLoading error={null} data={null}>
        <div>Content</div>
      </AsyncState>
    );
    expect(screen.getByLabelText('Loading content')).toBeInTheDocument();
  });

  it('renders custom loading fallback', () => {
    render(
      <AsyncState isLoading error={null} data={null} loadingFallback={<div>Custom Loading</div>}>
        <div>Content</div>
      </AsyncState>
    );
    expect(screen.getByText('Custom Loading')).toBeInTheDocument();
  });

  it('renders error state with role alert', () => {
    render(
      <AsyncState isLoading={false} error={new Error('Test error')} data={null}>
        <div>Content</div>
      </AsyncState>
    );
    expect(screen.getByRole('alert')).toBeInTheDocument();
  });

  it('renders retry button when onRetry is provided', () => {
    const onRetry = vi.fn();
    render(
      <AsyncState isLoading={false} error={new Error('Error')} data={null} onRetry={onRetry}>
        <div>Content</div>
      </AsyncState>
    );
    const retryBtn = screen.getByRole('button', { name: /retry/i });
    fireEvent.click(retryBtn);
    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it('renders empty fallback when data is empty', () => {
    render(
      <AsyncState isLoading={false} error={null} data={null} emptyFallback={<div>Empty State</div>}>
        <div>Content</div>
      </AsyncState>
    );
    expect(screen.getByText('Empty State')).toBeInTheDocument();
  });

  it('renders children when data is present', () => {
    render(
      <AsyncState isLoading={false} error={null} data={{ items: [] }}>
        <div>Content</div>
      </AsyncState>
    );
    expect(screen.getByText('Content')).toBeInTheDocument();
  });
});

describe('LoadingOverlay', () => {
  it('renders when visible', () => {
    render(<LoadingOverlay isVisible />);
    expect(screen.getByLabelText('Loading...')).toBeInTheDocument();
  });

  it('shows custom message', () => {
    render(<LoadingOverlay isVisible message="Processing..." />);
    expect(screen.getByText('Processing...')).toBeInTheDocument();
  });

  it('renders cancel button when onCancel provided', () => {
    const onCancel = vi.fn();
    render(<LoadingOverlay isVisible onCancel={onCancel} />);
    const cancelBtn = screen.getByRole('button', { name: /cancel/i });
    fireEvent.click(cancelBtn);
    expect(onCancel).toHaveBeenCalledTimes(1);
  });

  it('returns null when not visible', () => {
    const { container } = render(<LoadingOverlay isVisible={false} />);
    expect(container.firstChild).toBeNull();
  });
});
