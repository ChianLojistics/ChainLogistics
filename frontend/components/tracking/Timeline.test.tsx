import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { Timeline } from './Timeline';
import { fetchProductEventsPage } from '@/lib/contract/events';
import type { TimelineEvent } from '@/lib/types/tracking';

// Mock the events module
vi.mock('@/lib/contract/events', () => ({
  fetchProductEventsPage: vi.fn(),
  getRelativeTime: vi.fn((ts: number) => `${Math.floor((Date.now() / 1000 - ts) / 86400)} days ago`),
  formatEventTimestamp: vi.fn((ts: number) => new Date(ts * 1000).toLocaleString()),
}));

const mockFetchProductEventsPage = vi.mocked(fetchProductEventsPage);

const mockEvents: TimelineEvent[] = [
  {
    event_id: 1,
    product_id: 'prod-123',
    actor: 'GCFXHS5DRCQZ4QZ7Z4X7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7',
    event_type: 'HARVEST',
    timestamp: Math.floor(Date.now() / 1000) - 86400 * 3, // 3 days ago
    note: 'Harvested coffee cherries',
  },
  {
    event_id: 2,
    product_id: 'prod-123',
    actor: 'GCFXHS5DRCQZ4QZ7Z4X7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7',
    event_type: 'SHIP',
    timestamp: Math.floor(Date.now() / 1000) - 86400 * 2, // 2 days ago
    note: 'Washed and dried',
  },
  {
    event_id: 3,
    product_id: 'prod-123',
    actor: 'GCFXHS5DRCQZ4QZ7Z4X7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7',
    event_type: 'RECEIVE',
    timestamp: Math.floor(Date.now() / 1000) - 86400, // 1 day ago
    note: 'Shipped to destination',
  },
];

describe('Timeline component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should show loading skeleton initially', () => {
    mockFetchProductEventsPage.mockImplementation(() => new Promise(() => {})); // Never resolves

    render(<Timeline productId="prod-123" />);

    expect(screen.getByTestId('timeline-skeleton')).toBeInTheDocument();
  });

  it('should display events after loading', async () => {
    mockFetchProductEventsPage.mockResolvedValue({
      events: mockEvents,
      total: 3,
      offset: 0,
      limit: 20,
      hasMore: false,
    });

    render(<Timeline productId="prod-123" />);

    await waitFor(() => {
      expect(screen.getByText('Harvest')).toBeInTheDocument();
      expect(screen.getByText('Ship')).toBeInTheDocument();
      expect(screen.getByText('Receive')).toBeInTheDocument();
    });
  });

  it('should display error message when fetch fails', async () => {
    mockFetchProductEventsPage.mockRejectedValue(new Error('Network error'));

    render(<Timeline productId="prod-123" />);

    await waitFor(() => {
      expect(screen.getByText(/failed to load events/i)).toBeInTheDocument();
      expect(screen.getByText(/network error/i)).toBeInTheDocument();
    });
  });

  it('should show empty state when no events exist', async () => {
    mockFetchProductEventsPage.mockResolvedValue({
      events: [],
      total: 0,
      offset: 0,
      limit: 20,
      hasMore: false,
    });

    render(<Timeline productId="prod-123" />);

    await waitFor(() => {
      expect(screen.getByText(/no tracking events yet/i)).toBeInTheDocument();
    });
  });

  it('should call fetch with correct parameters', async () => {
    mockFetchProductEventsPage.mockResolvedValue({
      events: mockEvents,
      total: 3,
      offset: 0,
      limit: 20,
      hasMore: false,
    });

    render(<Timeline productId="prod-123" />);

    await waitFor(() => {
      expect(mockFetchProductEventsPage).toHaveBeenCalledWith('prod-123', {
        offset: 0,
        limit: 20,
      });
    });
  });

  it('should show load more button when hasMore is true', async () => {
    mockFetchProductEventsPage.mockResolvedValue({
      events: mockEvents.slice(0, 2),
      total: 3,
      offset: 0,
      limit: 20,
      hasMore: true,
    });

    render(<Timeline productId="prod-123" />);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /load more/i })).toBeInTheDocument();
    });
  });

  it('should load more events when button is clicked', async () => {
    mockFetchProductEventsPage
      .mockResolvedValueOnce({
        events: mockEvents.slice(0, 2),
        total: 3,
        offset: 0,
        limit: 20,
        hasMore: true,
      })
      .mockResolvedValueOnce({
        events: mockEvents.slice(2),
        total: 3,
        offset: 2,
        limit: 20,
        hasMore: false,
      });

    render(<Timeline productId="prod-123" />);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /load more/i })).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole('button', { name: /load more/i }));

    await waitFor(() => {
      expect(mockFetchProductEventsPage).toHaveBeenCalledTimes(2);
      expect(mockFetchProductEventsPage).toHaveBeenLastCalledWith('prod-123', {
        offset: 2,
        limit: 20,
      });
    });
  });

  it('should have retry button when error occurs', async () => {
    mockFetchProductEventsPage.mockRejectedValueOnce(new Error('Network error'));

    render(<Timeline productId="prod-123" />);

    await waitFor(() => {
      const retryButton = screen.getByRole('button', { name: /retry/i });
      expect(retryButton).toBeInTheDocument();
    });
  });

  it('should retry loading when retry button is clicked', async () => {
    mockFetchProductEventsPage
      .mockRejectedValueOnce(new Error('Network error'))
      .mockResolvedValueOnce({
        events: mockEvents,
        total: 3,
        offset: 0,
        limit: 20,
        hasMore: false,
      });

    render(<Timeline productId="prod-123" />);

    await waitFor(() => {
      const retryButton = screen.getByRole('button', { name: /retry/i });
      expect(retryButton).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole('button', { name: /retry/i }));

    await waitFor(() => {
      expect(mockFetchProductEventsPage).toHaveBeenCalledTimes(2);
    });
  });
});
