const { chromium } = require('playwright');
const path = require('path');
const fs = require('fs');

const outDir = process.env.OUT_DIR || path.resolve(__dirname, '../target/screenshots');

async function capture() {
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({
    viewport: { width: 1440, height: 900 },
    deviceScaleFactor: 2,
  });
  const page = await context.newPage();

  console.log('Navigating to http://localhost:3008...');
  await page.goto('http://localhost:3008', { waitUntil: 'networkidle' });
  await page.waitForTimeout(1000);

  // Helper to click surface nav button
  async function clickSurface(label) {
    console.log(`Navigating to surface: ${label}...`);
    const btn = page.locator(`button:has-text("${label}")`).first();
    if (await btn.count() > 0) {
      await btn.click();
      await page.waitForTimeout(600);
      return true;
    }
    return false;
  }

  // 1. Atlas Surface (Default Ledger theme)
  console.log('Capturing Atlas (Ledger theme)...');
  await page.screenshot({ path: path.join(outDir, 'cd-01-atlas-ledger.png') });

  // 1b. Trigger Replay Change Pulse on Atlas
  console.log('Triggering Replay Change Pulse on Atlas...');
  const pulseBtn = page.locator('button:has-text("Replay Change Pulse")').first();
  if (await pulseBtn.count() > 0) {
    await pulseBtn.click();
    await page.waitForTimeout(400);
    await page.screenshot({ path: path.join(outDir, 'cd-01b-atlas-pulse.png') });
  }

  // 2. Switch to Vault Theme
  console.log('Switching to Vault theme...');
  const vaultBtn = page.locator('button:has-text("vault")').first();
  if (await vaultBtn.count() > 0) {
    await vaultBtn.click();
    await page.waitForTimeout(500);
    await page.screenshot({ path: path.join(outDir, 'cd-02-atlas-vault.png') });
  }

  // 3. The Bridge Surface
  if (await clickSurface('The Bridge')) {
    await page.screenshot({ path: path.join(outDir, 'cd-03-bridge.png') });
  }

  // 4. The Composer Surface (with interactive parameter chip clicked and YAML tab)
  if (await clickSurface('The Composer')) {
    // Click parameter chip "30 days" to open inline editor
    const chip = page.locator('button:has-text("30 days")').first();
    if (await chip.count() > 0) {
      await chip.click();
      await page.waitForTimeout(300);
    }
    await page.screenshot({ path: path.join(outDir, 'cd-04-composer.png') });
  }

  // 5. The Ledger Surface
  if (await clickSurface('The Ledger')) {
    await page.screenshot({ path: path.join(outDir, 'cd-05-ledger.png') });
  }

  // 6. The Docket Surface
  if (await clickSurface('The Docket')) {
    await page.screenshot({ path: path.join(outDir, 'cd-06-docket.png') });
  }

  // 7. The Pipeline Surface (with quick fix or verification)
  if (await clickSurface('The Pipeline')) {
    // Click Quick-Fix: Assign Owner (T. Mori)
    const fixBtn = page.locator('button:has-text("Quick-Fix: Assign Owner")').first();
    if (await fixBtn.count() > 0) {
      await fixBtn.click();
      await page.waitForTimeout(300);
    }
    await page.screenshot({ path: path.join(outDir, 'cd-07-pipeline.png') });
  }

  // 8. Root Fabric & Identity
  if (await clickSurface('Root Fabric & Identity')) {
    await page.screenshot({ path: path.join(outDir, 'cd-08-fabric.png') });
  }

  // 9. Jurisdictions & SLSA
  if (await clickSurface('Jurisdictions & SLSA')) {
    await page.screenshot({ path: path.join(outDir, 'cd-09-jurisdictions.png') });
  }

  // 10. Policy Gates & SBOM
  if (await clickSurface('Policy Gates & SBOM')) {
    await page.screenshot({ path: path.join(outDir, 'cd-10-policy-gates.png') });
  }

  // 11. Open Copilot Drawer
  console.log('Testing Copilot drawer...');
  const copilotBtn = page.locator('button:has-text("Copilot")').first();
  if (await copilotBtn.count() > 0) {
    await copilotBtn.click();
    await page.waitForTimeout(400);
    await page.screenshot({ path: path.join(outDir, 'cd-11-copilot-drawer.png') });
  }

  console.log('All screenshots captured successfully!');
  await browser.close();
}

capture().catch((err) => {
  console.error('Error capturing screenshots:', err);
  process.exit(1);
});
