import { test, expect } from "@playwright/test";

/**
 * Product verification journey — public page, no wallet required.
 * Consumers scan a QR code and land on /verify/[id].
 */
test.describe("Product verification", () => {
    test("verify page renders for a given product ID", async ({ page }) => {
        await page.goto("/verify/PROD-001");

        await expect(
            page.getByRole("heading", { name: /verified authentic/i })
        ).toBeVisible();
    });

    test("displays the product ID on the verify page", async ({ page }) => {
        await page.goto("/verify/TEST-PRODUCT-XYZ");

        await expect(page.getByText("TEST-PRODUCT-XYZ")).toBeVisible();
    });

    test("shows supply chain journey section", async ({ page }) => {
        await page.goto("/verify/PROD-001");

        await expect(page.getByText(/supply chain journey/i)).toBeVisible();
    });

    test("shows current status badge", async ({ page }) => {
        await page.goto("/verify/PROD-001");

        await expect(page.getByText(/ready for sale/i)).toBeVisible();
    });

    test("verify page works without wallet connection", async ({ page }) => {
        // No wallet mock — should still load
        await page.goto("/verify/PUBLIC-PRODUCT-001");

        await expect(
            page.getByRole("heading", { name: /verified authentic/i })
        ).toBeVisible();
    });
});
