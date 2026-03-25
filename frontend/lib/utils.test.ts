import { describe, it, expect } from "vitest";
import { cn } from "./utils";

describe("Utility Functions", () => {
  describe("cn (className utility)", () => {
    it("should merge class names correctly", () => {
      expect(cn("foo", "bar")).toBe("foo bar");
    });

    it("should handle conditional classes", () => {
      expect(cn("foo", true && "bar", false && "baz")).toBe("foo bar");
    });

    it("should handle Tailwind class conflicts", () => {
      expect(cn("px-2", "px-4")).toBe("px-4");
      expect(cn("text-red-500", "text-blue-500")).toBe("text-blue-500");
    });

    it("should handle arrays and objects", () => {
      expect(cn(["foo", "bar"], { baz: true, qux: false })).toBe("foo bar baz");
    });

    it("should handle undefined and null values", () => {
      expect(cn("foo", undefined, null, "bar")).toBe("foo bar");
    });

    it("should handle empty inputs", () => {
      expect(cn()).toBe("");
      expect(cn("", "  ")).toBe("");
    });

    it("should preserve important modifiers", () => {
      expect(cn("px-2", "px-4!")).toBe("px-4!");
    });
  });
});
