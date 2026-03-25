import { describe, it, expect } from "vitest";
import { StellarNetwork, DEFAULT_NETWORK, HORIZON_URL_BY_NETWORK } from "./networks";

describe("Stellar Networks", () => {
  describe("DEFAULT_NETWORK", () => {
    it("should default to testnet", () => {
      expect(DEFAULT_NETWORK).toBe("testnet");
    });
  });

  describe("HORIZON_URL_BY_NETWORK", () => {
    it("should provide correct URLs for all networks", () => {
      expect(HORIZON_URL_BY_NETWORK.testnet).toBe("https://horizon-testnet.stellar.org");
      expect(HORIZON_URL_BY_NETWORK.mainnet).toBe("https://horizon.stellar.org");
      expect(HORIZON_URL_BY_NETWORK.futurenet).toBe("https://horizon-futurenet.stellar.org");
    });

    it("should be frozen/immutable", () => {
      expect(Object.isFrozen(HORIZON_URL_BY_NETWORK)).toBe(true);
    });
  });

  describe("StellarNetwork type", () => {
    it("should accept valid network values", () => {
      const validNetworks: StellarNetwork[] = ["testnet", "mainnet", "futurenet"];

      validNetworks.forEach(network => {
        expect(["testnet", "mainnet", "futurenet"]).toContain(network);
      });
    });
  });
});
