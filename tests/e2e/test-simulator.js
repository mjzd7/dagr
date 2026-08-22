const { chromium } = require('playwright');
const path = require('path');
const assert = require('assert');

(async () => {
    console.log('🧪 Starting E2E Verification for Custom Code Simulator, Visual Graph & Telemetry History...');
    const browser = await chromium.launch({ headless: true });
    const page = await browser.newPage();

    const filePath = 'file://' + path.resolve(__dirname, '../../site/index.html');
    await page.goto(filePath);

    // 1. Check title and hero
    const heroText = await page.locator('h1').innerText();
    assert(heroText.includes('Give your AI assistant laser focus'), 'Hero title text mismatch');

    // 2. Check Visual Graph Canvas
    const isCanvasVisible = await page.locator('#astGraphCanvas').isVisible();
    assert(isCanvasVisible, 'Visual AST Graph Canvas should be visible');

    // 3. Click "🧪 Paste Your Own Code"
    await page.click('#sim-btn-custom');
    const isPanelVisible = await page.locator('#custom-code-panel').isVisible();
    assert(isPanelVisible, 'Custom code panel should be visible');

    // 4. Fill Custom Code snippet
    const customCode = `
export interface BillingRecord {
  id: string;
  amount: number;
}

export function executeBilling(record: BillingRecord): boolean {
  return record.amount > 0;
}

// 400 lines of unrelated helper code
`;
    await page.fill('#custom-code-input', customCode);
    await page.fill('#custom-symbol-input', 'executeBilling');

    // 5. Click Slice button
    await page.click('button:has-text("⚡ Slice with DAGR")');

    // 6. Verify Target Label and Sliced Code
    const targetLabel = await page.locator('#sim-target-label').innerText();
    assert(targetLabel.includes('executeBilling'), 'Target label should contain executeBilling');

    const slicedCode = await page.locator('#sim-sliced-code').innerText();
    assert(slicedCode.includes('export interface BillingRecord'), 'Should hoist BillingRecord interface');
    assert(slicedCode.includes('executeBilling'), 'Should include executeBilling function');

    // 7. Verify Telemetry Ledger Row
    const rows = await page.locator('#history-ledger-body tr').count();
    assert.strictEqual(rows, 1, 'Should have 1 history row recorded');

    const totalSlices = await page.locator('#history-total-slices').innerText();
    assert.strictEqual(totalSlices, '1', 'Total slices count should be 1');

    console.log('✅ ALL IN-BROWSER CUSTOM AST SLICING, VISUAL GRAPH & TELEMETRY TESTS PASSED 100%!');
    await browser.close();
})();
