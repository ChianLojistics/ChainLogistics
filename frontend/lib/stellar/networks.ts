export type StellarNetwork = "testnet" | "mainnet" | "futurenet";

export const DEFAULT_NETWORK: StellarNetwork = "testnet";

export const HORIZON_URL_BY_NETWORK: Record<StellarNetwork, string> = Object.freeze({
  testnet: "https://horizon-testnet.stellar.org",
  mainnet: "https://horizon.stellar.org",
  futurenet: "https://horizon-futurenet.stellar.org",
});
