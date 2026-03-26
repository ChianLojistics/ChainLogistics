import { describe, expect, it } from "vitest";

import {
  formatFirstErrorMessage,
  productIdSchema,
  stellarPublicKeySchema,
  requiredString,
  eventTypeSchema,
  eventTimestampSchema,
  eventTrackingSchema,
  ALLOWED_EVENT_TYPES,
  EVENT_NOTE_MAX_LEN,
  VALIDATION_MESSAGES,
} from "@/lib/validation";

describe("validation schemas", () => {
  it("validates product ID format", () => {
    expect(productIdSchema.safeParse("SKU-123_ABC").success).toBe(true);
    expect(productIdSchema.safeParse("bad id").success).toBe(false);
    expect(productIdSchema.safeParse("*").success).toBe(false);
  });

  it("validates required strings", () => {
    const schema = requiredString("Name");
    expect(schema.safeParse("Coffee").success).toBe(true);
    expect(schema.safeParse("").success).toBe(false);
  });

  it("validates Stellar public key", () => {
    const valid = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF";
    expect(stellarPublicKeySchema.safeParse(valid).success).toBe(true);

    expect(stellarPublicKeySchema.safeParse("not-a-key").success).toBe(false);
    expect(stellarPublicKeySchema.safeParse("SB".padEnd(56, "A")).success).toBe(false);
  });

  it("formats first available error message", () => {
    expect(formatFirstErrorMessage([undefined, "", "Bad value", "Other"])).toBe(
      "Bad value"
    );
    expect(formatFirstErrorMessage([undefined, null, "  "])).toBe("Invalid value");
  });

  describe("event type validation", () => {
    it("accepts all allowed event types", () => {
      for (const eventType of ALLOWED_EVENT_TYPES) {
        expect(eventTypeSchema.safeParse(eventType).success).toBe(true);
      }
    });

    it("rejects invalid event types", () => {
      const result = eventTypeSchema.safeParse("INVALID_TYPE");
      expect(result.success).toBe(false);
      if (!result.success) {
        expect(result.error.issues[0].message).toBe(VALIDATION_MESSAGES.eventTypeInvalid);
      }
    });

    it("rejects empty event type", () => {
      expect(eventTypeSchema.safeParse("").success).toBe(false);
    });

    it("rejects lowercase event types", () => {
      expect(eventTypeSchema.safeParse("harvest").success).toBe(false);
    });
  });

  describe("event timestamp validation", () => {
    it("accepts a timestamp in the past", () => {
      const pastTimestamp = Date.now() - 60000;
      expect(eventTimestampSchema.safeParse(pastTimestamp).success).toBe(true);
    });

    it("accepts the current timestamp", () => {
      expect(eventTimestampSchema.safeParse(Date.now()).success).toBe(true);
    });

    it("rejects a timestamp in the future", () => {
      const futureTimestamp = Date.now() + 60000;
      const result = eventTimestampSchema.safeParse(futureTimestamp);
      expect(result.success).toBe(false);
      if (!result.success) {
        expect(result.error.issues[0].message).toBe(VALIDATION_MESSAGES.timestampFuture);
      }
    });
  });

  describe("event tracking schema", () => {
    const validInput = {
      productId: "SKU-123",
      eventType: "HARVEST",
      location: "Warehouse A",
      note: "Sample note",
      timestamp: Date.now() - 1000,
    };

    it("accepts valid event tracking data", () => {
      expect(eventTrackingSchema.safeParse(validInput).success).toBe(true);
    });

    it("accepts data without optional note and timestamp", () => {
      const { note, timestamp, ...required } = validInput;
      expect(eventTrackingSchema.safeParse(required).success).toBe(true);
    });

    it("rejects note exceeding max length", () => {
      const longNote = "a".repeat(EVENT_NOTE_MAX_LEN + 1);
      const result = eventTrackingSchema.safeParse({ ...validInput, note: longNote });
      expect(result.success).toBe(false);
      if (!result.success) {
        expect(result.error.issues[0].message).toBe(
          VALIDATION_MESSAGES.maxLength("Note", EVENT_NOTE_MAX_LEN)
        );
      }
    });

    it("accepts note at exactly max length", () => {
      const maxNote = "a".repeat(EVENT_NOTE_MAX_LEN);
      expect(eventTrackingSchema.safeParse({ ...validInput, note: maxNote }).success).toBe(true);
    });

    it("rejects invalid event type in full schema", () => {
      const result = eventTrackingSchema.safeParse({ ...validInput, eventType: "UNKNOWN" });
      expect(result.success).toBe(false);
    });

    it("rejects future timestamp in full schema", () => {
      const result = eventTrackingSchema.safeParse({
        ...validInput,
        timestamp: Date.now() + 100000,
      });
      expect(result.success).toBe(false);
    });

    it("rejects missing product ID", () => {
      const { productId, ...rest } = validInput;
      expect(eventTrackingSchema.safeParse(rest).success).toBe(false);
    });

    it("rejects missing location", () => {
      const { location, ...rest } = validInput;
      expect(eventTrackingSchema.safeParse(rest).success).toBe(false);
    });

    it("rejects empty location", () => {
      const result = eventTrackingSchema.safeParse({ ...validInput, location: "" });
      expect(result.success).toBe(false);
    });
  });
});
