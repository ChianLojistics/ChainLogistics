import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { Features } from './Features';

describe('Features', () => {
  it('renders section heading', () => {
    render(<Features />);
    expect(screen.getByText('Everything you need for transparent supply chains')).toBeInTheDocument();
  });

  it('renders all feature cards', () => {
    render(<Features />);
    expect(screen.getByText('Immutable Records')).toBeInTheDocument();
    expect(screen.getByText('Multi-Party Verification')).toBeInTheDocument();
    expect(screen.getByText('QR Code Verification')).toBeInTheDocument();
    expect(screen.getByText('Real-Time Visibility')).toBeInTheDocument();
    expect(screen.getByText('Fraud Prevention')).toBeInTheDocument();
    expect(screen.getByText('Certificate Verification')).toBeInTheDocument();
  });

  it('renders feature descriptions', () => {
    render(<Features />);
    expect(screen.getByText(/Every product event is recorded/)).toBeInTheDocument();
    expect(screen.getByText(/Farmers, processors, shippers/)).toBeInTheDocument();
  });

  it('has the features section id', () => {
    const { container } = render(<Features />);
    const section = container.querySelector('#features');
    expect(section).toBeInTheDocument();
  });
});
