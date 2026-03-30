export const CONTRACT_CONFIG = {
  CONTRACT_ID: process.env.NEXT_PUBLIC_CONTRACT_ID || "",
  NETWORK: (process.env.NEXT_PUBLIC_STELLAR_NETWORK || "testnet") as
    | "testnet"
    | "mainnet"
    | "futurenet",
  RPC_URL:
    process.env.NEXT_PUBLIC_RPC_URL ||
    (process.env.NEXT_PUBLIC_STELLAR_NETWORK === "mainnet"
      ? "https://soroban-rpc.mainnet.stellar.org"
      : process.env.NEXT_PUBLIC_STELLAR_NETWORK === "futurenet"
        ? "https://rpc-futurenet.stellar.org"
        : "https://soroban-testnet.stellar.org"),
};

export function validateContractConfig(): void {
  if (!CONTRACT_CONFIG.CONTRACT_ID) {
    throw new Error(
      "Missing contract configuration: NEXT_PUBLIC_CONTRACT_ID is not set. Add it to your .env.local (or CI environment) and restart the dev server."
    );
  }
}
