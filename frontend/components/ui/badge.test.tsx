import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { Badge } from './badge';

describe('Badge', () => {
  it('renders with default variant', () => {
    render(<Badge>Active</Badge>);
    const badge = screen.getByRole('status');
    expect(badge).toHaveTextContent('Active');
    expect(badge).toHaveClass('bg-primary');
  });

  it('applies the destructive variant', () => {
    render(<Badge variant="destructive">Error</Badge>);
    const badge = screen.getByRole('status');
    expect(badge).toHaveClass('bg-destructive');
  });

  it('applies the outline variant', () => {
    render(<Badge variant="outline">Outline</Badge>);
    const badge = screen.getByRole('status');
    expect(badge).toHaveClass('text-foreground');
  });

  it('applies the secondary variant', () => {
    render(<Badge variant="secondary">Secondary</Badge>);
    const badge = screen.getByRole('status');
    expect(badge).toHaveClass('bg-secondary');
  });
});
