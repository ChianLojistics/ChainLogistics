import { render } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { Skeleton } from './skeleton';

describe('Skeleton', () => {
  it('renders and is hidden from screen readers', () => {
    const { container } = render(<Skeleton className="h-4 w-full" />);
    const div = container.firstChild as HTMLElement;
    expect(div).toHaveAttribute('aria-hidden', 'true');
    expect(div).toHaveClass('h-4', 'w-full');
  });

  it('applies animate-pulse class', () => {
    const { container } = render(<Skeleton />);
    expect(container.firstChild).toHaveClass('animate-pulse');
  });
});
