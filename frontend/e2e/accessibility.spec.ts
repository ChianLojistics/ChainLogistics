import { test, expect } from "@playwright/test";
import { mockConnectedWallet, mockDisconnectedWallet } from "./fixtures";

/**
 * Accessibility checks for critical pages.
 * Verifies landmark regions, focus management, and ARIA attributes.
 */
test.describe("Accessibility", () => {
    test("marketing page has main landmark", async ({ page }) => {
        await page.goto("/");
        await expect(page.locator("main#main-content")).toBeVisible();
    });

    test("app pages have main landmark", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/dashboard");
        await expect(page.locator("main")).toBeVisible();
    });

    test("registration form fields have labels", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/register");

        // All inputs should have associated labels
        await expect(page.getByLabel(/product id/i)).toBeVisible();
        await expect(page.getByLabel(/product name/i)).toBeVisible();
    });

    test("form validation errors use role=alert", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/register");

        // Trigger validation
        await page.getByRole("button", { name: /next/i }).click();

        // Error messages should use role="alert" for screen readers
        const alerts = page.locator('[role="alert"]');
        await expect(alerts.first()).toBeVisible();
    });

    test("verify page has proper heading hierarchy", async ({ page }) => {
        await page.goto("/verify/PROD-001");

        const h1 = page.locator("h1");
        await expect(h1).toBeVisible();
        await expect(h1).toContainText(/verified/i);
    });

    test("interactive elements are keyboard focusable on registration form", async ({
        page,
    }) => {
        await mockConnectedWallet(page);
        await page.goto("/register");

        // Tab to the first input
        await page.keyboard.press("Tab");
        const focused = page.locator(":focus");
        await expect(focused).toBeVisible();
    });

    test("disconnected wallet alert is announced on products page", async ({
        page,
    }) => {
        await mockDisconnectedWallet(page);
        await page.goto("/products");

        // The connect wallet message should be in a visible element
        await expect(
            page.getByText(/connect.*wallet/i).first()
        ).toBeVisible();
    });
});
