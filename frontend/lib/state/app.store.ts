import { create } from "zustand";

export type AppState = {
  network: "testnet" | "mainnet" | "futurenet";
  setNetwork: (network: AppState["network"]) => void;
};

export const useAppStore = create<AppState>((set) => ({
  network: "testnet",
  setNetwork: (network) => set({ network }),
}));
