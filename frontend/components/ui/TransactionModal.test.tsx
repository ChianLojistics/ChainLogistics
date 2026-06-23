import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { TransactionModal } from './TransactionModal';

describe('TransactionModal', () => {
  it('renders with default props', () => {
    render(
      <TransactionModal
        open
        onOpenChange={vi.fn()}
        step="preparing"
      />
    );
    expect(screen.getByText('Transaction')).toBeInTheDocument();
    expect(screen.getByText('Follow the transaction progress.')).toBeInTheDocument();
  });

  it('shows progress bar with correct value', () => {
    render(
      <TransactionModal
        open
        onOpenChange={vi.fn()}
        step="submitting"
      />
    );
    const progressbar = screen.getByRole('progressbar');
    expect(progressbar).toHaveAttribute('aria-valuenow', '75');
  });

  it('shows transaction steps', () => {
    render(
      <TransactionModal
        open
        onOpenChange={vi.fn()}
        step="signing"
      />
    );
    expect(screen.getByText('Preparing')).toBeInTheDocument();
    expect(screen.getByText('Signing')).toBeInTheDocument();
    expect(screen.getByText('Submitting')).toBeInTheDocument();
    expect(screen.getByText('Complete')).toBeInTheDocument();
  });

  it('shows confirmation when complete', () => {
    render(
      <TransactionModal
        open
        onOpenChange={vi.fn()}
        step="complete"
      />
    );
    expect(screen.getByText('Transaction confirmed')).toBeInTheDocument();
  });

  it('shows error state', () => {
    render(
      <TransactionModal
        open
        onOpenChange={vi.fn()}
        step="error"
        errorTitle="Transaction failed"
        errorMessage="Insufficient funds"
      />
    );
    expect(screen.getByText('Transaction failed')).toBeInTheDocument();
    expect(screen.getByText('Insufficient funds')).toBeInTheDocument();
  });

  it('renders retry button in error state', () => {
    const onRetry = vi.fn();
    render(
      <TransactionModal
        open
        onOpenChange={vi.fn()}
        step="error"
        onRetry={onRetry}
      />
    );
    const retryBtn = screen.getByRole('button', { name: /retry/i });
    fireEvent.click(retryBtn);
    expect(onRetry).toHaveBeenCalledTimes(1);
  });
});
