import { test, expect } from "@playwright/test";
import { mockConnectedWallet, mockDisconnectedWallet } from "./fixtures";

/**
 * Product registration journey — critical user flow.
 *
 * Steps:
 *  1. Navigate to /register
 *  2. Fill Step 1 (Product ID + Name)
 *  3. Fill Step 2 (Origin + Category + optional Description)
 *  4. Review Step 3 and submit
 *  5. See success screen with transaction hash
 */
test.describe("Product registration", () => {
    test("shows wallet warning on review step when disconnected", async ({
        page,
    }) => {
        await mockDisconnectedWallet(page);
        await page.goto("/register");

        // Step 1
        await page.getByLabel(/product id/i).fill("TEST-001");
        await page.getByLabel(/product name/i).fill("Test Coffee Beans");
        await page.getByRole("button", { name: /next/i }).click();

        // Step 2
        await page.getByLabel(/origin location/i).fill("Ethiopia");
        await page.getByLabel(/category/i).selectOption("Food & Beverage");
        await page.getByRole("button", { name: /next/i }).click();

        // Step 3 — review
        await expect(page.getByText(/TEST-001/)).toBeVisible();
        await expect(page.getByText(/Test Coffee Beans/)).toBeVisible();
        await expect(page.getByText(/Ethiopia/)).toBeVisible();

        // Wallet warning should appear
        await expect(
            page.getByRole("alert").filter({ hasText: /connect.*wallet/i })
        ).toBeVisible();

        // Submit button should be disabled
        await expect(
            page.getByRole("button", { name: /register product/i })
        ).toBeDisabled();
    });

    test("completes full registration flow with connected wallet", async ({
        page,
    }) => {
        await mockConnectedWallet(page);
        await page.goto("/register");

        // Step 1 — Basic Info
        await expect(
            page.getByRole("heading", { name: /basic product information/i })
        ).toBeVisible();
        await page.getByLabel(/product id/i).fill("SKU-E2E-001");
        await page.getByLabel(/product name/i).fill("E2E Test Product");
        await page.getByRole("button", { name: /next/i }).click();

        // Step 2 — Origin & Category
        await expect(
            page.getByRole("heading", { name: /origin.*category/i })
        ).toBeVisible();
        await page.getByLabel(/origin location/i).fill("Colombia");
        await page.getByLabel(/category/i).selectOption("Food & Beverage");
        await page.getByLabel(/description/i).fill("An E2E test product");
        await page.getByRole("button", { name: /next/i }).click();

        // Step 3 — Review
        await expect(
            page.getByRole("heading", { name: /review/i })
        ).toBeVisible();
        await expect(page.getByText("SKU-E2E-001")).toBeVisible();
        await expect(page.getByText("E2E Test Product")).toBeVisible();
        await expect(page.getByText("Colombia")).toBeVisible();
        await expect(page.getByText("Food & Beverage")).toBeVisible();

        // Submit
        const submitBtn = page.getByRole("button", { name: /register product/i });
        await expect(submitBtn).toBeEnabled();
        await submitBtn.click();

        // Success screen
        await expect(
            page.getByRole("heading", { name: /registration successful/i })
        ).toBeVisible({ timeout: 10_000 });
        await expect(page.getByText(/transaction hash/i)).toBeVisible();
    });

    test("validates required fields before advancing", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/register");

        // Try to advance without filling anything
        await page.getByRole("button", { name: /next/i }).click();

        // Should stay on step 1 and show validation errors
        await expect(
            page.getByRole("heading", { name: /basic product information/i })
        ).toBeVisible();
        // At least one error message should appear
        await expect(page.locator('[role="alert"]').first()).toBeVisible();
    });

    test("back button navigates to previous step", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/register");

        await page.getByLabel(/product id/i).fill("SKU-BACK-001");
        await page.getByLabel(/product name/i).fill("Back Test");
        await page.getByRole("button", { name: /next/i }).click();

        // Now on step 2 — go back
        await page.getByRole("button", { name: /back/i }).click();

        // Should be back on step 1 with values preserved
        await expect(
            page.getByRole("heading", { name: /basic product information/i })
        ).toBeVisible();
    });

    test("step indicator reflects current step", async ({ page }) => {
        await mockConnectedWallet(page);
        await page.goto("/register");

        // Step 1 indicator active
        await expect(page.getByText(/basic info/i)).toBeVisible();

        await page.getByLabel(/product id/i).fill("SKU-STEP-001");
        await page.getByLabel(/product name/i).fill("Step Test");
        await page.getByRole("button", { name: /next/i }).click();

        // Step 2 indicator visible
        await expect(page.getByText(/origin details/i)).toBeVisible();
    });
});
