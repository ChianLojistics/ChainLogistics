import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { ProgressIndicator } from './progress-indicator';

const steps = [
  { id: '1', label: 'Step 1', description: 'First step' },
  { id: '2', label: 'Step 2', description: 'Second step' },
  { id: '3', label: 'Step 3', description: 'Third step' },
];

describe('ProgressIndicator', () => {
  it('renders steps with labels', () => {
    render(<ProgressIndicator steps={steps} currentStep={0} />);
    expect(screen.getByText('Step 1')).toBeInTheDocument();
    expect(screen.getByText('Step 2')).toBeInTheDocument();
    expect(screen.getByText('Step 3')).toBeInTheDocument();
  });

  it('has progressbar role with correct values', () => {
    render(<ProgressIndicator steps={steps} currentStep={1} />);
    const bar = screen.getByRole('progressbar');
    expect(bar).toHaveAttribute('aria-valuenow', '2');
    expect(bar).toHaveAttribute('aria-valuemin', '1');
    expect(bar).toHaveAttribute('aria-valuemax', '3');
  });

  it('shows completed steps with checkmark', () => {
    render(<ProgressIndicator steps={steps} currentStep={2} completedSteps={[0, 1]} />);
    const checkmarks = screen.getAllByLabelText('Completed');
    expect(checkmarks.length).toBeGreaterThanOrEqual(2);
  });

  it('renders in vertical orientation', () => {
    render(<ProgressIndicator steps={steps} currentStep={0} orientation="vertical" />);
    expect(screen.getByText('First step')).toBeInTheDocument();
    expect(screen.getByText('Second step')).toBeInTheDocument();
  });
});
