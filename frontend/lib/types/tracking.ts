import type { ProductId } from "./product";

export type TrackingEventType = "REGISTER" | "TRANSFER" | "CHECKPOINT";

/** Structured metadata attached to a tracking event. */
export type EventMetadata = {
  location?: string;
  temperature?: number;
  humidity?: number;
  notes?: string;
  [key: string]: string | number | boolean | undefined;
};

export type TrackingEvent = {
  productId: ProductId;
  type: TrackingEventType;
  timestamp: number; // Unix timestamp in seconds (matching smart contract u64)
  location?: string;
  dataHash?: string;
  note?: string; // Added to match backend and smart contract
  metadata?: EventMetadata;
};

export type TimelineEvent = {
  eventId: number;
  productId: string;
  actor: string;
  timestamp: number; // Unix timestamp in seconds
  eventType: string;
  note: string;
  dataHash?: string;
};

export type EventCardProps = {
  event: TimelineEvent;
  isFirst: boolean;
  isLast: boolean;
};
