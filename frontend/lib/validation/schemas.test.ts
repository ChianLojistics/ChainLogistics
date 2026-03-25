import { describe, expect, it } from "vitest";

import {
  formatFirstErrorMessage,
  productIdSchema,
  stellarPublicKeySchema,
  requiredString,
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
});
