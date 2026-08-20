const { chromium } = require('playwright');
const path = require('path');
const assert = require('assert');

(async () => {
    console.log('🧪 Starting E2E Verification for 3D Zero-Hang Engine, Billboard Labels & Fullscreen Modal...');
    const browser = await chromium.launch({ headless: true });
    const page = await browser.newPage();

    const filePath = 'file://' + path.resolve(__dirname, '../../site/index.html');
    await page.goto(filePath);

    // 1. Switch to 3D Orbit Mode
    await page.click('#graph-view-btn-3d');
    const is3DVisible = await page.locator('#graph3dContainer').isVisible();
    assert(is3DVisible, '3D container must be visible');

    // 2. Check that 3D billboard labels and meshes were created
    const stats3D = await page.evaluate(() => {
        return {
            meshes: global3DVisualizer.nodeMeshes.length,
            labels: global3DVisualizer.labelSprites.length,
            lines: global3DVisualizer.lines.length,
            autoOrbit: global3DVisualizer.autoOrbit
        };
    });

    assert(stats3D.meshes >= 3, 'Should have target, contract and boundary meshes');
    assert(stats3D.labels >= 3, 'Should have billboard text labels for all nodes');
    console.log(`✓ 3D Scene Verified: ${stats3D.meshes} meshes, ${stats3D.labels} billboard sprites`);

    // 3. Open Fullscreen Modal
    await page.click('button:has-text("⛶ Fullscreen")');
    const isModalVisible = await page.locator('#graph-fullscreen-modal').isVisible();
    assert(isModalVisible, 'Fullscreen modal must be visible');

    // 4. Test Camera Controls in Fullscreen
    await page.click('button:has-text("🔄 Reset Camera")');
    await page.click('button:has-text("🌀 Auto-Orbit")');

    // 5. Close Fullscreen Modal
    await page.click('button:has-text("✕ Exit Fullscreen")');
    const isModalClosed = await page.locator('#graph-fullscreen-modal').isHidden();
    assert(isModalClosed, 'Fullscreen modal must be closed');

    // 6. Verify Graph Anatomy Guide Cards exist
    const anatomyGuide = await page.locator('text=How to Read This Code Graph').isVisible();
    assert(anatomyGuide, 'Graph Anatomy Guide section must be visible');

    console.log('✅ ALL 3D ENGINE, BILLBOARD LABELS & FULLSCREEN MODAL TESTS PASSED 100%!');
    await browser.close();
})();
