import { create } from "zustand";
import { persist } from "zustand/middleware";
import { connectWallet, disconnectWallet, getCurrentAddress } from "../stellar/wallet";

export type WalletState = {
  status: "disconnected" | "connecting" | "connected" | "error";
  publicKey: string | null;
  error: string | null;
  setStatus: (status: WalletState["status"]) => void;
  setPublicKey: (publicKey: string | null) => void;
  setError: (error: string | null) => void;
  connect: () => Promise<void>;
  disconnect: () => Promise<void>;
  /**
   * Call once on app mount. Verifies persisted connection is still valid
   * and starts polling to detect account switches in Freighter.
   */
  initialize: () => Promise<void>;
};

// Module-level watcher — one interval per app session.
let _accountWatcher: ReturnType<typeof setInterval> | null = null;

type SetSlice = (partial: Partial<Pick<WalletState, "status" | "publicKey" | "error">>) => void;

function startAccountWatcher(getState: () => WalletState, setState: SetSlice) {
  if (_accountWatcher !== null) return; // already running
  _accountWatcher = setInterval(async () => {
    const { status, publicKey } = getState();
    if (status !== "connected") {
      stopAccountWatcher();
      return;
    }
    const currentAddress = await getCurrentAddress();
    if (!currentAddress) {
      setState({ status: "disconnected", publicKey: null, error: null });
      stopAccountWatcher();
    } else if (currentAddress !== publicKey) {
      // User switched accounts in Freighter
      setState({ publicKey: currentAddress });
    }
  }, 3000);
}

function stopAccountWatcher() {
  if (_accountWatcher !== null) {
    clearInterval(_accountWatcher);
    _accountWatcher = null;
  }
}

export const useWalletStore = create<WalletState>()(
  persist(
    (set, get) => ({
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
            error: null,
          });
          startAccountWatcher(get, set);
        } catch (err: unknown) {
          const message = err instanceof Error ? err.message : "Failed to connect wallet";
          set({ status: "error", error: message });
          throw err;
        }
      },

      disconnect: async () => {
        stopAccountWatcher();
        await disconnectWallet();
        set({ status: "disconnected", publicKey: null, error: null });
      },

      initialize: async () => {
        const { status, publicKey } = get();
        if (status !== "connected") return;

        // Verify the persisted connection is still valid
        const currentAddress = await getCurrentAddress();
        if (!currentAddress) {
          set({ status: "disconnected", publicKey: null, error: null });
          return;
        }
        if (currentAddress !== publicKey) {
          set({ publicKey: currentAddress });
        }

        startAccountWatcher(get, set);
      },
    }),
    {
      name: "chain-logistics-wallet",
    }
  )
);
