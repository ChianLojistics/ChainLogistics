import type { ProductId } from "./product";

export type TrackingEventType = "REGISTER" | "TRANSFER" | "CHECKPOINT";

export type TrackingEvent = {
  productId: ProductId;
  type: TrackingEventType;
  timestamp: number;
  metadata?: Record<string, unknown>;
};
