import { test, expect } from "@playwright/test";

function walletStorageState(options: { publicKey: string }) {
  return {
    state: {
      status: "connected",
      publicKey: options.publicKey,
      network: "testnet",
      error: null,
    },
    version: 0,
  };
}

test("Product registration flow", async ({ page }) => {
  const publicKey = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF";

  await page.addInitScript(({ key, value }) => {
    window.localStorage.setItem(key, value);
  }, {
    key: "chain-logistics-wallet",
    value: JSON.stringify(walletStorageState({ publicKey })),
  });

  await page.goto("/register");

  // Wait for persisted wallet state to hydrate (store initialization is async).
  await expect(
    page.getByRole("button", { name: new RegExp(`Wallet connected: ${publicKey}`) })
  ).toBeVisible({ timeout: 15_000 });

  await page.getByLabel("Product ID").fill("SKU-12345");
  await page.getByLabel("Product Name").fill("Test Product");
  await page.getByRole("button", { name: "Next" }).click();

  await page.getByLabel("Origin Location").fill("E2E Origin");
  await page.getByLabel("Category").selectOption("Electronics");
  await page.getByRole("button", { name: "Next" }).click();

  const submit = page.getByRole("button", { name: "Register Product" });
  const successHeading = page.getByRole("heading", { name: "Registration Successful!" });

  await Promise.race([
    submit.waitFor({ state: "visible" }),
    successHeading.waitFor({ state: "visible" }),
  ]);

  if (await submit.isVisible().catch(() => false)) {
    await expect(submit).toBeEnabled();
    await submit.click();
  }

  await expect(successHeading).toBeVisible({ timeout: 30_000 });
  await expect(page.getByText("Transaction Hash:")).toBeVisible();
});
