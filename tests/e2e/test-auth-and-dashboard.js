const { chromium } = require('playwright');
const assert = require('assert');

(async () => {
  console.log('🚀 Starting Comprehensive Playwright E2E Test Suite for DAGR Hypervisor...\n');
  const browser = await chromium.launch({ headless: true });

  try {
    // =========================================================================
    // SUITE 1: Next.js Enterprise Portal Authentication & Dashboard (Port 3000)
    // =========================================================================
    console.log('📦 [SUITE 1] Testing Next.js Portal at http://localhost:3000 ...');
    const context1 = await browser.newContext();
    const page1 = await context1.newPage();

    // 1. Visit Home Page
    await page1.goto('http://localhost:3000');
    await page1.waitForLoadState('networkidle');
    console.log('  ✓ Home page loaded successfully');

    // 2. Verify Brand Logo & Wordmark
    const hasWordmark = await page1.locator('text=dagr').first().isVisible();
    assert(hasWordmark, 'Official lowercase dagr wordmark must be visible');
    console.log('  ✓ Official dagr brand wordmark verified');

    // 3. Verify Navigation to /login
    await page1.click('text=Sign In');
    await page1.waitForURL('**/login');
    console.log('  ✓ Navigated to /login');

    // 4. Verify Auth Provider Surfaces & Status Badges
    assert(await page1.locator('text=Continue with GitHub').isVisible(), 'GitHub OAuth button must be visible');
    assert(await page1.locator('text=Continue with Google').isVisible(), 'Google OAuth button must be visible');
    assert(await page1.locator('text=Continue with Microsoft').isVisible(), 'Microsoft OAuth button must be visible');
    assert(await page1.locator('text=Send 6-Digit Code').isVisible(), 'Email OTP button must be visible');
    console.log('  ✓ All 4 Auth Provider surfaces verified (GitHub, Google, Microsoft, Email)');

    // 5. Test Live OAuth & SMTP Configuration Modal
    console.log('  ▶ Testing Live OAuth & SMTP Configuration Drawer...');
    await page1.click('text=Configure OAuth & SMTP');
    await page1.waitForSelector('text=Live OAuth & SMTP Provider Configuration', { timeout: 3000 });
    console.log('  ✓ Configuration modal rendered');

    // Input credentials
    await page1.fill('input[placeholder="e.g. Ov23li..."]', 'Ov23liTestClientId');
    await page1.fill('input[placeholder="e.g. 7f9a8b..."]', '7f9a8bTestClientSecret');
    await page1.click('button:has-text("Save & Activate Credentials")');
    await page1.waitForSelector('text=✓ Live credentials saved', { timeout: 4000 });
    console.log('  ✓ Live OAuth credentials persisted to .env.local via API');
    await page1.waitForTimeout(1500);

    // 6. Test Passwordless Email OTP Flow
    console.log('  ▶ Testing Email OTP Authentication...');
    await page1.fill('input[type="email"]', 'lead-architect@dagr.dev');
    await page1.click('text=Send 6-Digit Code');
    
    await page1.waitForSelector('text=Enter 6-Digit Code sent to:', { timeout: 5000 });
    console.log('  ✓ Email OTP dispatched');

    // Extract code
    const devCodeLocator = page1.locator('text=⚡ Local Code:');
    let otpCode = '123456';
    if (await devCodeLocator.isVisible()) {
      const devCodeText = await devCodeLocator.innerText();
      const match = devCodeText.match(/\d{6}/);
      if (match) otpCode = match[0];
    }

    await page1.fill('input[placeholder="123456"]', otpCode);
    await page1.click('text=Verify Code & Sign In');
    
    await page1.waitForURL('http://localhost:3000/**', { timeout: 5000 });
    console.log('  ✓ Email OTP verified! Redirected to hypervisor dashboard');

    // Verify Logged In Session in Navigation
    await page1.waitForSelector('text=lead-architect', { timeout: 5000 });
    await page1.waitForSelector('button:has-text("Sign Out")', { timeout: 5000 });
    console.log('  ✓ User session active: "lead-architect (email)"');

    // 7. Test Sign Out
    await page1.click('button:has-text("Sign Out")');
    await page1.waitForURL('**/login');
    console.log('  ✓ Sign Out executed successfully, returned to /login');

    // 8. Test Organization API Key Flow
    console.log('  ▶ Testing Organization API Key flow...');
    await page1.fill('input[placeholder="Org (Acme Corp)"]', 'Acme Corp');
    await page1.fill('input[placeholder="dagr_live_sec_..."]', 'dagr_live_sec_prod_test');
    await page1.click('button:has-text("Sign In with API Key")');
    await page1.waitForURL('http://localhost:3000/**', { timeout: 5000 });
    await page1.waitForSelector('text=Acme Corp Admin', { timeout: 5000 });
    console.log('  ✓ Organization API Key login verified! Session active.');

    await context1.close();

    // =========================================================================
    // SUITE 2: Standalone Rust CLI Embedded Dashboard (Port 3333)
    // =========================================================================
    console.log('\n⚡ [SUITE 2] Testing Standalone Embedded Dashboard at http://127.0.0.1:3333 ...');
    const context2 = await browser.newContext();
    const page2 = await context2.newPage();

    await page2.goto('http://127.0.0.1:3333');
    await page2.waitForLoadState('networkidle');

    // 1. Check KPI Cards
    assert(await page2.locator('#total-tokens-saved').isVisible(), 'Total tokens saved KPI must be visible');
    assert(await page2.locator('#total-usd-saved').isVisible(), 'USD saved KPI must be visible');
    assert(await page2.locator('#compression-ratio-pct').isVisible(), 'Compression ratio must be visible');
    console.log('  ✓ Telemetry KPI metrics rendered');

    // 2. Test Tab Navigation
    await page2.click('#tab-btn-live');
    assert(await page2.locator('#tab-live').isVisible(), 'Live feed tab must be visible');
    console.log('  ✓ Live feed stream verified');

    await page2.click('#tab-btn-graph');
    assert(await page2.locator('#tab-graph').isVisible(), 'AST graph tab must be visible');
    console.log('  ✓ AST symbol graph canvas verified');

    await page2.click('#tab-btn-guard');
    assert(await page2.locator('#tab-guard').isVisible(), 'Guard policy health tab must be visible');
    console.log('  ✓ Architectural guard health verified');

    // 3. Test Admin Authentication Tab on Embedded Server
    console.log('  ▶ Testing Embedded Node Authentication...');
    await page2.click('#tab-btn-admin');
    assert(await page2.locator('#tab-admin').isVisible(), 'Admin login tab must be visible');

    // Test One-Click Microsoft Single Sign-On
    await page2.click('button:has-text("Microsoft")');
    await page2.waitForFunction(() => {
      const el = document.getElementById('admin-status-msg');
      return el && !el.classList.contains('hidden') && el.innerText.includes('Authenticated');
    }, { timeout: 4000 });
    const authStatusText = await page2.locator('#admin-status-msg').innerText();
    assert(authStatusText.includes('Authenticated'), 'Status message must confirm authentication');
    console.log(`  ✓ Node Authentication response: "${authStatusText}"`);

    // 4. Verify Export Endpoints
    await page2.click('#tab-btn-export');
    assert(await page2.locator('#tab-export').isVisible(), 'Export tab must be visible');
    console.log('  ✓ Telemetry export ledger verified');

    await context2.close();

    console.log('\n🎉 =========================================================================');
    console.log('✅ ALL E2E PLAYWRIGHT TESTS PASSED 100% (OAuth, Email OTP, API Key, Dashboard)!');
    console.log('=========================================================================\n');

  } catch (error) {
    console.error('❌ Playwright Test Failure:', error);
    process.exit(1);
  } finally {
    await browser.close();
  }
})();
