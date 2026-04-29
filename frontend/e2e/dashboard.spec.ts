import { test, expect } from "@playwright/test";
import { mockConnectedWallet, mockDisconnectedWallet } from "./fixtures";

/**
 * Dashboard analytics journey.
 */
test.describe("Dashboard", () => {
    test("shows connect wallet prompt when disconnected", async ({ page }) => {
        await mockDisconnectedWallet(page);
        await page.goto("/dashboard");

        await expect(
            page.getByText(/connect.*wallet|connect a wallet/i).first()
        ).toBeVisible();
    });

    test("renders stat cards when wallet is connected", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/dashboard");

        await expect(page.getByText(/total products/i)).toBeVisible();
        await expect(page.getByText(/total events/i)).toBeVisible();
        await expect(page.getByText(/top origin/i)).toBeVisible();
        await expect(page.getByText(/last updated/i)).toBeVisible();
    });

    test("refresh button is present and clickable", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/dashboard");

        const refreshBtn = page.getByRole("button", { name: /refresh/i });
        await expect(refreshBtn).toBeVisible();
        await refreshBtn.click();
        // After click it should still be visible (not navigate away)
        await expect(refreshBtn).toBeVisible();
    });

    test("activity over time section is present", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/dashboard");

        await expect(page.getByText(/activity over time/i)).toBeVisible();
    });

    test("top origins section is present", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/dashboard");

        await expect(page.getByText(/top origins/i)).toBeVisible();
    });

    test("page heading is correct", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/dashboard");

        await expect(
            page.getByRole("heading", { name: /dashboard/i })
        ).toBeVisible();
    });
});
