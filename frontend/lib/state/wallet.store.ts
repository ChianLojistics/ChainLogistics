 
import { create } from "zustand";
import { connectWallet, disconnectWallet, getCurrentAddress, getFreighterNetwork } from "../stellar/wallet";

const E2E_MOCKS_ENABLED = process.env.NEXT_PUBLIC_E2E_MOCKS === "true";

export type WalletState = {
  status: "disconnected" | "connecting" | "connected" | "error";
  publicKey: string | null;
  network: "testnet" | "mainnet" | "futurenet" | null;
  error: string | null;
  setStatus: (status: WalletState["status"]) => void;
  setPublicKey: (publicKey: string | null) => void;
  setNetwork: (network: WalletState["network"]) => void;
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
  (set, get) => ({
      status: "disconnected",
      publicKey: null,
      network: null,
      error: null,
      setStatus: (status) => set({ status }),
      setPublicKey: (publicKey) => set({ publicKey }),
      setNetwork: (network) => set({ network }),
      setError: (error) => set({ error }),

      connect: async () => {
        set({ status: "connecting", error: null });
        try {
          const result = await Promise.race([
            connectWallet(),
            new Promise<never>((_, reject) =>
              setTimeout(
                () =>
                  reject(
                    new Error(
                      "Wallet connection timed out. Check that Freighter is installed/unlocked and that your browser didn't block the popup."
                    )
                  ),
                20000
              )
            ),
          ]);
          const detectedNetwork = await getFreighterNetwork();
          set({
            status: "connected",
            publicKey: result.account.publicKey,
            network: detectedNetwork,
            error: null,
          });
          startAccountWatcher(get, set);
        } catch (err: unknown) {
          let message = err instanceof Error ? err.message : "Failed to connect wallet";
          if (message.toLowerCase().includes("timed out")) {
            message =
              "Wallet connection timed out. Check that Freighter is installed/unlocked and that your browser didn't block the popup.";
          }

          set({ status: "error", error: message });
          throw err;
        }
      },

      disconnect: async () => {
        stopAccountWatcher();
        await disconnectWallet();
        set({ status: "disconnected", publicKey: null, network: null, error: null });
      },

      initialize: async () => {
        const { status, publicKey } = get();

        if (E2E_MOCKS_ENABLED) {
          set({
            status: "connected",
            publicKey: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
            network: "testnet",
            error: null,
          });
          startAccountWatcher(get, set);
          return;
        }

        // If the page refreshed mid-connection, persisted state can be stuck at "connecting".
        // Reset to a safe state on boot so the UI doesn't spin forever.
        if (status === "connecting") {
          set({ status: "disconnected", publicKey: null, network: null, error: null });
          return;
        }

        if (status !== "connected") return;

        // Verify the persisted connection is still valid
        const currentAddress = await getCurrentAddress();
        if (!currentAddress) {
          set({ status: "disconnected", publicKey: null, network: null, error: null });
          return;
        }
        if (currentAddress !== publicKey) {
          set({ publicKey: currentAddress });
        }

        const detectedNetwork = await getFreighterNetwork();
        set({ network: detectedNetwork });

        startAccountWatcher(get, set);
      },
    }),
);
