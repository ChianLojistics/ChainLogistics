import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { Footer } from './Footer';

describe('Footer', () => {
  it('renders the brand name', () => {
    render(<Footer />);
    expect(screen.getByText('ChainLojistic')).toBeInTheDocument();
  });

  it('renders navigation sections', () => {
    render(<Footer />);
    expect(screen.getByText('Product')).toBeInTheDocument();
    expect(screen.getByText('Company')).toBeInTheDocument();
    expect(screen.getByText('Resources')).toBeInTheDocument();
    expect(screen.getByText('Legal')).toBeInTheDocument();
  });

  it('renders social links with aria labels', () => {
    render(<Footer />);
    const githubLink = screen.getByLabelText('GitHub');
    const twitterLink = screen.getByLabelText('Twitter');
    const discordLink = screen.getByLabelText('Discord');
    expect(githubLink).toHaveAttribute('href', 'https://github.com/ChainLojistics/ChainLogistics');
    expect(twitterLink).toBeInTheDocument();
    expect(discordLink).toBeInTheDocument();
  });

  it('renders product navigation links', () => {
    render(<Footer />);
    expect(screen.getByText('Features')).toBeInTheDocument();
    expect(screen.getByText('How It Works')).toBeInTheDocument();
    expect(screen.getByText('Use Cases')).toBeInTheDocument();
  });

  it('renders copyright notice', () => {
    render(<Footer />);
    const year = new Date().getFullYear();
    expect(screen.getByText(new RegExp(String(year)))).toBeInTheDocument();
  });
});
