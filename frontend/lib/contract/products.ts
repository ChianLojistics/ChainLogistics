import type { Product } from "@/lib/types/product";

/**
 * Fetches products by owner address.
 * 
 * Note: Since blockchain doesn't support complex queries, this function
 * should ideally use an indexer service. For now, this is a placeholder
 * that can be replaced with actual contract calls or indexer API calls.
 * 
 * @param owner - The owner's public key/address
 * @returns Array of products owned by the given address
 */
export async function getProductsByOwner(owner: string): Promise<Product[]> {
  // TODO: Replace with actual contract call or indexer query
  // For now, return empty array as placeholder
  // 
  // Example implementation with Soroban SDK:
  // const contract = new Contract(CONTRACT_ID);
  // const result = await contract.getProductsByOwner({ owner });
  // return result.map(transformProduct);
  
  // Mock data for development - remove in production
  if (process.env.NODE_ENV === "development") {
    return getMockProducts(owner);
  }
  
  return [];
}

/**
 * Mock products for development/testing.
 * Remove this function when connecting to real contract/indexer.
 */
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
