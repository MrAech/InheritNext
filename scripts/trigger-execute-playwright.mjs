/*
Node script (ESM) that runs a headless browser, opens the dev frontend,
and triggers the frontend mock auto-distribution by dynamically importing
the frontend API module and calling executeEstateNow() in page context.

Usage:
  1. Start the frontend dev server:
     cd src/civ_frontend && npm run dev

  2. Install Playwright (approve before running if you want me to run installs):
     npm i -D playwright

  3. Run this script from repo root:
     node --experimental-import-meta-resolve scripts/trigger-execute-playwright.mjs

Notes:
  - The script assumes the dev server is available at http://localhost:5173.
    If your dev server uses another port, set DEV_URL environment variable:
      DEV_URL=http://localhost:3000 node scripts/trigger-execute-playwright.mjs
  - If your app requires authentication, the script will attempt to open '/dashboard'
    and will still run the dynamic import in the page context; if auth blocks, open
    a publicly available route (e.g. '/') or sign in first.
*/

import { chromium } from 'playwright';

const DEV_URL = process.env.DEV_URL || 'http://localhost:5173';
const TARGET_PATH = '/'; // change to '/dashboard' if you are already authenticated

(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({
    // increase timeout if your dev server is slow
    timeout: 120_000,
  });

  try {
    console.log(`Opening ${DEV_URL}${TARGET_PATH} ...`);
    await page.goto(DEV_URL + TARGET_PATH, { waitUntil: 'networkidle' });

    // Wait a bit for Vite module graph to be available and app to initialize
    await page.waitForTimeout(800);

    // Evaluate in page context: dynamic import of the frontend module and call executeEstateNow()
    const result = await page.evaluate(async () => {
      try {
        // Vite serves source modules under /src/... during dev
        // This import path matches the snippet used in browser console.
        const api = await import('/src/lib/api.ts');
        if (api && typeof api.executeEstateNow === 'function') {
          const ok = await api.executeEstateNow();
          // also return some mockStore audit lines if accessible (module may not export mockStore)
          return { called: true, ok };
        } else {
          return { called: false, error: 'executeEstateNow not found on module' };
        }
      } catch (e) {
        return { called: false, error: String(e) };
      }
    });

    console.log('executeEstateNow result (page):', result);

    // Optionally fetch the audit log endpoint rendered by the app (if available) by calling listAuditLog
    const audit = await page.evaluate(async () => {
      try {
        const api = await import('/src/lib/api.ts');
        if (api && typeof api.listAuditLog === 'function') {
          const a = await api.listAuditLog(20);
          return { ok: true, audit: a.slice(-10) };
        }
        return { ok: false, error: 'listAuditLog not available' };
      } catch (e) {
        return { ok: false, error: String(e) };
      }
    });

    console.log('Recent audit (page):', audit);
  } catch (err) {
    console.error('Script error:', err);
    process.exitCode = 2;
  } finally {
    await browser.close();
  }
})();
