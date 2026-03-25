import { describe, it, expect, vi, beforeEach } from "vitest";
import { generateProductQR, generateProductQRSVG, getVerificationUrl } from "../qr";

// Mock QRCode library
vi.mock('qrcode', () => ({
  default: {
    toDataURL: vi.fn(),
    toString: vi.fn(),
  },
}));

const mockQRCode = await import('qrcode');

describe("QR Code Utilities", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Reset environment
    delete process.env.NEXT_PUBLIC_APP_URL;
    // Mock window.location for SSR tests
    Object.defineProperty(window, 'location', {
      value: { origin: 'http://localhost:3000' },
      writable: true,
    });
  });

  describe("getVerificationUrl", () => {
    it("should generate verification URL with default localhost", () => {
      const productId = "prod-123";
      const result = getVerificationUrl(productId);
      expect(result).toBe("http://localhost:3000/verify/prod-123");
    });

    it("should use NEXT_PUBLIC_APP_URL when available", () => {
      process.env.NEXT_PUBLIC_APP_URL = "https://chainlogistics.app";
      const productId = "prod-456";
      const result = getVerificationUrl(productId);
      expect(result).toBe("https://chainlogistics.app/verify/prod-456");
    });

    it("should handle window.location.origin in browser environment", () => {
      delete process.env.NEXT_PUBLIC_APP_URL;
      Object.defineProperty(window, 'location', {
        value: { origin: 'https://staging.chainlogistics.app' },
        writable: true,
      });
      const productId = "prod-789";
      const result = getVerificationUrl(productId);
      expect(result).toBe("https://staging.chainlogistics.app/verify/prod-789");
    });
  });

  describe("generateProductQR", () => {
    it("should generate QR code data URL", async () => {
      const mockDataURL = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA";
      mockQRCode.default.toDataURL.mockResolvedValue(mockDataURL);

      const productId = "prod-123";
      const result = await generateProductQR(productId);

      expect(mockQRCode.default.toDataURL).toHaveBeenCalledWith(
        "http://localhost:3000/verify/prod-123",
        {
          width: 300,
          margin: 2,
          color: {
            dark: '#000000',
            light: '#FFFFFF'
          }
        }
      );
      expect(result).toBe(mockDataURL);
    });

    it("should handle QRCode generation errors", async () => {
      const error = new Error("QR generation failed");
      vi.mocked(mockQRCode.default.toDataURL).mockRejectedValue(error);

      const productId = "prod-123";
      await expect(generateProductQR(productId)).rejects.toThrow("QR generation failed");
    });
  });

  describe("generateProductQRSVG", () => {
    it("should generate QR code SVG string", async () => {
      const mockSVG = '<svg xmlns="http://www.w3.org/2000/svg"><!-- QR Code --></svg>';
      mockQRCode.default.toString.mockResolvedValue(mockSVG);

      const productId = "prod-456";
      const result = await generateProductQRSVG(productId);

      expect(mockQRCode.default.toString).toHaveBeenCalledWith(
        "http://localhost:3000/verify/prod-456",
        {
          type: 'svg',
          width: 300,
          margin: 2,
          color: {
            dark: '#000000',
            light: '#FFFFFF'
          }
        }
      );
      expect(result).toBe(mockSVG);
    });

    it("should handle SVG generation errors", async () => {
      const error = new Error("SVG generation failed");
      vi.mocked(mockQRCode.default.toString).mockRejectedValue(error);

      const productId = "prod-456";
      await expect(generateProductQRSVG(productId)).rejects.toThrow("SVG generation failed");
    });
  });
});
