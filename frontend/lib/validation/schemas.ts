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

export const productRegistrationSchema = z.object({
  id: productIdSchema,
  name: requiredString("Name").max(128, VALIDATION_MESSAGES.maxLength("Name", 128)),
  origin: requiredString("Origin").max(256, VALIDATION_MESSAGES.maxLength("Origin", 256)),
  description: z.string().max(2048, VALIDATION_MESSAGES.maxLength("Description", 2048)).optional(),
  category: requiredString("Category").max(64, VALIDATION_MESSAGES.maxLength("Category", 64)),
});

export type ProductRegistrationValues = z.infer<typeof productRegistrationSchema>;

export const eventTrackingSchema = z.object({
  productId: productIdSchema,
  eventType: requiredString("Event type"),
  location: requiredString("Location"),
  note: z.string().max(512, VALIDATION_MESSAGES.maxLength("Note", 512)).optional(),
});

export type EventTrackingValues = z.infer<typeof eventTrackingSchema>;
