import { describe, it, expect } from 'vitest';
import { productIdSchema, stellarPublicKeySchema, productRegistrationSchema } from './schemas';

describe('Validation Schemas', () => {
  describe('productIdSchema', () => {
    it('should validate a correct product ID', () => {
      const result = productIdSchema.safeParse('valid-product-id_123');
      expect(result.success).toBe(true);
    });

    it('should reject a product ID that is too short', () => {
      const result = productIdSchema.safeParse('');
      expect(result.success).toBe(false);
    });

    it('should reject a product ID with invalid characters', () => {
      const result = productIdSchema.safeParse('invalid product id!');
      expect(result.success).toBe(false);
    });
  });

  describe('stellarPublicKeySchema', () => {
    it('should validate a correct Stellar public key', () => {
      // Valid Stellar public key
      const result = stellarPublicKeySchema.safeParse('GAQCE2UKHWWEDZ37V7434OT6FOWV6W5XQ4J4S5E4LFR6XVXNXZ4C2P5X');
      expect(result.success).toBe(true);
    });

    it('should reject an invalid Stellar public key', () => {
      const result = stellarPublicKeySchema.safeParse('invalid-key');
      expect(result.success).toBe(false);
    });
  });

  describe('productRegistrationSchema', () => {
    it('should validate correct registration values', () => {
      const result = productRegistrationSchema.safeParse({
        id: 'prod-123',
        name: 'Test Product',
        origin: 'Farm',
        category: 'Food'
      });
      expect(result.success).toBe(true);
    });

    it('should require mandatory fields', () => {
      const result = productRegistrationSchema.safeParse({
        id: 'prod-123'
      });
      expect(result.success).toBe(false);
    });
  });
});
