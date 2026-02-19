export type WalletStatus = "disconnected" | "connecting" | "connected" | "error";

export type WalletAccount = {
  publicKey: string;
};

export type WalletConnectionResult = {
  account: WalletAccount;
};

export async function connectWallet(): Promise<WalletConnectionResult> {
  throw new Error("Wallet integration not implemented yet");
}

export async function disconnectWallet(): Promise<void> {
  return;
}
