import type { ProductId } from "./product";

export type TrackingEventType = "REGISTER" | "TRANSFER" | "CHECKPOINT";

export type TrackingEvent = {
  productId: ProductId;
  type: TrackingEventType;
  timestamp: number;
  metadata?: Record<string, unknown>;
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
