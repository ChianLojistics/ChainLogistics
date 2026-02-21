import { create } from "zustand";
import { persist } from "zustand/middleware";
import { connectWallet, disconnectWallet } from "../stellar/wallet";

export type WalletState = {
  status: "disconnected" | "connecting" | "connected" | "error";
  publicKey: string | null;
  error: string | null;
  setStatus: (status: WalletState["status"]) => void;
  setPublicKey: (publicKey: string | null) => void;
  setError: (error: string | null) => void;
  connect: () => Promise<void>;
  disconnect: () => Promise<void>;
};

export const useWalletStore = create<WalletState>()(
  persist(
    (set) => ({
      status: "disconnected",
      publicKey: null,
      error: null,
      setStatus: (status) => set({ status }),
      setPublicKey: (publicKey) => set({ publicKey }),
      setError: (error) => set({ error }),
      connect: async () => {
        set({ status: "connecting", error: null });
        try {
          const result = await connectWallet();
          set({
            status: "connected",
            publicKey: result.account.publicKey,
            error: null
          });
        } catch (err: any) {
          set({
            status: "error",
            error: err.message || "Failed to connect wallet"
          });
          throw err;
        }
      },
      disconnect: async () => {
        await disconnectWallet();
        set({ status: "disconnected", publicKey: null, error: null });
      },
    }),
    {
      name: "chain-logistics-wallet",
    }
  )
);
