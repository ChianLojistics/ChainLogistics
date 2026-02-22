import { isConnected, getAddress, requestAccess, signTransaction } from "@stellar/freighter-api";

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
  const { isConnected: connected, error: connError } = await isConnected();
  if (connError) throw new WalletError(connError);
  if (!connected) {
    throw new WalletError("Freighter wallet is not installed");
  }

  const { address, error } = await requestAccess();
  if (error || !address) {
    throw new WalletError(error || "Access denied by user");
  }

  return { account: { publicKey: address } };
}

export async function disconnectWallet(): Promise<void> {
  // Freighter doesn't support programmatic disconnect; local state is cleared by the store.
  return;
}

/**
 * Returns the currently active address if the user has previously authorized
 * the app, or null if not connected / not authorized.
 */
export async function getCurrentAddress(): Promise<string | null> {
  try {
    const { address, error } = await getAddress();
    if (error || !address) return null;
    return address;
  } catch {
    return null;
  }
}

export async function signWithFreighter(xdr: string, networkPassphrase: string): Promise<string> {
  const { signedTxXdr, error } = await signTransaction(xdr, { networkPassphrase });
  if (error || !signedTxXdr) {
    throw new WalletError(error || "Failed to sign transaction");
  }
  return signedTxXdr;
}
