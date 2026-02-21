import type { TimelineEvent } from "@/lib/types/tracking";
import { createContractClient } from "@/lib/stellar/contractClient";
import { CONTRACT_CONFIG, validateContractConfig } from "./config";

export async function fetchProductEvents(
  productId: string
): Promise<TimelineEvent[]> {
  try {
    validateContractConfig();

    const contractClient = createContractClient({
      contractId: CONTRACT_CONFIG.CONTRACT_ID,
      network: CONTRACT_CONFIG.NETWORK,
      rpcUrl: CONTRACT_CONFIG.RPC_URL,
    });

    const eventIds = await contractClient.get_product_event_ids(productId);

    if (eventIds.length === 0) {
      return [];
    }

    const events = await Promise.all(
      eventIds.map((id) => contractClient.get_event(id))
    );

    const validEvents = events.filter(
      (e): e is TimelineEvent => e !== null
    );

    return validEvents.sort((a, b) => b.timestamp - a.timestamp);
  } catch (error) {
    console.error("Failed to fetch product events:", error);
    throw error;
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
