import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { ProductCard } from './ProductCard';
import type { Product } from '@/lib/types/product';

// Mock next/link
vi.mock('next/link', () => ({
  default: ({ children, href }: { children: React.ReactNode; href: string }) => (
    <a href={href}>{children}</a>
  ),
}));

// Mock next/navigation
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: vi.fn(),
  }),
}));

const mockProduct: Product = {
  id: 'PROD-TEST-123',
  name: 'Organic Coffee Beans',
  description: 'Premium organic coffee from Ethiopia',
  origin: {
    location: 'Ethiopia',
  },
  owner: 'GCFXHS5DRCQZ4QZ7Z4X7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7',
  created_at: Date.now() / 1000,
  active: true,
  category: 'Food & Beverage',
  tags: ['organic', 'fair-trade', 'premium'],
  eventCount: 5,
};

describe('ProductCard component', () => {
  it('should render product information correctly', () => {
    render(<ProductCard product={mockProduct} />);

    expect(screen.getByText('Organic Coffee Beans')).toBeInTheDocument();
    expect(screen.getByText('ID: PROD-TEST-123')).toBeInTheDocument();
    expect(screen.getByText('Ethiopia')).toBeInTheDocument();
    expect(screen.getByText('Food & Beverage')).toBeInTheDocument();
  });

  it('should display active status badge correctly', () => {
    render(<ProductCard product={mockProduct} />);

    const statusBadge = screen.getByText('Active');
    expect(statusBadge).toBeInTheDocument();
  });

  it('should display inactive status badge correctly', () => {
    const inactiveProduct = { ...mockProduct, active: false };
    render(<ProductCard product={inactiveProduct} />);

    const statusBadge = screen.getByText('Inactive');
    expect(statusBadge).toBeInTheDocument();
  });

  it('should display event count correctly', () => {
    render(<ProductCard product={mockProduct} />);

    expect(screen.getByText('5 Events')).toBeInTheDocument();
  });

  it('should display singular event count correctly', () => {
    const singleEventProduct = { ...mockProduct, eventCount: 1 };
    render(<ProductCard product={singleEventProduct} />);

    expect(screen.getByText('1 Event')).toBeInTheDocument();
  });

  it('should truncate owner address correctly', () => {
    render(<ProductCard product={mockProduct} />);

    expect(screen.getByText('GCFX…Z7Z7')).toBeInTheDocument();
  });

  it('should display tags when available', () => {
    render(<ProductCard product={mockProduct} />);

    expect(screen.getByText('organic')).toBeInTheDocument();
    expect(screen.getByText('fair-trade')).toBeInTheDocument();
  });

  it('should have correct link to product details', () => {
    render(<ProductCard product={mockProduct} />);

    const viewLink = screen.getByRole('link', { name: /^view$/i });
    expect(viewLink).toHaveAttribute('href', '/products/PROD-TEST-123');
  });

  it('should have correct link to add event page', () => {
    render(<ProductCard product={mockProduct} />);

    const addEventLink = screen.getByRole('link', { name: /add event/i });
    expect(addEventLink).toHaveAttribute('href', '/products/PROD-TEST-123/add-event');
  });

  it('should format date correctly', () => {
    render(<ProductCard product={mockProduct} />);

    const formattedDate = new Date(mockProduct.created_at * 1000).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });

    expect(screen.getByText(formattedDate)).toBeInTheDocument();
  });

  it('should have accessible elements', () => {
    render(<ProductCard product={mockProduct} />);

    expect(screen.getByRole('link', { name: /add event/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /view details/i })).toBeInTheDocument();
  });
});
