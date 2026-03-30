import { test, expect } from "@playwright/test";

test("Add event flow", async ({ page }) => {
  await page.goto("/tracking/add");

  await expect(page.getByRole("heading", { name: "Supply Chain Operations" })).toBeVisible();

  await page.locator("#product").selectOption("PRD-1001-XYZ");
  await page.getByRole("radio").filter({ hasText: "Ship" }).first().click();
  await page.locator("#location").fill("E2E Facility");

  await page.getByRole("button", { name: /Sign & Submit Event/i }).click();

  await expect(page.getByRole("heading", { name: "Event Recorded!" })).toBeVisible({ timeout: 30_000 });
});

test("View product timeline", async ({ page }) => {
  const productId = "PROD-001";
  await page.goto(`/products/${productId}`);

  await expect(page.getByRole("heading", { name: "Supply Chain Timeline" })).toBeVisible();
  await expect(page.getByText(/Mock: Shipment dispatched/i)).toBeVisible();
});
