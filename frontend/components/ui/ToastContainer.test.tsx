import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { ToastContainer } from './ToastContainer';
import { useToastStore } from '@/lib/toast/store';

vi.mock('@/lib/toast/store', () => ({
  useToastStore: vi.fn(),
}));

const mockToast = {
  id: 'toast-1',
  type: 'info' as const,
  title: 'Info',
  message: 'Info message',
};

describe('ToastContainer', () => {
  it('renders nothing when no toasts', () => {
    vi.mocked(useToastStore).mockReturnValue({
      toasts: [],
      position: 'top-right',
      removeToast: vi.fn(),
    });
    const { container } = render(<ToastContainer />);
    expect(container.firstChild).toBeNull();
  });

  it('renders toasts in a region with aria label', () => {
    vi.mocked(useToastStore).mockReturnValue({
      toasts: [mockToast],
      position: 'top-right',
      removeToast: vi.fn(),
    });
    render(<ToastContainer />);
    expect(screen.getByRole('region', { name: /notifications/i })).toBeInTheDocument();
    expect(screen.getByText('Info message')).toBeInTheDocument();
  });

  it('renders multiple toasts', () => {
    vi.mocked(useToastStore).mockReturnValue({
      toasts: [
        mockToast,
        { ...mockToast, id: 'toast-2', title: 'Warning', type: 'warning' as const },
      ],
      position: 'top-right',
      removeToast: vi.fn(),
    });
    render(<ToastContainer />);
    expect(screen.getByText('Info')).toBeInTheDocument();
    expect(screen.getByText('Warning')).toBeInTheDocument();
  });
});
