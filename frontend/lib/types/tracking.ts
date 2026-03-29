import type { ProductId } from "./product";

export type TrackingEventType = "REGISTER" | "TRANSFER" | "CHECKPOINT" | "HARVEST" | "PROCESSING" | "PACKAGING" | "SHIPPING" | "RECEIVING" | "QUALITY_CHECK";

/** Structured metadata attached to a tracking event. */
export type EventMetadata = {
  location?: string;
  temperature?: number;
  humidity?: number;
  notes?: string;
  [key: string]: string | number | boolean | undefined;
};

// Core tracking event structure (unified)
export type TrackingEvent = {
  event_id: number;
  product_id: ProductId;
  actor: string; // Address as string
  timestamp: number; // Unix timestamp (seconds)
  event_type: TrackingEventType;
  location: string;
  data_hash: string;
  note: string;
  metadata?: EventMetadata;
};

// Backend-specific structure for API responses
export type TrackingEventResponse = {
  id: number;
  product_id: string;
  actor_address: string;
  timestamp: string; // ISO datetime from backend
  event_type: string;
  location: string;
  data_hash: string;
  note: string;
  metadata: Record<string, any>;
  created_at: string; // ISO datetime from backend
};

// Smart contract specific structure
export type TrackingEventContract = {
  event_id: number;
  product_id: string;
  actor: string; // Address
  timestamp: number; // Unix timestamp (seconds)
  event_type: string; // Symbol from contract
  location: string;
  data_hash: string; // BytesN<32> as string
  note: string;
  metadata: Record<string, string>; // Map<Symbol, String>
};

// Timeline event for UI display (legacy support)
export type TimelineEvent = {
  event_id: number;
  product_id: string;
  actor: string;
  timestamp: number;
  event_type: string;
  location: string;
  note: string;
  data_hash?: string;
  metadata?: EventMetadata;
};

export type EventCardProps = {
  event: TimelineEvent;
  isFirst: boolean;
  isLast: boolean;
};

// Event creation payload
export type CreateTrackingEventRequest = {
  product_id: ProductId;
  event_type: TrackingEventType;
  location: string;
  data_hash?: string;
  note?: string;
  metadata?: EventMetadata;
};

// Event filtering options
export type EventFilter = {
  event_type?: TrackingEventType;
  start_time?: number;
  end_time?: number;
  location?: string;
  actor?: string;
};
