export type ProductId = string;

/**
 * Normalizes unix timestamps from either seconds or milliseconds into seconds.
 */
export function normalizeUnixSeconds(value: number): number {
  return value > 1_000_000_000_000 ? Math.floor(value / 1000) : value;
}

export type Product = {
  id: ProductId;
  name: string;
  description: string;
  origin: {
    location: string;
  };
  origin_location?: string;
  owner: string; // Address as string
  owner_address?: string;
  created_at: number; // Unix timestamp (seconds preferred, ms tolerated)
  createdAt?: number;
  active: boolean;
  is_active?: boolean;
  isActive?: boolean;
  category: string;
  tags: string[];
  note?: string;
  eventCount?: number; // Client-side computed field
};
