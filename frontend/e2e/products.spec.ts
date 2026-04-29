import { test, expect } from "@playwright/test";
import { mockConnectedWallet, mockDisconnectedWallet } from "./fixtures";

/**
 * Products listing & filtering journey.
 */
test.describe("Products page", () => {
    test("renders page heading", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/products");

        await expect(
            page.getByRole("heading", { name: /products/i })
        ).toBeVisible();
    });

    test("shows product cards when wallet is connected (mock data in dev)", async ({
        page,
    }) => {
        await mockConnectedWallet(page);
        await page.goto("/products");

        // Wait for loading to finish — either products appear or empty state
        await page.waitForLoadState("networkidle");

        // The product list or empty state should be visible
        const productList = page.locator('[data-testid="product-list"], .product-card, [class*="ProductCard"]');
        const emptyState = page.getByText(/no products/i);

        // One of these should be present
        const hasProducts = await productList.count() > 0;
        const hasEmpty = await emptyState.isVisible().catch(() => false);
        expect(hasProducts || hasEmpty).toBeTruthy();
    });

    test("search input is present and interactive", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/products");

        const searchInput = page.getByPlaceholder(/search/i).first();
        await expect(searchInput).toBeVisible();
        await searchInput.fill("coffee");
        await expect(searchInput).toHaveValue("coffee");
    });

    test("category filter is present", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/products");

        // Category filter select or input should exist
        const categoryFilter = page
            .locator('select, [role="combobox"]')
            .filter({ hasText: /category|all/i })
            .first();
        await expect(categoryFilter).toBeVisible();
    });

    test("shows connect prompt when disconnected", async ({ page }) => {
        await mockDisconnectedWallet(page);
        await page.goto("/products");

        await expect(
            page.getByText(/connect.*wallet/i).first()
        ).toBeVisible();
    });
});
