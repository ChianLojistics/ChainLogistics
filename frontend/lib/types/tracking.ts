import type { ProductId } from "./product";
import { normalizeUnixSeconds } from "./product";

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
  eventId?: number;
  product_id: string;
  productId?: string;
  actor: string;
  timestamp: number;
  event_type: string;
  eventType?: string;
  note: string;
  notes?: string;
  data_hash?: string;
  dataHash?: string;
};

export function normalizeTimelineEvent(event: TimelineEvent): TimelineEvent {
  return {
    event_id: event.event_id ?? event.eventId ?? 0,
    product_id: event.product_id ?? event.productId ?? "",
    actor: event.actor,
    timestamp: normalizeUnixSeconds(event.timestamp),
    event_type: event.event_type ?? event.eventType ?? "UNKNOWN",
    note: event.note ?? event.notes ?? "",
    data_hash: event.data_hash ?? event.dataHash,
  };
}

export type EventCardProps = {
  event: TimelineEvent;
  isFirst: boolean;
  isLast: boolean;
};
