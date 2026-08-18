const { test, expect } = require('@playwright/test');

test.describe('DAGR Lifetime Telemetry & ROI Dashboard', () => {
  test('loads dashboard and renders KPI metrics', async ({ page }) => {
    await page.goto('http://127.0.0.1:3333');

    // 1. Verify Page Title and Header
    await expect(page).toHaveTitle(/DAGR Hypervisor/);
    await expect(page.locator('text=DAGR HYPERVISOR')).toBeVisible();

    // 2. Verify Metric Cards
    await expect(page.locator('#total-tokens-saved')).toBeVisible();
    await expect(page.locator('#total-usd-saved')).toBeVisible();
    await expect(page.locator('#compression-ratio-pct')).toBeVisible();
    await expect(page.locator('#violations-prevented')).toBeVisible();

    // 3. Verify Canvas Chart
    await expect(page.locator('#velocityChart')).toBeVisible();
  });

  test('switches between all navigation tabs cleanly', async ({ page }) => {
    await page.goto('http://127.0.0.1:3333');

    // Tab 2: Live Feed
    await page.click('#tab-btn-live');
    await expect(page.locator('#tab-live')).toBeVisible();
    await expect(page.locator('#events-table-body')).toBeVisible();

    // Tab 3: Code Graph
    await page.click('#tab-btn-graph');
    await expect(page.locator('#tab-graph')).toBeVisible();
    await expect(page.locator('#graphCanvas')).toBeVisible();

    // Tab 4: Guard Health
    await page.click('#tab-btn-guard');
    await expect(page.locator('#tab-guard')).toBeVisible();
    await expect(page.locator('text=Presentation Layer Separation')).toBeVisible();

    // Tab 5: Export
    await page.click('#tab-btn-export');
    await expect(page.locator('#tab-export')).toBeVisible();
    await expect(page.locator('text=Download CSV Ledger')).toBeVisible();
  });

  test('captures dark-mode aesthetic screenshot', async ({ page }) => {
    await page.goto('http://127.0.0.1:3333');
    await page.waitForTimeout(500);
    await page.screenshot({ path: 'target/dashboard_preview.png', fullPage: true });
  });
});
