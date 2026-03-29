import { describe, it, expect } from "vitest";
import { productConversion, eventConversion, timestampUtils, validationUtils } from "../data-conversion";
import type { ProductResponse } from "../../types/product";
import type { TrackingEventResponse, TrackingEvent } from "../../types/tracking";

describe("Data Conversion Utilities", () => {
  describe("timestampUtils", () => {
    it("should convert ISO to Unix timestamp", () => {
      const isoString = "2024-03-29T12:00:00Z";
      const unix = timestampUtils.isoToUnix(isoString);
      expect(unix).toBe(1711704000);
    });

    it("should convert Unix timestamp to ISO", () => {
      const unix = 1711704000;
      const iso = timestampUtils.unixToIso(unix);
      expect(iso).toBe("2024-03-29T12:00:00.000Z");
    });

    it("should convert Unix timestamp to Date", () => {
      const unix = 1711704000;
      const date = timestampUtils.unixToDate(unix);
      expect(date.toISOString()).toBe("2024-03-29T12:00:00.000Z");
    });

    it("should return current Unix timestamp", () => {
      const now = timestampUtils.now();
      expect(now).toBeGreaterThan(1711704000); // After March 2024
      expect(now).toBeLessThan(Date.now() / 1000 + 1); // Within 1 second
    });
  });

  describe("productConversion", () => {
    const mockBackendProduct: ProductResponse = {
      id: "PROD-123",
      name: "Organic Coffee",
      description: "Premium organic coffee beans",
      origin_location: "Ethiopia",
      category: "Beverages",
      tags: ["organic", "fair-trade"],
      certifications: ["cert123", "cert456"],
      media_hashes: ["hash789"],
      custom_fields: { region: "Yirgacheffe" },
      owner_address: "GABC1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ",
      is_active: true,
      created_at: "2024-03-29T12:00:00Z",
      updated_at: "2024-03-29T13:00:00Z",
      eventCount: 5,
      created_by: "user123",
      updated_by: "user456",
    };

    it("should convert backend product to frontend format", () => {
      const frontend = productConversion.fromBackend(mockBackendProduct);
      
      expect(frontend.id).toBe("PROD-123");
      expect(frontend.name).toBe("Organic Coffee");
      expect(frontend.origin.location).toBe("Ethiopia");
      expect(frontend.owner).toBe(mockBackendProduct.owner_address);
      expect(frontend.created_at).toBe(1711704000);
      expect(frontend.active).toBe(true);
      expect(frontend.certifications).toEqual(["cert123", "cert456"]);
      expect(frontend.custom_fields).toEqual({ region: "Yirgacheffe" });
    });

    it("should convert contract product to frontend format", () => {
      const contractProduct = {
        id: "PROD-456",
        name: "Specialty Tea",
        description: "Premium tea leaves",
        origin: { location: "India" },
        owner: "GDEF1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        created_at: 1711707600,
        active: true,
        category: "Beverages",
        tags: ["premium"],
        certifications: ["cert789"],
        media_hashes: ["hash123"],
        custom: { grade: "A+" },
      };

      const frontend = productConversion.fromContract(contractProduct);
      
      expect(frontend.id).toBe("PROD-456");
      expect(frontend.origin.location).toBe("India");
      expect(frontend.created_at).toBe(1711707600);
      expect(frontend.custom_fields).toEqual({ grade: "A+" });
      expect(frontend.updated_at).toBeUndefined();
      expect(frontend.created_by).toBeUndefined();
    });

    it("should convert frontend product to backend format", () => {
      const frontendProduct = {
        name: "New Product",
        description: "Test product",
        origin: { location: "Colombia" },
        category: "Test",
        tags: ["test"],
        certifications: ["cert111"],
        media_hashes: ["hash222"],
        custom_fields: { test: "value" },
        owner: "GHIJ1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        active: true,
      };

      const backend = productConversion.toBackend(frontendProduct);
      
      expect(backend.name).toBe("New Product");
      expect(backend.origin_location).toBe("Colombia");
      expect(backend.owner_address).toBe(frontendProduct.owner);
      expect(backend.custom_fields).toEqual({ test: "value" });
    });
  });

  describe("eventConversion", () => {
    const mockBackendEvent: TrackingEventResponse = {
      id: 123,
      product_id: "PROD-123",
      actor_address: "GABC1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ",
      timestamp: "2024-03-29T12:30:00Z",
      event_type: "SHIPPING",
      location: "Port of Seattle",
      data_hash: "abc123def456789012345678901234567890123456789012345678901234567890",
      note: "Shipped via cargo",
      metadata: { temperature: 20, humidity: 45 },
      created_at: "2024-03-29T12:30:00Z",
    };

    it("should convert backend event to frontend format", () => {
      const frontend = eventConversion.fromBackend(mockBackendEvent);
      
      expect(frontend.event_id).toBe(123);
      expect(frontend.product_id).toBe("PROD-123");
      expect(frontend.actor).toBe(mockBackendEvent.actor_address);
      expect(frontend.timestamp).toBe(1711705800);
      expect(frontend.event_type).toBe("SHIPPING");
      expect(frontend.location).toBe("Port of Seattle");
      expect(frontend.data_hash).toBe(mockBackendEvent.data_hash);
      expect(frontend.metadata).toEqual({ temperature: 20, humidity: 45 });
    });

    it("should convert contract event to frontend format", () => {
      const contractEvent = {
        event_id: 456,
        product_id: "PROD-456",
        actor: "GDEF1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        timestamp: 1711709400,
        event_type: "PROCESSING",
        location: "Processing Plant",
        data_hash: "def123abc456789012345678901234567890123456789012345678901234567890",
        note: "Quality check passed",
        metadata: { quality: "A" },
      };

      const frontend = eventConversion.fromContract(contractEvent);
      
      expect(frontend.event_id).toBe(456);
      expect(frontend.timestamp).toBe(1711709400);
      expect(frontend.event_type).toBe("PROCESSING");
      expect(frontend.metadata).toEqual({ quality: "A" });
    });

    it("should convert frontend event to timeline format", () => {
      const frontendEvent = {
        event_id: 789,
        product_id: "PROD-789",
        actor: "GHIJ1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        timestamp: 1711713000,
        event_type: "RECEIVING" as const,
        location: "Warehouse",
        data_hash: "123abc456def789012345678901234567890123456789012345678901234567890",
        note: "Received and stored",
        metadata: { received_by: "John Doe" },
      };

      const timeline = eventConversion.toTimeline(frontendEvent);
      
      expect(timeline.event_id).toBe(789);
      expect(timeline.product_id).toBe("PROD-789");
      expect(timeline.timestamp).toBe(1711713000);
      expect(timeline.event_type).toBe("RECEIVING");
      expect(timeline.metadata).toEqual({ received_by: "John Doe" });
    });
  });

  describe("validationUtils", () => {
    it("should validate Stellar addresses", () => {
      const validAddress = "GABC1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ";
      const invalidAddress = "INVALID_ADDRESS";
      
      expect(validationUtils.isValidStellarAddress(validAddress)).toBe(true);
      expect(validationUtils.isValidStellarAddress(invalidAddress)).toBe(false);
    });

    it("should validate hash format", () => {
      const validHash = "abc123def456789012345678901234567890123456789012345678901234567890";
      const invalidHash = "invalid_hash";
      
      expect(validationUtils.isValidHash(validHash)).toBe(true);
      expect(validationUtils.isValidHash(invalidHash)).toBe(false);
    });

    it("should validate product ID format", () => {
      const validId = "PROD-123-456";
      const invalidId = "product@id#with$special%chars";
      
      expect(validationUtils.isValidProductId(validId)).toBe(true);
      expect(validationUtils.isValidProductId(invalidId)).toBe(false);
    });

    it("should validate timestamps", () => {
      const validTimestamp = 1711704000; // March 29, 2024
      const invalidTimestamp = -1;
      const futureTimestamp = timestampUtils.now() + 86500; // 1 day in future
      
      expect(validationUtils.isValidTimestamp(validTimestamp)).toBe(true);
      expect(validationUtils.isValidTimestamp(invalidTimestamp)).toBe(false);
      expect(validationUtils.isValidTimestamp(futureTimestamp)).toBe(true);
    });
  });
});
