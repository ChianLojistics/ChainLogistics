import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { Navigation } from './Navigation';

describe('Navigation', () => {
  it('renders the brand name', () => {
    render(<Navigation />);
    expect(screen.getByText('ChainLojistic')).toBeInTheDocument();
  });

  it('renders desktop navigation links', () => {
    render(<Navigation />);
    expect(screen.getByText('Features')).toBeInTheDocument();
    expect(screen.getByText('How It Works')).toBeInTheDocument();
    expect(screen.getByText('Use Cases')).toBeInTheDocument();
    expect(screen.getByText('Get Started')).toBeInTheDocument();
  });

  it('renders mobile menu button', () => {
    render(<Navigation />);
    const menuBtn = screen.getByLabelText('Open menu');
    expect(menuBtn).toBeInTheDocument();
    expect(menuBtn).toHaveAttribute('aria-expanded', 'false');
  });

  it('toggles mobile menu when button is clicked', () => {
    render(<Navigation />);
    const menuBtn = screen.getByLabelText('Open menu');
    fireEvent.click(menuBtn);
    expect(menuBtn).toHaveAttribute('aria-expanded', 'true');
  });

  it('has proper nav aria label', () => {
    render(<Navigation />);
    expect(screen.getByLabelText('Main navigation')).toBeInTheDocument();
  });
});
