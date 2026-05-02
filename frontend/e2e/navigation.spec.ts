import { test, expect } from "@playwright/test";
import { mockConnectedWallet } from "./fixtures";

/**
 * Navigation & routing journey.
 * Verifies that key routes are reachable and the nav bar works.
 */
test.describe("Navigation", () => {
    test("app layout nav bar is present on dashboard", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/dashboard");

        await expect(page.locator("nav").first()).toBeVisible();
    });

    test("navigates from dashboard to products via nav", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/dashboard");

        const productsLink = page.locator('a[href*="/products"]').first();
        await expect(productsLink).toBeVisible();
        await productsLink.click();

        await expect(page).toHaveURL(/\/products/);
        await expect(
            page.getByRole("heading", { name: /products/i })
        ).toBeVisible();
    });

    test("navigates from dashboard to register via nav", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/dashboard");

        const registerLink = page.locator('a[href*="/register"]').first();
        await expect(registerLink).toBeVisible();
        await registerLink.click();

        await expect(page).toHaveURL(/\/register/);
        await expect(
            page.getByRole("heading", { name: /product registration/i })
        ).toBeVisible();
    });

    test("404 page renders for unknown routes", async ({ page }) => {
        const response = await page.goto("/this-route-does-not-exist");
        // Next.js returns 404 for unknown routes
        expect(response?.status()).toBe(404);
    });

    test("success screen redirects to dashboard", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/register");

        // Complete registration
        await page.getByLabel(/product id/i).fill("SKU-NAV-001");
        await page.getByLabel(/product name/i).fill("Nav Test Product");
        await page.getByRole("button", { name: /next/i }).click();

        await page.getByLabel(/origin location/i).fill("Brazil");
        await page.getByLabel(/category/i).selectOption("Food & Beverage");
        await page.getByRole("button", { name: /next/i }).click();

        await page.getByRole("button", { name: /register product/i }).click();

        // Wait for success screen
        await expect(
            page.getByRole("heading", { name: /registration successful/i })
        ).toBeVisible({ timeout: 10_000 });

        // Click "View Dashboard" button
        await page.getByRole("button", { name: /view dashboard/i }).click();
        await expect(page).toHaveURL(/\/dashboard/);
    });
});
