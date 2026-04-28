import type { TimelineEvent } from "@/lib/types/tracking";
import { normalizeTimelineEvent } from "@/lib/types/tracking";
import { createContractClient } from "@/lib/stellar/contractClient";
import { trackContractInteraction, trackError } from "@/lib/analytics";
import { CONTRACT_CONFIG, validateContractConfig } from "./config";

export type ProductEventsPage = {
  events: TimelineEvent[];
  total: number;
  offset: number;
  limit: number;
  hasMore: boolean;
};

const DEFAULT_EVENTS_PAGE_SIZE = 20;
const MAX_EVENTS_PAGE_SIZE = 50;

const E2E_MOCKS_ENABLED = process.env.NEXT_PUBLIC_E2E_MOCKS === "true";

function getE2EMockEvents(productId: string): TimelineEvent[] {
  const nowSec = Math.floor(Date.now() / 1000);
  return [
    {
      event_id: 3,
      product_id: productId,
      actor: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
      timestamp: nowSec - 3600,
      event_type: "SHIP",
      note: "Mock: Shipment dispatched",
    },
    {
      event_id: 2,
      product_id: productId,
      actor: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
      timestamp: nowSec - 7200,
      event_type: "PACKAGE",
      note: "Mock: Packaged for shipping",
    },
    {
      event_id: 1,
      product_id: productId,
      actor: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
      timestamp: nowSec - 10_800,
      event_type: "HARVEST",
      note: "Mock: Harvested at origin",
    },
  ];
}

export async function fetchProductEventsPage(
  productId: string,
  options?: {
    offset?: number;
    limit?: number;
  }
): Promise<ProductEventsPage> {
  const startedAt = Date.now();
  const offset = Math.max(0, options?.offset ?? 0);
  const limit = Math.min(
    MAX_EVENTS_PAGE_SIZE,
    Math.max(1, options?.limit ?? DEFAULT_EVENTS_PAGE_SIZE)
  );

  try {
    if (E2E_MOCKS_ENABLED) {
      const all = getE2EMockEvents(productId);
      const paged = all.slice(offset, offset + limit);
      return {
        events: paged,
        total: all.length,
        offset,
        limit,
        hasMore: offset + limit < all.length,
      };
    }

    validateContractConfig();

    const contractClient = createContractClient({
      contractId: CONTRACT_CONFIG.CONTRACT_ID,
      network: CONTRACT_CONFIG.NETWORK,
      rpcUrl: CONTRACT_CONFIG.RPC_URL,
    });

    const eventIds = await contractClient.get_product_event_ids(productId);
    const sortedIds = [...eventIds].sort((a, b) => b - a);
    const pagedIds = sortedIds.slice(offset, offset + limit);

    const events = await Promise.all(pagedIds.map((id) => contractClient.get_event(id)));
    const validEvents = events
      .filter((e): e is TimelineEvent => e !== null)
      .map(normalizeTimelineEvent)
      .sort((a, b) => b.timestamp - a.timestamp);

    const page: ProductEventsPage = {
      events: validEvents,
      total: sortedIds.length,
      offset,
      limit,
      hasMore: offset + limit < sortedIds.length,
    };

    trackContractInteraction({
      method: "fetch_product_events_page",
      durationMs: Date.now() - startedAt,
      success: true,
      context: {
        productId,
        offset,
        limit,
        resultCount: validEvents.length,
        totalCount: sortedIds.length,
      },
    });

    return page;
  } catch (error) {
    console.error("Failed to fetch paged product events:", error);
    trackContractInteraction({
      method: "fetch_product_events_page",
      durationMs: Date.now() - startedAt,
      success: false,
      errorMessage: error instanceof Error ? error.message : String(error),
      context: { productId, offset, limit },
    });
    trackError(error, { source: "fetchProductEventsPage", productId, offset, limit });
    throw error;
  }
}

export async function fetchProductEvents(
  productId: string
): Promise<TimelineEvent[]> {
  const allEvents: TimelineEvent[] = [];
  let offset = 0;
  const limit = MAX_EVENTS_PAGE_SIZE;

  while (true) {
    const page = await fetchProductEventsPage(productId, { offset, limit });
    allEvents.push(...page.events);
    if (!page.hasMore) {
      return allEvents.sort((a, b) => b.timestamp - a.timestamp);
    }
    offset += limit;
  }
}

/**
 * Formats a timestamp (Unix epoch in seconds) to a readable date string.
 */
export function formatEventTimestamp(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  return new Intl.DateTimeFormat("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(date);
}

/**
 * Gets the relative time string (e.g., "2 hours ago").
 */
export function getRelativeTime(timestamp: number): string {
  const now = Math.floor(Date.now() / 1000);
  const diff = now - timestamp;

  if (diff < 60) return "just now";
  if (diff < 3600) return `${Math.floor(diff / 60)} minutes ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)} hours ago`;
  if (diff < 604800) return `${Math.floor(diff / 86400)} days ago`;
  
  return formatEventTimestamp(timestamp);
}
