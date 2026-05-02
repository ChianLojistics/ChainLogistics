import type { Product } from "@/lib/types/product";
import { CONTRACT_CONFIG } from "./config";
import { trackContractInteraction } from "@/lib/analytics";

/**
 * Fetches products by owner address.
 *
 * The on-chain contract does not currently expose a "list products by owner"
 * query, so this function returns mock data when a contract ID is not
 * configured (development) and an empty array otherwise.
 * Replace the body with a real indexer / contract call once one is available.
 *
 * @param owner - The owner's public key/address
 * @returns Array of products owned by the given address
 */
export async function getProductsByOwner(owner: string): Promise<Product[]> {
  const startedAt = Date.now();

  const useMockData = process.env.NEXT_PUBLIC_USE_MOCK_DATA === "true";

  // When no contract is configured, return mock data only when explicitly enabled.
  if (!CONTRACT_CONFIG.CONTRACT_ID) {
    if (useMockData) {
      return getMockProducts(owner);
    }
    return [];
  }

  // Placeholder: until the contract/indexer exposes a "list by owner" method,
  // return an empty array and track the call for observability.
  trackContractInteraction({
    method: "get_products_by_owner",
    durationMs: Date.now() - startedAt,
    success: true,
    context: { owner, resultCount: 0, stub: true },
  });

  return [];
}

/** Mock products used during local development when no contract is configured. */
function getMockProducts(owner: string): Product[] {
  const now = Date.now();
  const dayMs = 24 * 60 * 60 * 1000;
  
  return [
    {
      id: "PROD-001",
      name: "Organic Coffee Beans",
      description: "Premium organic coffee beans from Colombia",
      origin: { location: "Colombia, South America" },
      owner,
      created_at: Math.floor((now - 30 * dayMs) / 1000),
      active: true,
      category: "Beverages",
      tags: ["organic", "fair-trade"],
      eventCount: 5,
    },
    {
      id: "PROD-002",
      name: "Fresh Avocados",
      description: "Hass avocados from Mexico",
      origin: { location: "Michoacán, Mexico" },
      owner,
      created_at: Math.floor((now - 15 * dayMs) / 1000),
      active: true,
      category: "Produce",
      tags: ["organic", "fresh"],
      eventCount: 3,
    },
    {
      id: "PROD-003",
      name: "Organic Cotton T-Shirt",
      description: "100% organic cotton t-shirt",
      origin: { location: "India" },
      owner,
      created_at: Math.floor((now - 7 * dayMs) / 1000),
      active: true,
      category: "Apparel",
      tags: ["organic", "sustainable"],
      eventCount: 8,
    },
    {
      id: "PROD-004",
      name: "Honey",
      description: "Raw wildflower honey",
      origin: { location: "Vermont, USA" },
      owner,
      created_at: Math.floor((now - 45 * dayMs) / 1000),
      active: false,
      category: "Food",
      tags: ["raw", "local"],
      eventCount: 12,
    },
  ];
}
