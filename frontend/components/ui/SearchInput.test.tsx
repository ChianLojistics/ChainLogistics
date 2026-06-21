import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { SearchInput } from './SearchInput';

const suggestions = [
  { id: '1', label: 'Apple' },
  { id: '2', label: 'Banana' },
  { id: '3', label: 'Orange' },
];

describe('SearchInput', () => {
  it('renders input with placeholder', () => {
    render(<SearchInput suggestions={suggestions} />);
    const input = screen.getByRole('combobox');
    expect(input).toBeInTheDocument();
    expect(input).toHaveAttribute('placeholder', 'Search…');
  });

  it('has proper ARIA attributes', () => {
    render(<SearchInput suggestions={suggestions} />);
    const input = screen.getByRole('combobox');
    expect(input).toHaveAttribute('aria-autocomplete', 'list');
    expect(input).toHaveAttribute('aria-expanded', 'false');
  });

  it('shows suggestions on input change', () => {
    render(<SearchInput suggestions={suggestions} />);
    const input = screen.getByRole('combobox');
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: 'Ap' } });
    expect(screen.getByRole('option')).toBeInTheDocument();
  });

  it('calls onValueChange when input changes', () => {
    const onValueChange = vi.fn();
    render(<SearchInput suggestions={suggestions} onValueChange={onValueChange} />);
    const input = screen.getByRole('combobox');
    fireEvent.change(input, { target: { value: 'test' } });
    expect(onValueChange).toHaveBeenCalledWith('test');
  });

  it('calls onSelectSuggestion when suggestion is clicked', () => {
    const onSelect = vi.fn();
    render(<SearchInput suggestions={suggestions} onSelectSuggestion={onSelect} />);
    const input = screen.getByRole('combobox');
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: 'Ap' } });
    const option = screen.getByRole('option');
    fireEvent.mouseDown(option);
    expect(onSelect).toHaveBeenCalledWith(expect.objectContaining({ id: '1', label: 'Apple' }));
  });

  it('shows empty text when no suggestions match', () => {
    render(<SearchInput suggestions={suggestions} emptyText="No results" />);
    const input = screen.getByRole('combobox');
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: 'xyz' } });
    expect(screen.getByText('No results')).toBeInTheDocument();
  });
});
