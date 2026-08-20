// @ts-check
const { test, expect } = require('@playwright/test');
const path = require('path');

test.describe('DAGR Landing Page Custom Code Simulator & Telemetry Ledger', () => {
    test('should slice custom user code and record iteration in history ledger', async ({ page }) => {
        const filePath = 'file://' + path.resolve(__dirname, '../../site/index.html');
        await page.goto(filePath);

        // 1. Verify Page Loaded
        await expect(page.locator('h1')).toContainText('Cut 95% AI Token Bloat');

        // 2. Click "🧪 Paste Your Own Code" Switcher
        await page.click('#sim-btn-custom');

        // 3. Verify Custom Code Panel is Visible
        await expect(page.locator('#custom-code-panel')).toBeVisible();

        // 4. Fill in Custom Code
        const customSnippet = `
        export interface UserAccount {
            id: string;
            name: string;
            balance: number;
        }

        export function getAccountBalance(account: UserAccount): number {
            return account.balance;
        }

        // 500 lines of unrelated accounting functions
        `;

        await page.fill('#custom-code-input', customSnippet);
        await page.fill('#custom-symbol-input', 'getAccountBalance');

        // 5. Click "⚡ Slice with DAGR"
        await page.click('button:has-text("⚡ Slice with DAGR")');

        // 6. Verify Sliced Result in Side-by-Side Playground
        await expect(page.locator('#sim-target-label')).toContainText('getAccountBalance');
        await expect(page.locator('#sim-sliced-code')).toContainText('export interface UserAccount');
        await expect(page.locator('#sim-sliced-code')).toContainText('export function getAccountBalance');

        // 7. Verify History Ledger Table Updated
        const historyRows = page.locator('#history-ledger-body tr');
        await expect(historyRows).toHaveCount(1);
        await expect(historyRows.first()).toContainText('getAccountBalance');
        await expect(page.locator('#history-total-slices')).toHaveText('1');

        console.log('✅ In-browser custom code AST slicing & telemetry ledger verified successfully!');
    });
});
