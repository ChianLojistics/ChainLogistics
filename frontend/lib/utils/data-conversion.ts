import { Product, ProductResponse, ProductContract } from "../types/product";
import { 
  TrackingEvent, 
  TrackingEventResponse, 
  TrackingEventContract, 
  TimelineEvent, 
  EventMetadata 
} from "../types/tracking";

// Timestamp conversion utilities
export const timestampUtils = {
  // Convert ISO datetime string to Unix timestamp (seconds)
  isoToUnix: (isoString: string): number => {
    return Math.floor(new Date(isoString).getTime() / 1000);
  },

  // Convert Unix timestamp (seconds) to ISO datetime string
  unixToIso: (unixTimestamp: number): string => {
    return new Date(unixTimestamp * 1000).toISOString();
  },

  // Convert Unix timestamp (seconds) to Date object
  unixToDate: (unixTimestamp: number): Date => {
    return new Date(unixTimestamp * 1000);
  },

  // Get current Unix timestamp (seconds)
  now: (): number => {
    return Math.floor(Date.now() / 1000);
  },
};

// Product conversion utilities
export const productConversion = {
  // Convert backend response to frontend Product
  fromBackend: (backend: ProductResponse): Product => {
    return {
      id: backend.id,
      name: backend.name,
      description: backend.description,
      origin: {
        location: backend.origin_location,
      },
      owner: backend.owner_address,
      created_at: timestampUtils.isoToUnix(backend.created_at),
      active: backend.is_active,
      category: backend.category,
      tags: backend.tags,
      certifications: backend.certifications,
      media_hashes: backend.media_hashes,
      custom_fields: backend.custom_fields as Record<string, string>,
      eventCount: backend.eventCount,
      updated_at: backend.updated_at ? timestampUtils.isoToUnix(backend.updated_at) : undefined,
      created_by: backend.created_by,
      updated_by: backend.updated_by,
    };
  },

  // Convert smart contract product to frontend Product
  fromContract: (contract: ProductContract): Product => {
    return {
      id: contract.id,
      name: contract.name,
      description: contract.description,
      origin: contract.origin,
      owner: contract.owner,
      created_at: contract.created_at,
      active: contract.active,
      category: contract.category,
      tags: contract.tags,
      certifications: contract.certifications,
      media_hashes: contract.media_hashes,
      custom_fields: contract.custom,
      updated_at: undefined, // Not available in contract
      created_by: undefined, // Not available in contract
      updated_by: undefined, // Not available in contract
    };
  },

  // Convert frontend Product to backend format (for API requests)
  toBackend: (product: Omit<Product, "id" | "created_at" | "eventCount" | "updated_at" | "created_by" | "updated_by">) => {
    return {
      name: product.name,
      description: product.description,
      origin_location: product.origin.location,
      category: product.category,
      tags: product.tags,
      certifications: product.certifications,
      media_hashes: product.media_hashes,
      custom_fields: product.custom_fields || {},
      owner_address: product.owner,
    };
  },
};

// Tracking event conversion utilities
export const eventConversion = {
  // Convert backend response to frontend TrackingEvent
  fromBackend: (backend: TrackingEventResponse): TrackingEvent => {
    return {
      event_id: backend.id,
      product_id: backend.product_id,
      actor: backend.actor_address,
      timestamp: timestampUtils.isoToUnix(backend.timestamp),
      event_type: backend.event_type as any,
      location: backend.location,
      data_hash: backend.data_hash,
      note: backend.note,
      metadata: backend.metadata as any,
    };
  },

  // Convert smart contract event to frontend TrackingEvent
  fromContract: (contract: TrackingEventContract): TrackingEvent => {
    return {
      event_id: contract.event_id,
      product_id: contract.product_id,
      actor: contract.actor,
      timestamp: contract.timestamp,
      event_type: contract.event_type as any,
      location: contract.location,
      data_hash: contract.data_hash,
      note: contract.note,
      metadata: contract.metadata as any,
    };
  },

  // Convert frontend TrackingEvent to TimelineEvent (for UI)
  toTimeline: (event: TrackingEvent): TimelineEvent => {
    return {
      event_id: event.event_id,
      product_id: event.product_id,
      actor: event.actor,
      timestamp: event.timestamp,
      event_type: event.event_type,
      location: event.location,
      note: event.note,
      data_hash: event.data_hash,
      metadata: event.metadata,
    };
  },

  // Convert frontend TrackingEvent to backend format (for API requests)
  toBackend: (event: Omit<TrackingEvent, "event_id">) => {
    return {
      product_id: event.product_id,
      actor_address: event.actor,
      timestamp: timestampUtils.unixToIso(event.timestamp),
      event_type: event.event_type,
      location: event.location,
      data_hash: event.data_hash,
      note: event.note,
      metadata: event.metadata || {},
    };
  },
};

// Validation utilities
export const validationUtils = {
  // Validate Stellar address format
  isValidStellarAddress: (address: string): boolean => {
    // Basic validation for Stellar address (G-prefixed, 56 characters)
    return /^G[A-Z0-9]{55}$/.test(address);
  },

  // Validate hash format (SHA-256 hex)
  isValidHash: (hash: string): boolean => {
    return /^[a-fA-F0-9]{64}$/.test(hash);
  },

  // Validate product ID format
  isValidProductId: (id: string): boolean => {
    return /^[a-zA-Z0-9_-]{1,50}$/.test(id);
  },

  // Validate timestamp format
  isValidTimestamp: (timestamp: number): boolean => {
    return timestamp > 0 && timestamp < timestampUtils.now() + 86400; // Allow 1 day in future
  },
};
