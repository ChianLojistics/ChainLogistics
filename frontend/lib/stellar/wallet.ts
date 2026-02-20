import { isConnected, getPublicKey, requestAccess, signTransaction } from "@stellar/freighter-api";

export type WalletStatus = "disconnected" | "connecting" | "connected" | "error";

export type WalletAccount = {
  publicKey: string;
};

export type WalletConnectionResult = {
  account: WalletAccount;
};

export class WalletError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "WalletError";
  }
}

export async function connectWallet(): Promise<WalletConnectionResult> {
  const installed = await isConnected();
  if (!installed) {
    throw new WalletError("Freighter wallet not installed");
  }

  const access = await requestAccess();
  if (!access) {
    // requestAccess returns a boolean or the public key depending on version
    // Usually it returns a boolean or throws if rejected.
    // If it returns false or empty, user denied access.
    throw new WalletError("Access denied by user");
  }

  const publicKey = await getPublicKey();
  if (!publicKey) {
    throw new WalletError("Failed to retrieve public key");
  }

  return { account: { publicKey } };
}

export async function disconnectWallet(): Promise<void> {
  // Freighter doesn't have a programmatic disconnect that clears permissions in the extension,
  // but we can clear our local state.
  return;
}

export async function signWithFreighter(xdr: string, network: string): Promise<string> {
  return await signTransaction(xdr, { network });
}
