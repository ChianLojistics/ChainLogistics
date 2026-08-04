import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { ProductCard } from './ProductCard';

const mockProduct = {
  id: 'prod-123',
  name: 'Test Product',
  description: 'A test product used in unit tests.',
  category: 'Coffee',
  owner: 'GCFXHS5DRCQZ4QZ7Z4X7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7',
  origin: { location: 'Ethiopia' },
  created_at: Math.floor(Date.now() / 1000) - 86400 * 30,
  active: true,
  eventCount: 5,
  tags: ['organic', 'fair-trade'],
};

describe('ProductCard', () => {
  it('renders product name', () => {
    render(<ProductCard product={mockProduct} />);
    expect(screen.getByText('Test Product')).toBeInTheDocument();
  });

  it('renders product ID', () => {
    render(<ProductCard product={mockProduct} />);
    expect(screen.getByText(/ID: prod-123/)).toBeInTheDocument();
  });

  it('renders origin location', () => {
    render(<ProductCard product={mockProduct} />);
    expect(screen.getByText('Ethiopia')).toBeInTheDocument();
  });

  it('renders Active badge for active products', () => {
    render(<ProductCard product={mockProduct} />);
    expect(screen.getByText('Active')).toBeInTheDocument();
  });

  it('renders Inactive badge for inactive products', () => {
    render(<ProductCard product={{ ...mockProduct, active: false }} />);
    expect(screen.getByText('Inactive')).toBeInTheDocument();
  });

  it('renders event count', () => {
    render(<ProductCard product={mockProduct} />);
    expect(screen.getByText(/5 Events/)).toBeInTheDocument();
  });

  it('has a link to product details', () => {
    render(<ProductCard product={mockProduct} />);
    const links = screen.getAllByRole('link');
    expect(links.some((link) => link.getAttribute('href') === '/products/prod-123')).toBe(true);
  });

  it('renders category value', () => {
    render(<ProductCard product={mockProduct} />);
    expect(screen.getByText('Coffee')).toBeInTheDocument();
  });
});
