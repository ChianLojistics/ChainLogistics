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

test("Wallet connection flow (simulated)", async ({ page }) => {
  const publicKey = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF";

  await page.addInitScript(({ key, value }) => {
    window.localStorage.setItem(key, value);
  }, {
    key: "chain-logistics-wallet",
    value: JSON.stringify(walletStorageState({ publicKey })),
  });

  await page.goto("/dashboard");

  await expect(page.getByRole("button", { name: new RegExp(`Wallet connected: ${publicKey}`) })).toBeVisible();

  await page.getByRole("button", { name: new RegExp(`Wallet connected: ${publicKey}`) }).click();
  await expect(page.getByRole("menuitem", { name: "Disconnect" })).toBeVisible();
});
