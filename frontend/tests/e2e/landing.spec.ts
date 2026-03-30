import { test, expect } from "@playwright/test";

test("Landing page navigation", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByRole("link", { name: "ChainLojistic" })).toBeVisible();
  await page.locator('a[href="/register"]').first().click();
  await expect(page).toHaveURL(/\/register/);

  await expect(page.getByRole("heading", { name: "Product Registration" })).toBeVisible();
});
