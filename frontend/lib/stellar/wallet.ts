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

  const { address: publicKey, error } = await getAddress();
  if (error || !publicKey) {
    throw new WalletError(error || "Failed to retrieve public key");
  }

  return { account: { publicKey } };
}

export async function disconnectWallet(): Promise<void> {
  // Freighter doesn't have a programmatic disconnect that clears permissions in the extension,
  // but we can clear our local state.
  return;
}

export async function signWithFreighter(xdr: string, network: string): Promise<string> {
  const { signedTxXdr, error } = await signTransaction(xdr, { networkPassphrase: network });
  if (error || !signedTxXdr) {
    throw new WalletError(error || "Failed to sign transaction");
  }
  return signedTxXdr;
}
