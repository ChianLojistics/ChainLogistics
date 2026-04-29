import { test, expect } from "@playwright/test";
import { mockConnectedWallet, mockDisconnectedWallet } from "./fixtures";

/**
 * Wallet connection journey.
 * The Freighter extension is not available in CI, so we test the UI states
 * that appear when the wallet is connected vs. disconnected.
 */
test.describe("Wallet connection", () => {
    test("shows connect prompt when wallet is disconnected", async ({ page }) => {
        await mockDisconnectedWallet(page);
        await page.goto("/dashboard");

        // Dashboard should prompt the user to connect
        await expect(
            page.getByText(/connect.*wallet|connect a wallet/i).first()
        ).toBeVisible();
    });

    test("dashboard loads data when wallet is connected", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/dashboard");

        // Stat cards should render (even with 0 values from mock data)
        await expect(page.getByText(/total products/i)).toBeVisible();
        await expect(page.getByText(/total events/i)).toBeVisible();
    });

    test("products page shows connect prompt when disconnected", async ({
        page,
    }) => {
        await mockDisconnectedWallet(page);
        await page.goto("/products");

        await expect(
            page.getByText(/connect.*wallet/i).first()
        ).toBeVisible();
    });

    test("products page loads when wallet is connected", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/products");

        await expect(page.getByRole("heading", { name: /products/i })).toBeVisible();
    });
});
