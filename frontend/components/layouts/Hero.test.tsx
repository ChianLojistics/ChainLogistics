import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { Hero } from './Hero';

describe('Hero', () => {
  it('renders the main heading', () => {
    render(<Hero />);
    expect(screen.getByText(/Transparent supply chains/)).toBeInTheDocument();
  });

  it('renders the subtitle', () => {
    render(<Hero />);
    expect(screen.getByText(/immutable transparency/)).toBeInTheDocument();
  });

  it('renders CTA buttons', () => {
    render(<Hero />);
    expect(screen.getByText('Get Started Free')).toBeInTheDocument();
    expect(screen.getByText('See How It Works')).toBeInTheDocument();
  });

  it('renders trust indicators', () => {
    render(<Hero />);
    expect(screen.getByText('No credit card required')).toBeInTheDocument();
    expect(screen.getByText('Open source')).toBeInTheDocument();
  });

  it('renders powered by badge', () => {
    render(<Hero />);
    expect(screen.getByText('Powered by Stellar Blockchain')).toBeInTheDocument();
  });
});
