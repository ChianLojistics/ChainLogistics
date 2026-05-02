# End-to-End Tests

This directory contains Playwright E2E tests for the Chain Logistics frontend.

## Overview

The E2E test suite validates critical user journeys across multiple browsers and devices:

- **Browsers**: Chromium, Firefox, WebKit (Safari engine)
- **Mobile**: Chrome on Pixel 5, Safari on iPhone 13
- **Total**: 225 tests across 9 test files

## Test Coverage

### Critical User Journeys

1. **Marketing Homepage** (`marketing.spec.ts`)
   - Landing page loads correctly
   - Navigation and CTA links work
   - Basic accessibility checks

2. **Wallet Connection** (`wallet-connection.spec.ts`)
   - Connect/disconnect wallet states
   - Dashboard and products page behavior with/without wallet

3. **Product Registration** (`product-registration.spec.ts`)
   - Multi-step form flow (Basic Info → Origin → Review → Submit)
   - Form validation
   - Success screen and transaction hash display
   - Navigation between steps

4. **Products Listing** (`products.spec.ts`)
   - Product list rendering
   - Search and filter functionality
   - Wallet connection requirements

5. **Dashboard Analytics** (`dashboard.spec.ts`)
   - Stat cards display
   - Activity charts
   - Refresh functionality
   - Top origins section

6. **Product Tracking** (`product-tracking.spec.ts`)
   - Product detail page with timeline
   - Event tracking pages
   - Supply chain visualization

7. **Product Verification** (`verification.spec.ts`)
   - Public verification page (no wallet required)
   - QR code scan destination
   - Product authenticity display

8. **Navigation** (`navigation.spec.ts`)
   - Route transitions
   - Nav bar functionality
   - 404 handling
   - Post-registration redirect

9. **Accessibility** (`accessibility.spec.ts`)
   - Landmark regions (main, nav)
   - Form labels and ARIA attributes
   - Keyboard navigation
   - Screen reader announcements

## Running Tests

### Local Development

```bash
# Install Playwright browsers (first time only)
npm run playwright:install

# Run all E2E tests
npm run test:e2e

# Run with UI mode (interactive)
npm run test:e2e:ui

# Run specific test file
npx playwright test e2e/product-registration.spec.ts

# Run specific browser
npx playwright test --project=chromium

# Debug mode
npx playwright test --debug

# View last test report
npm run test:e2e:report
```

### CI/CD

E2E tests run automatically in the CI pipeline after frontend tests pass:

```yaml
# .github/workflows/ci.yml
e2e-tests:
  needs: [ frontend-tests ]
  runs-on: ubuntu-latest
  steps:
    - Install dependencies
    - Install Playwright browsers
    - Run E2E tests
    - Upload reports (on failure)
```

## Test Architecture

### Fixtures

**`fixtures/wallet.ts`** - Mock wallet connection helpers:
- `mockConnectedWallet(page, publicKey?)` - Injects connected wallet state
- `mockDisconnectedWallet(page)` - Clears wallet state

These bypass the Freighter browser extension requirement in tests by pre-seeding the Zustand store's localStorage persistence.

### Configuration

**`playwright.config.ts`** - Main configuration:
- Base URL: `http://localhost:3000`
- Retries: 2 in CI, 0 locally
- Parallel execution: 2 workers in CI
- Reporters: GitHub Actions + HTML
- Web server: Auto-starts `npm run dev`

### Environment Variables

Tests use testnet configuration:
```bash
NEXT_PUBLIC_STELLAR_NETWORK=testnet
NEXT_PUBLIC_CONTRACT_ID=
NEXT_PUBLIC_RPC_URL=https://soroban-testnet.stellar.org
```

## Writing New Tests

### Test Structure

```typescript
import { test, expect } from "@playwright/test";
import { mockConnectedWallet } from "./fixtures";

test.describe("Feature name", () => {
  test("specific behavior", async ({ page }) => {
    await mockConnectedWallet(page);
    await page.goto("/route");
    
    await expect(page.getByRole("heading")).toBeVisible();
  });
});
```

### Best Practices

1. **Use semantic selectors**: Prefer `getByRole`, `getByLabel`, `getByText` over CSS selectors
2. **Mock wallet state**: Use fixtures to avoid Freighter extension dependency
3. **Wait for visibility**: Use `toBeVisible()` instead of `toBeTruthy()`
4. **Test user flows**: Focus on complete journeys, not isolated components
5. **Accessibility**: Include ARIA checks and keyboard navigation tests
6. **Mobile coverage**: Ensure tests work on mobile viewports

### Debugging Tips

```bash
# Run with headed browser
npx playwright test --headed

# Run with slow motion
npx playwright test --headed --slow-mo=1000

# Generate test code from browser actions
npx playwright codegen http://localhost:3000

# View trace for failed test
npx playwright show-trace test-results/.../trace.zip
```

## CI Artifacts

When tests fail in CI, the following artifacts are uploaded:

- **playwright-report/** - HTML report with screenshots and videos
- **test-results/** - Raw test results and traces

Retention: 14 days for reports, 7 days for results.

## Cross-Browser Testing

All tests run on 5 configurations:

| Project | Browser | Viewport |
|---------|---------|----------|
| chromium | Chrome/Edge | 1280x720 |
| firefox | Firefox | 1280x720 |
| webkit | Safari | 1280x720 |
| mobile-chrome | Chrome | 393x851 (Pixel 5) |
| mobile-safari | Safari | 390x844 (iPhone 13) |

## Known Limitations

1. **Freighter Extension**: Tests mock wallet state instead of using the real extension
2. **Smart Contract**: Tests run against mock data when `CONTRACT_ID` is empty
3. **Network Calls**: Some tests may be flaky if external RPC endpoints are slow
4. **Visual Regression**: Not included (consider adding Percy or Playwright's visual comparison)

## Maintenance

- Update `@playwright/test` regularly: `npm update @playwright/test`
- Re-install browsers after updates: `npm run playwright:install`
- Review and update selectors when UI changes
- Add new tests for new features before merging

## Resources

- [Playwright Documentation](https://playwright.dev)
- [Best Practices](https://playwright.dev/docs/best-practices)
- [Debugging Guide](https://playwright.dev/docs/debug)
- [CI/CD Integration](https://playwright.dev/docs/ci)
