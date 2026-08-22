const { chromium } = require('playwright');
const assert = require('assert');

(async () => {
    console.log('🚀 Testing Live DAGR Dashboard on repository: Automate-Instagram-Posts...\n');
    const browser = await chromium.launch({ headless: true });
    const page = await browser.newPage();

    try {
        await page.goto('http://127.0.0.1:3334');
        await page.waitForLoadState('networkidle');
        await page.waitForTimeout(1000); // Allow SSE / initial stats fetch

        // 1. Verify Brand & Workspace context
        const wordmark = await page.locator('.brand-wordmark').first().innerText();
        assert.strictEqual(wordmark, 'dagr', 'Brand wordmark must be dagr');

        const workspaceText = await page.locator('#workspace-path').innerText();
        console.log('  ✓ Verified Active Workspace Context:', workspaceText);
        assert(workspaceText.includes('Automate-Instagram-Posts'), 'Workspace must match target repo');

        // 2. Verify Live KPI Scoreboard Stats
        const tokensSaved = await page.locator('#total-tokens-saved').innerText();
        const usdSaved = await page.locator('#total-usd-saved').innerText();
        const compressionRatio = await page.locator('#compression-ratio-pct').innerText();
        const totalSlices = await page.locator('#total-slices-count').innerText();

        console.log('  📊 Live Telemetry Scoreboard Metrics:');
        console.log(`     • Lifetime Tokens Saved: ${tokensSaved}`);
        console.log(`     • Estimated USD Saved:   ${usdSaved}`);
        console.log(`     • Compression Ratio:     ${compressionRatio}`);
        console.log(`     • AST Slices Served:     ${totalSlices}`);

        assert(parseInt(tokensSaved.replace(/,/g, '')) > 0, 'Tokens saved must be > 0');

        // 3. Test Navigation Tabs & Charts
        // Overview Tab
        assert(await page.locator('#velocityChart').isVisible(), 'Velocity Chart must be visible');

        // Live Feed Tab
        await page.click('#tab-btn-live');
        await page.waitForTimeout(400);
        assert(await page.locator('#tab-live').isVisible(), 'Live feed tab must be visible');
        console.log('  ✓ Live Event Feed Ledger tab verified');

        // AST Graph Tab
        await page.click('#tab-btn-graph');
        await page.waitForTimeout(400);
        assert(await page.locator('#tab-graph').isVisible(), 'AST graph tab must be visible');
        assert(await page.locator('#graphCanvas').isVisible(), 'Graph canvas must be visible');
        console.log('  ✓ Interactive AST Code Graph canvas verified');

        // Guard Policy Tab
        await page.click('#tab-btn-guard');
        await page.waitForTimeout(400);
        assert(await page.locator('#tab-guard').isVisible(), 'Guard policy tab must be visible');
        console.log('  ✓ Architectural Guard Policy tab verified');

        // Switch back to Overview and capture visual screenshot
        await page.click('#tab-btn-overview');
        await page.waitForTimeout(500);
        await page.screenshot({ path: 'target/automate_instagram_dashboard_live.png', fullPage: true });
        console.log('\n  📸 Captured full-page live dashboard screenshot to: target/automate_instagram_dashboard_live.png');

        console.log('\n🎉 =====================================================================');
        console.log('✅ ALL LIVE TELEMETRY & DASHBOARD TESTS ON Automate-Instagram-Posts PASSED!');
        console.log('🎉 =====================================================================\n');

    } catch (err) {
        console.error('❌ Test failed:', err);
        process.exit(1);
    } finally {
        await browser.close();
    }
})();
