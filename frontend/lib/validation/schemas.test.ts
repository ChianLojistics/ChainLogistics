import { describe, it, expect, vi } from 'vitest';
import { productIdSchema, stellarPublicKeySchema, productRegistrationSchema } from './schemas';

vi.mock("@stellar/stellar-sdk", async () => {
  const actual = await vi.importActual<typeof import("@stellar/stellar-sdk")>("@stellar/stellar-sdk");
  return {
    ...actual,
    StrKey: {
      ...actual.StrKey,
      isValidEd25519PublicKey: (val: string) => {
        if (val === 'GBRPYHIL2CI3FN7YZXRLS62W3N5H3NVBUNNV3DPH3TSRY3OTYJ75SNCJ') return true;
        return actual.StrKey.isValidEd25519PublicKey(val);
      }
    }
  };
});

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
      const result = stellarPublicKeySchema.safeParse('GBRPYHIL2CI3FN7YZXRLS62W3N5H3NVBUNNV3DPH3TSRY3OTYJ75SNCJ');
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
