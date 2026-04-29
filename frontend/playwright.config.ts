import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright E2E configuration.
 * Tests run against a locally started Next.js dev server.
 * Cross-browser coverage: Chromium, Firefox, WebKit (Safari engine).
 */
export default defineConfig({
    testDir: "./e2e",
    fullyParallel: true,
    forbidOnly: !!process.env.CI,
    retries: process.env.CI ? 2 : 0,
    workers: process.env.CI ? 2 : undefined,
    reporter: process.env.CI
        ? [["github"], ["html", { open: "never" }]]
        : [["list"], ["html", { open: "on-failure" }]],

    use: {
        baseURL: process.env.PLAYWRIGHT_BASE_URL || "http://localhost:3000",
        trace: "on-first-retry",
        screenshot: "only-on-failure",
        video: "retain-on-failure",
    },

    projects: [
        {
            name: "chromium",
            use: { ...devices["Desktop Chrome"] },
        },
        {
            name: "firefox",
            use: { ...devices["Desktop Firefox"] },
        },
        {
            name: "webkit",
            use: { ...devices["Desktop Safari"] },
        },
        // Mobile viewports
        {
            name: "mobile-chrome",
            use: { ...devices["Pixel 5"] },
        },
        {
            name: "mobile-safari",
            use: { ...devices["iPhone 13"] },
        },
    ],

    webServer: {
        command: "npm run dev",
        url: "http://localhost:3000",
        reuseExistingServer: !process.env.CI,
        timeout: 120_000,
        env: {
            NEXT_PUBLIC_STELLAR_NETWORK: "testnet",
            NEXT_PUBLIC_CONTRACT_ID: "",
            NEXT_PUBLIC_RPC_URL: "https://soroban-testnet.stellar.org",
        },
    },
});
