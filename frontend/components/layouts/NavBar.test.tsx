import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { NavBar } from './NavBar';

// WalletStatus is mocked via setup.ts global mock
describe('NavBar', () => {
  it('renders the brand name', () => {
    render(<NavBar />);
    expect(screen.getByText('ChainLogistics')).toBeInTheDocument();
  });

  it('renders navigation links on desktop', () => {
    render(<NavBar />);
    const desktopNav = screen.getByLabelText('Main navigation');
    expect(desktopNav).toHaveTextContent('Dashboard');
    expect(desktopNav).toHaveTextContent('Register Product');
    expect(desktopNav).toHaveTextContent('Tracking');
  });

  it('renders mobile menu button', () => {
    render(<NavBar />);
    const menuBtn = screen.getByLabelText('Open menu');
    expect(menuBtn).toBeInTheDocument();
    expect(menuBtn).toHaveAttribute('aria-expanded', 'false');
  });

  it('toggles mobile menu when button is clicked', () => {
    render(<NavBar />);
    const menuBtn = screen.getByLabelText('Open menu');
    fireEvent.click(menuBtn);
    expect(menuBtn).toHaveAttribute('aria-expanded', 'true');
    expect(screen.getByLabelText('Close menu')).toBeInTheDocument();
  });

  it('toggles menu closed when button is clicked again', () => {
    render(<NavBar />);
    const menuBtn = screen.getByLabelText('Open menu');
    fireEvent.click(menuBtn);
    expect(menuBtn).toHaveAttribute('aria-expanded', 'true');
    fireEvent.click(menuBtn);
    expect(menuBtn).toHaveAttribute('aria-expanded', 'false');
  });

  it('has proper nav aria label', () => {
    render(<NavBar />);
    expect(screen.getByLabelText('Main navigation')).toBeInTheDocument();
  });
});
