import { describe, expect, it } from "vitest";

import {
  getBlockchainConfig,
  getNetworkName,
  getSupportedNetworks,
} from "@/lib/blockchain/config";

describe("blockchain config integration", () => {
  it("returns all supported networks with valid names", () => {
    const networks = getSupportedNetworks();

    expect(networks).toContain("stellar");
    expect(networks).toContain("ethereum");
    expect(networks).toContain("polygon");

    for (const network of networks) {
      const config = getBlockchainConfig(network);
      const name = getNetworkName(network);

      expect(config.network).toBe(network);
      expect(typeof config.rpcUrl).toBe("string");
      expect(name.length).toBeGreaterThan(0);
    }
  });

  it("throws on unsupported network", () => {
    expect(() => getBlockchainConfig("unknown" as never)).toThrow(
      "Unsupported blockchain network: unknown"
    );
  });
});
