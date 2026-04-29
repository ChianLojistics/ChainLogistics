import { Page } from "@playwright/test";

/**
 * Injects a mock connected wallet into the Zustand wallet store.
 * This bypasses the Freighter browser extension requirement in tests.
 */
export async function mockConnectedWallet(
    page: Page,
    publicKey = "GABC1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890"
) {
    await page.addInitScript(
        ({ key, pk }) => {
            // Pre-seed localStorage so the Zustand persist middleware picks it up
            const walletState = {
                state: {
                    status: "connected",
                    publicKey: pk,
                    network: "testnet",
                    error: null,
                },
                version: 0,
            };
            localStorage.setItem(key, JSON.stringify(walletState));
        },
        { key: "chain-logistics-wallet", pk: publicKey }
    );
}

/** Clears the persisted wallet state so the page loads as disconnected. */
export async function mockDisconnectedWallet(page: Page) {
    await page.addInitScript(() => {
        localStorage.removeItem("chain-logistics-wallet");
    });
}
