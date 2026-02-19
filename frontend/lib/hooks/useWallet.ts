import { useWalletStore } from "@/lib/state/wallet.store";

export function useWallet() {
  return useWalletStore();
}
