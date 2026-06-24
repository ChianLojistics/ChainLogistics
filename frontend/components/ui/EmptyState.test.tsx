import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { EmptyState } from './EmptyState';

describe('EmptyState', () => {
  it('renders title and description', () => {
    render(<EmptyState title="No items" description="No items found." />);
    expect(screen.getByText('No items')).toBeInTheDocument();
    expect(screen.getByText('No items found.')).toBeInTheDocument();
  });

  it('renders a CTA button with onClick', () => {
    const onClick = vi.fn();
    render(
      <EmptyState
        title="Empty"
        description="Nothing here"
        cta={{ label: 'Create', onClick }}
      />
    );
    const button = screen.getByRole('button', { name: /create/i });
    fireEvent.click(button);
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it('renders a CTA link with href', () => {
    render(
      <EmptyState
        title="Empty"
        description="Nothing here"
        cta={{ label: 'Go', href: '/create' }}
      />
    );
    const link = screen.getByRole('link', { name: /go/i });
    expect(link).toHaveAttribute('href', '/create');
  });

  it('renders with an icon', () => {
    render(
      <EmptyState
        title="Empty"
        icon={<span data-testid="test-icon" />}
      />
    );
    expect(screen.getByTestId('test-icon')).toBeInTheDocument();
  });
});
