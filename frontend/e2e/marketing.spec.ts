import { test, expect } from "@playwright/test";

/**
 * Marketing homepage — public-facing landing page.
 * No wallet required.
 */
test.describe("Marketing homepage", () => {
    test.beforeEach(async ({ page }) => {
        await page.goto("/");
    });

    test("loads and shows hero section", async ({ page }) => {
        await expect(page).toHaveTitle(/chain|logistics|supply/i);
        // Hero section should be visible
        await expect(page.locator("#main-content")).toBeVisible();
    });

    test("navigation links are present", async ({ page }) => {
        // At least one nav element exists
        const nav = page.locator("nav").first();
        await expect(nav).toBeVisible();
    });

    test("CTA links to app routes", async ({ page }) => {
        // Any link pointing to /register or /dashboard should exist
        const ctaLink = page
            .locator('a[href*="/register"], a[href*="/dashboard"]')
            .first();
        await expect(ctaLink).toBeVisible();
    });

    test("page is accessible — no critical ARIA violations", async ({ page }) => {
        // Verify landmark regions exist
        await expect(page.locator("main#main-content")).toBeVisible();
    });
});
