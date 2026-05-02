import { test, expect } from "@playwright/test";
import { mockConnectedWallet } from "./fixtures";

/**
 * Product tracking / timeline journey.
 * Tests the /products/[id] detail page and /tracking route.
 */
test.describe("Product tracking", () => {
    test("product detail page renders with product ID", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/products/PROD-001");

        await expect(
            page.getByRole("heading", { name: /product tracking/i })
        ).toBeVisible();

        // Product ID should be displayed
        await expect(page.getByText("PROD-001")).toBeVisible();
    });

    test("supply chain timeline section is present", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/products/PROD-001");

        await expect(page.getByText(/supply chain timeline/i)).toBeVisible();
    });

    test("tracking page renders", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/tracking");

        await expect(
            page.getByRole("heading", { name: /event tracking/i })
        ).toBeVisible();
    });

    test("add tracking event page renders", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/tracking/add");

        // Page should load without crashing
        await expect(page.locator("main")).toBeVisible();
    });
});
