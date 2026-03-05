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

async function withTimeout<T>(promise: Promise<T>, ms: number, message: string): Promise<T> {
  let timeoutId: ReturnType<typeof setTimeout> | null = null;
  const timeoutPromise = new Promise<never>((_, reject) => {
    timeoutId = setTimeout(() => reject(new WalletError(message)), ms);
  });

  try {
    return await Promise.race([promise, timeoutPromise]);
  } finally {
    if (timeoutId !== null) clearTimeout(timeoutId);
  }
}

export async function connectWallet(): Promise<WalletConnectionResult> {
  const { error: connError } = await isConnected();
  if (connError) throw new WalletError(connError);

  // `isConnected()` can legitimately be false when the user hasn't yet authorized the app.
  // We should still attempt `requestAccess()`; it will prompt the user and return the address.
  try {
    const { address, error } = await withTimeout(
      requestAccess(),
      20000,
      "Wallet connection timed out. Check that Freighter is installed/unlocked and that the popup wasn't blocked."
    );
    if (error || !address) {
      throw new WalletError(error || "Access denied by user");
    }
    return { account: { publicKey: address } };
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : "Failed to connect wallet";
    // Freighter API error messages vary; normalize the common "not installed" case.
    if (message.toLowerCase().includes("not installed") || message.toLowerCase().includes("freighter")) {
      throw new WalletError(message);
    }
    throw err;
  }
}

export async function disconnectWallet(): Promise<void> {
  // Freighter doesn't support programmatic disconnect; local state is cleared by the store.
  return;
}

export async function getFreighterNetwork(): Promise<"testnet" | "mainnet" | "futurenet" | null> {
  try {
    const freighterApi = await import("@stellar/freighter-api");
    const anyApi = freighterApi as unknown as {
      getNetwork?: () => Promise<Record<string, unknown>>;
      getNetworkDetails?: () => Promise<Record<string, unknown>>;
    };

    const getErrorString = (res: unknown): string | null => {
      if (!res || typeof res !== "object") return null;
      const record = res as Record<string, unknown>;
      return typeof record.error === "string" && record.error ? record.error : null;
    };

    const normalize = (value: string) => {
      const raw = value.toLowerCase();
      // Mainnet is often referred to as "pubnet" or "Public Global Stellar Network".
      if (raw.includes("main") || raw.includes("pubnet") || raw.includes("public global")) return "mainnet" as const;
      if (raw.includes("public") && raw.includes("stellar")) return "mainnet" as const;
      if (raw.includes("future")) return "futurenet" as const;
      // Testnet is often referred to as "Test SDF Network".
      if (raw.includes("test sdf") || raw.includes("test")) return "testnet" as const;
      return null;
    };

    const pickNetworkString = (res: Record<string, unknown>): string => {
      const candidates = [
        res.networkPassphrase,
        res.passphrase,
        res.networkName,
        res.network,
        res.networkUrl,
      ];

      for (const c of candidates) {
        if (typeof c === "string" && c.trim().length > 0) return c;
      }
      return "";
    };

    if (typeof anyApi.getNetworkDetails === "function") {
      const res = await anyApi.getNetworkDetails();
      if (getErrorString(res)) {
        return null;
      }
      const rawValue = res && typeof res === "object" ? pickNetworkString(res as Record<string, unknown>) : "";
      return normalize(rawValue) ?? null;
    }

    if (typeof anyApi.getNetwork === "function") {
      const res = await anyApi.getNetwork();
      if (getErrorString(res)) {
        return null;
      }
      const rawValue = res && typeof res === "object" ? pickNetworkString(res as Record<string, unknown>) : "";
      return normalize(rawValue) ?? null;
    }

    return null;
  } catch {
    return null;
  }
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
