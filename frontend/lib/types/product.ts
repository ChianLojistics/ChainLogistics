export type ProductId = string;

export type Product = {
  id: ProductId;
  name: string;
  description: string;
  origin: {
    location: string;
  };
  owner: string; // Address as string
  createdAt: number; // Unix timestamp in seconds (matching smart contract u64)
  active: boolean;
  category: string;
  tags: string[];
  eventCount?: number; // Client-side computed field
};
