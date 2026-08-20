const { chromium } = require('playwright');
const path = require('path');
const assert = require('assert');

(async () => {
    console.log('🧪 Starting E2E Verification for Codebase Importer & 3D WebGL Orbit Graph...');
    const browser = await chromium.launch({ headless: true });
    const page = await browser.newPage();

    const filePath = 'file://' + path.resolve(__dirname, '../../site/index.html');
    await page.goto(filePath);

    // 1. Verify Codebase Ingestion Section Exists
    const isCodebaseSectionVisible = await page.locator('#codebase-ingest').isVisible();
    assert(isCodebaseSectionVisible, 'Codebase ingestion section must be visible');

    // 2. Test Ingestion Mode Switching (GitHub -> ZIP -> Folder)
    await page.click('#ingest-tab-zip');
    assert(await page.locator('#ingest-panel-zip').isVisible(), 'ZIP panel must be visible');

    await page.click('#ingest-tab-folder');
    assert(await page.locator('#ingest-panel-folder').isVisible(), 'Folder panel must be visible');

    await page.click('#ingest-tab-github');
    assert(await page.locator('#ingest-panel-github').isVisible(), 'GitHub panel must be visible');

    // 3. Test In-Browser Multi-File Ingestion & Symbol Indexing
    const indexResult = await page.evaluate(() => {
        globalCodebaseImporter.addFile('src/auth/jwt.ts', `
            export interface TokenPayload {
                userId: string;
                role: string;
            }
            export function verifyJwtToken(token: string): TokenPayload {
                return { userId: "user_123", role: "admin" };
            }
        `);
        globalCodebaseImporter.addFile('src/billing/stripe.ts', `
            export interface StripeCharge {
                amount: number;
            }
            export function createStripeCharge(charge: StripeCharge): boolean {
                return true;
            }
        `);
        globalCodebaseImporter.indexSymbols();
        return {
            totalFiles: globalCodebaseImporter.files.size,
            totalSymbols: globalCodebaseImporter.symbolIndex.length,
            symbols: globalCodebaseImporter.symbolIndex.map(s => s.name)
        };
    });

    assert.strictEqual(indexResult.totalFiles, 2, 'Should index 2 files in virtual workspace');
    assert(indexResult.symbols.includes('verifyJwtToken'), 'Should extract verifyJwtToken');
    assert(indexResult.symbols.includes('createStripeCharge'), 'Should extract createStripeCharge');

    // 4. Test 2D / 3D Graph Switcher
    await page.click('#graph-view-btn-3d');
    const is3DContainerVisible = await page.locator('#graph3dContainer').isVisible();
    assert(is3DContainerVisible, '3D WebGL container must be visible when 3D mode is toggled');

    // 5. Test 1-Click Codebase Symbol Slicing
    await page.evaluate(() => {
        sliceCodebaseSymbol('src/auth/jwt.ts', 'verifyJwtToken', 'typescript');
    });

    const targetLabel = await page.locator('#sim-target-label').innerText();
    assert(targetLabel.includes('verifyJwtToken'), 'Target label should update to verifyJwtToken');

    const slicedCode = await page.locator('#sim-sliced-code').innerText();
    assert(slicedCode.includes('TokenPayload'), 'Should hoist TokenPayload interface');
    assert(slicedCode.includes('verifyJwtToken'), 'Should slice verifyJwtToken function');

    console.log('✅ ALL CODEBASE INGESTION & 3D WEBGL GRAPH TESTS PASSED 100%!');
    await browser.close();
})();
