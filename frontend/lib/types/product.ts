export type ProductId = string;

export type Product = {
  id: ProductId;
  name: string;
  description: string;
  origin: {
    location: string;
  };
  owner: string; // Address as string
  created_at: number; // Unix timestamp
  active: boolean;
  category: string;
  tags: string[];
  eventCount?: number; // Client-side computed field
};
