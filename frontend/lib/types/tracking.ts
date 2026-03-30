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
  timestamp: number;
  metadata?: EventMetadata;
};

export type TimelineEvent = {
  event_id: number;
  product_id: string;
  actor: string;
  timestamp: number;
  event_type: string;
  note: string;
  data_hash?: string;
};

export type EventCardProps = {
  event: TimelineEvent;
  isFirst: boolean;
  isLast: boolean;
};
