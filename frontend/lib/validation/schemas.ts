import { StrKey } from "@stellar/stellar-sdk";
import { z } from "zod";

import { VALIDATION_MESSAGES } from "./messages";

export const PRODUCT_ID_MIN_LEN = 1;
export const PRODUCT_ID_MAX_LEN = 64;

export const productIdSchema = z
  .string()
  .min(PRODUCT_ID_MIN_LEN, VALIDATION_MESSAGES.productIdLength(PRODUCT_ID_MIN_LEN, PRODUCT_ID_MAX_LEN))
  .max(PRODUCT_ID_MAX_LEN, VALIDATION_MESSAGES.productIdLength(PRODUCT_ID_MIN_LEN, PRODUCT_ID_MAX_LEN))
  .regex(/^[a-zA-Z0-9-_]+$/, VALIDATION_MESSAGES.productIdInvalid);

export const stellarPublicKeySchema = z
  .string()
  .min(1, VALIDATION_MESSAGES.required("Address"))
  .refine((value) => StrKey.isValidEd25519PublicKey(value), {
    message: VALIDATION_MESSAGES.stellarAddressInvalid,
  });

export function requiredString(fieldLabel: string) {
  return z.string().min(1, VALIDATION_MESSAGES.required(fieldLabel));
}

export function optionalStringMax(maxLen: number) {
  return z.string().max(maxLen).optional();
}

export function withCustomRule<T>(schema: z.ZodType<T>, predicate: (value: T) => boolean, message: string) {
  return schema.refine(predicate, { message });
}

export const LOCATION_REGEX = /^[\w\s,.\-']+$/;

export const productRegistrationSchema = z.object({
  id: productIdSchema,
  name: requiredString("Name").max(128, VALIDATION_MESSAGES.maxLength("Name", 128)),
  origin: requiredString("Origin")
    .max(256, VALIDATION_MESSAGES.maxLength("Origin", 256))
    .regex(LOCATION_REGEX, "Origin contains invalid characters"),
  description: z.string().max(2048, VALIDATION_MESSAGES.maxLength("Description", 2048)).optional(),
  category: requiredString("Category").max(64, VALIDATION_MESSAGES.maxLength("Category", 64)),
});

export type ProductRegistrationValues = z.infer<typeof productRegistrationSchema>;

export const ALLOWED_EVENT_TYPES = [
  "HARVEST",
  "PROCESS",
  "PACKAGE",
  "SHIP",
  "RECEIVE",
  "QUALITY_CHECK",
  "TRANSFER",
  "REGISTER",
  "CHECKPOINT",
] as const;

export type AllowedEventType = (typeof ALLOWED_EVENT_TYPES)[number];

export const EVENT_NOTE_MAX_LEN = 256;

export const eventTypeSchema = z
  .string()
  .min(1, VALIDATION_MESSAGES.required("Event type"))
  .refine((value): value is AllowedEventType => ALLOWED_EVENT_TYPES.includes(value as AllowedEventType), {
    message: VALIDATION_MESSAGES.eventTypeInvalid,
  });

export const eventTimestampSchema = z
  .number()
  .refine((value) => value <= Date.now(), {
    message: VALIDATION_MESSAGES.timestampFuture,
  });

export const eventTrackingSchema = z.object({
  productId: productIdSchema,
  eventType: eventTypeSchema,
  location: requiredString("Location")
    .max(256, VALIDATION_MESSAGES.maxLength("Location", 256))
    .regex(LOCATION_REGEX, "Location contains invalid characters"),
  note: z.string().max(EVENT_NOTE_MAX_LEN, VALIDATION_MESSAGES.maxLength("Note", EVENT_NOTE_MAX_LEN)).optional(),
  timestamp: eventTimestampSchema.optional(),
});

export type EventTrackingValues = z.infer<typeof eventTrackingSchema>;

// ─── Transfer Product Schema ──────────────────────────────────────────────────

export const transferProductSchema = z.object({
  productId: productIdSchema,
  recipientAddress: stellarPublicKeySchema,
});

export type TransferProductValues = z.infer<typeof transferProductSchema>;

// ─── Product Search Schema ────────────────────────────────────────────────────

export const PRODUCT_SEARCH_MIN_LEN = 1;
export const PRODUCT_SEARCH_MAX_LEN = 128;

export const productSearchSchema = z.object({
  query: z
    .string()
    .min(PRODUCT_SEARCH_MIN_LEN, VALIDATION_MESSAGES.required("Search query"))
    .max(
      PRODUCT_SEARCH_MAX_LEN,
      VALIDATION_MESSAGES.maxLength("Search query", PRODUCT_SEARCH_MAX_LEN)
    )
    .regex(
      /^[a-zA-Z0-9 \-_.,]+$/,
      "Search query contains invalid characters"
    ),
});

export type ProductSearchValues = z.infer<typeof productSearchSchema>;
