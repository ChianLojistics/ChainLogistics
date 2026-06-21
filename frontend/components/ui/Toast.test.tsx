import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { ToastComponent } from './Toast';

describe('ToastComponent', () => {
  const baseToast = {
    id: 'test-1',
    type: 'info' as const,
    title: 'Information',
    message: 'This is an info message',
  };

  it('renders title and message', () => {
    render(<ToastComponent toast={baseToast} onDismiss={vi.fn()} />);
    expect(screen.getByText('Information')).toBeInTheDocument();
    expect(screen.getByText('This is an info message')).toBeInTheDocument();
  });

  it('has role alert', () => {
    render(<ToastComponent toast={baseToast} onDismiss={vi.fn()} />);
    expect(screen.getByRole('alert')).toBeInTheDocument();
  });

  it('renders dismiss button', () => {
    render(<ToastComponent toast={baseToast} onDismiss={vi.fn()} />);
    expect(screen.getByLabelText('Dismiss notification')).toBeInTheDocument();
  });

  it('calls onDismiss when dismiss button is clicked', () => {
    const onDismiss = vi.fn();
    render(<ToastComponent toast={baseToast} onDismiss={onDismiss} />);
    fireEvent.click(screen.getByLabelText('Dismiss notification'));
    expect(onDismiss).toHaveBeenCalledWith('test-1');
  });

  it('renders success variant', () => {
    render(
      <ToastComponent
        toast={{ ...baseToast, type: 'success', title: 'Success' }}
        onDismiss={vi.fn()}
      />
    );
    expect(screen.getByText('Success')).toBeInTheDocument();
  });

  it('renders error variant', () => {
    render(
      <ToastComponent
        toast={{ ...baseToast, type: 'error', title: 'Error' }}
        onDismiss={vi.fn()}
      />
    );
    expect(screen.getByText('Error')).toBeInTheDocument();
  });

  it('renders action button when provided', () => {
    const onAction = vi.fn();
    render(
      <ToastComponent
        toast={{ ...baseToast, action: { label: 'Undo', onClick: onAction } }}
        onDismiss={vi.fn()}
      />
    );
    const actionBtn = screen.getByRole('button', { name: /undo/i });
    fireEvent.click(actionBtn);
    expect(onAction).toHaveBeenCalledTimes(1);
  });
});
