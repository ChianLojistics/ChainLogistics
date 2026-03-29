export type ProductId = string;

export type Product = {
  id: ProductId;
  name: string;
  description: string;
  origin: {
    location: string;
  };
  owner: string; // Address as string
  created_at: number; // Unix timestamp (seconds)
  active: boolean;
  category: string;
  tags: string[];
  certifications: string[]; // Hash strings
  media_hashes: string[]; // Hash strings
  custom_fields?: Record<string, string>; // Custom key-value pairs
  eventCount?: number; // Client-side computed field
  updated_at?: number; // Unix timestamp (seconds)
  created_by?: string; // User who created the product
  updated_by?: string; // User who last updated the product
};

// Backend-specific fields for API responses
export type ProductResponse = Product & {
  is_active: boolean;
  owner_address: string;
  origin_location: string;
  custom_fields: Record<string, any>;
  created_at: string; // ISO datetime from backend
  updated_at: string; // ISO datetime from backend
};

// Smart contract specific fields
export type ProductContract = {
  id: string;
  name: string;
  description: string;
  origin: { location: string };
  owner: string;
  created_at: number;
  active: boolean;
  category: string;
  tags: string[];
  certifications: string[];
  media_hashes: string[];
  custom: Record<string, string>;
  deactivation_info?: Array<{
    reason: string;
    deactivated_at: number;
    deactivated_by: string;
  }>;
};
