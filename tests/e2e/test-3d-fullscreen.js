const { chromium } = require('playwright');
const path = require('path');
const assert = require('assert');

(async () => {
    console.log('🧪 Starting E2E Verification for 3D Zero-Hang Engine, Billboard Labels & Seamless Fullscreen HUD...');
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
            cameraDist: global3DVisualizer.cameraDistance,
            autoOrbit: global3DVisualizer.autoOrbit
        };
    });

    assert(stats3D.meshes >= 3, 'Should have target, contract and boundary meshes');
    assert(stats3D.labels >= 3, 'Should have billboard text labels for all nodes');
    assert.strictEqual(stats3D.cameraDist, 95, 'Camera distance should be constant 95');
    console.log(`✓ 3D Scene Verified: ${stats3D.meshes} meshes, ${stats3D.labels} billboard sprites, camera distance ${stats3D.cameraDist}`);

    // 3. Test Continuous Orbit Without Decay (Wait 1.5 seconds)
    await page.waitForTimeout(1500);
    const cameraDistAfterOrbit = await page.evaluate(() => global3DVisualizer.cameraDistance);
    assert.strictEqual(cameraDistAfterOrbit, 95, 'Camera distance must remain strictly 95 without decaying into 0');

    // 4. Open Seamless Fullscreen Mode
    await page.click('#graph-fullscreen-btn');
    const isHudVisible = await page.locator('#fullscreen-hud-bar').isVisible();
    assert(isHudVisible, 'Fullscreen HUD bar must be visible');

    // 5. Test Camera Controls in Fullscreen
    await page.click('button:has-text("🔄 Reset Camera")');
    await page.click('button:has-text("🔍 Zoom +")');
    await page.click('button:has-text("🔍 Zoom -")');
    await page.click('button:has-text("🌀 Auto-Orbit")');

    // 6. Close Fullscreen
    await page.click('#graph-fullscreen-btn');
    const isHudClosed = await page.locator('#fullscreen-hud-bar').isHidden();
    assert(isHudClosed, 'Fullscreen HUD bar must be hidden after exiting');

    // 7. Verify Graph Anatomy Guide Cards exist
    const anatomyGuide = await page.locator('text=How to Read This Code Graph').isVisible();
    assert(anatomyGuide, 'Graph Anatomy Guide section must be visible');

    console.log('✅ ALL 3D ZERO-HANG ENGINE, BILLBOARD LABELS & SEAMLESS FULLSCREEN TESTS PASSED 100%!');
    await browser.close();
})();
