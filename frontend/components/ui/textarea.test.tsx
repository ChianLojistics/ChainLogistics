import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { Textarea } from './textarea';

describe('Textarea', () => {
  it('renders with default props', () => {
    render(<Textarea />);
    const textarea = screen.getByRole('textbox');
    expect(textarea).toBeInTheDocument();
  });

  it('renders label when provided', () => {
    render(<Textarea label="Description" />);
    expect(screen.getByLabelText('Description')).toBeInTheDocument();
  });

  it('renders error message with role alert', () => {
    render(<Textarea label="Notes" error="This field is required" />);
    expect(screen.getByRole('alert')).toHaveTextContent('This field is required');
  });

  it('associates error with textarea via aria-describedby', () => {
    render(<Textarea label="Notes" error="Required" id="notes" />);
    const textarea = screen.getByLabelText('Notes');
    expect(textarea).toHaveAttribute('aria-invalid', 'true');
    expect(textarea).toHaveAttribute('aria-describedby');
  });

  it('renders without error state', () => {
    render(<Textarea label="Notes" />);
    const textarea = screen.getByLabelText('Notes');
    expect(textarea).not.toHaveAttribute('aria-invalid');
  });
});
