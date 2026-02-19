import { create } from "zustand";

export type WalletState = {
  status: "disconnected" | "connecting" | "connected" | "error";
  publicKey: string | null;
  error: string | null;
  setStatus: (status: WalletState["status"]) => void;
  setPublicKey: (publicKey: string | null) => void;
  setError: (error: string | null) => void;
};

export const useWalletStore = create<WalletState>((set) => ({
  status: "disconnected",
  publicKey: null,
  error: null,
  setStatus: (status) => set({ status }),
  setPublicKey: (publicKey) => set({ publicKey }),
  setError: (error) => set({ error }),
}));
