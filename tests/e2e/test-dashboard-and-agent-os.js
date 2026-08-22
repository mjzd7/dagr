const { chromium } = require('playwright');
const path = require('path');
const assert = require('assert');

(async () => {
    console.log('🚀 Starting Comprehensive Playwright E2E Test Suite for DAGR Agent OS...\n');
    const browser = await chromium.launch({ headless: true });

    try {
        // =========================================================================
        // SUITE 1: Embedded DAGR Hypervisor & Telemetry Dashboard
        // =========================================================================
        console.log('📦 [SUITE 1] Testing Embedded DAGR Hypervisor & Telemetry Control Plane...');
        const page1 = await browser.newPage();
        const dashboardPath = 'file://' + path.resolve(__dirname, '../../crates/dagr-cli/src/web/dashboard.html');
        await page1.goto(dashboardPath);

        // 1. Verify Brand Wordmark & Version Badge
        const wordmark = await page1.locator('.brand-wordmark').first().innerText();
        assert.strictEqual(wordmark, 'dagr', 'Brand wordmark must be lowercase dagr');
        console.log('  ✓ Brand wordmark verified: "dagr"');


        // 2. Verify KPI Metric Scoreboard Cards
        assert(await page1.locator('#total-tokens-saved').isVisible(), 'Total tokens saved card must be visible');
        assert(await page1.locator('#total-usd-saved').isVisible(), 'USD saved FinOps card must be visible');
        assert(await page1.locator('#compression-ratio-pct').isVisible(), 'Compression ratio card must be visible');
        assert(await page1.locator('#violations-prevented').isVisible(), 'Violations prevented card must be visible');
        console.log('  ✓ All 4 Metric KPI Cards rendered and visible');

        // 3. Verify Velocity Chart Canvas
        assert(await page1.locator('#velocityChart').isVisible(), 'Velocity Chart canvas must be visible');
        console.log('  ✓ 2D Chart.js Velocity Canvas verified');

        // 4. Test Navigation Tabs Switching
        // Tab: Live Feed
        await page1.click('#tab-btn-live');
        assert(await page1.locator('#tab-live').isVisible(), 'Live feed tab must be visible');
        assert(await page1.locator('#events-table-body').isVisible(), 'Live events table body must be visible');
        console.log('  ✓ Live Feed tab and event ledger table verified');

        // Tab: AST Graph
        await page1.click('#tab-btn-graph');
        assert(await page1.locator('#tab-graph').isVisible(), 'AST graph tab must be visible');
        assert(await page1.locator('#graphCanvas').isVisible(), 'Force-directed graph canvas must be visible');
        console.log('  ✓ Interactive AST Code Graph canvas verified');

        // Tab: Guard Policy
        await page1.click('#tab-btn-guard');
        assert(await page1.locator('#tab-guard').isVisible(), 'Guard policy tab must be visible');
        console.log('  ✓ Architectural Guard Policy tab verified');

        // Tab: Admin Login
        await page1.click('#tab-btn-admin');
        assert(await page1.locator('#tab-admin').isVisible(), 'Admin login tab must be visible');
        console.log('  ✓ Admin Login tab verified');

        // Tab: Export
        await page1.click('#tab-btn-export');
        assert(await page1.locator('#tab-export').isVisible(), 'Export tab must be visible');
        assert(await page1.locator('a:has-text("Download CSV Ledger")').isVisible(), 'CSV export link must be visible');
        assert(await page1.locator('a:has-text("Download JSON Stream")').isVisible(), 'JSON export link must be visible');
        console.log('  ✓ CSV & JSON telemetry export surfaces verified');


        // Capture screenshot
        await page1.screenshot({ path: 'target/playwright_dashboard_verified.png' });
        console.log('  ✓ Dashboard screenshot captured to target/playwright_dashboard_verified.png\n');
        await page1.close();

        // =========================================================================
        // SUITE 2: DAGR Landing Page & Brand Surface
        // =========================================================================
        console.log('📦 [SUITE 2] Testing DAGR Public Landing Page & Aesthetic Brand Surface...');
        const page2 = await browser.newPage();
        const sitePath = 'file://' + path.resolve(__dirname, '../../site/index.html');
        await page2.goto(sitePath);

        // 1. Verify Title and Hero Copy
        const title = await page2.title();
        assert(title.includes('dagr'), 'Site title must include dagr');

        const heroHeadline = await page2.locator('h1').first().innerText();
        assert(heroHeadline.length > 0, 'Hero headline must be present');
        console.log('  ✓ Public Landing Page loaded with headline:', heroHeadline.trim().replace(/\n/g, ' '));

        // 2. Verify Titanium Cards Rendered
        const cardCount = await page2.locator('.titanium-card').count();
        assert(cardCount > 0, 'Titanium feature cards must be rendered');
        console.log(`  ✓ Verified ${cardCount} titanium feature cards rendered on landing page`);

        // Capture screenshot
        await page2.screenshot({ path: 'target/playwright_site_verified.png' });
        console.log('  ✓ Landing page screenshot captured to target/playwright_site_verified.png\n');
        await page2.close();

        console.log('🎉 =====================================================================');
        console.log('✅ ALL PLAYWRIGHT E2E UI, DASHBOARD & BRAND TESTS PASSED 100% GREEN!');
        console.log('🎉 =====================================================================');

    } catch (err) {
        console.error('❌ Playwright E2E Test Failed:', err);
        process.exit(1);
    } finally {
        await browser.close();
    }
})();
