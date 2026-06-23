// @ts-check
// Smoke test — runs first, confirms WASM loads before the full suite.
const { test, expect } = require('@playwright/test');

test('smoke: page loads and WASM initialises within 45 s', async ({ page }) => {
  const errors = [];
  page.on('pageerror', e => errors.push(e.message));
  page.on('console',   m => { if (m.type() === 'error') errors.push(m.text()); });

  await page.goto('/', { waitUntil: 'domcontentloaded' });

  // Wait for the WASM module to finish initialising.
  // __editor is set synchronously after `new DagEditor(...)` which is after `await init()`.
  await page.waitForFunction(
    () => typeof window.__editor !== 'undefined' &&
          typeof window.__editor.node_count === 'function',
    { timeout: 45_000, polling: 500 }
  );

  // No JS errors during load
  const criticalErrors = errors.filter(e =>
    !e.includes('autosave') && !e.includes('favicon')
  );
  expect(criticalErrors, `Page errors: ${criticalErrors.join('; ')}`).toEqual([]);

  // Basic sanity on the editor object
  const nodeCount = await page.evaluate(() => window.__editor.node_count());
  expect(typeof nodeCount).toBe('number');
  expect(nodeCount).toBe(0); // fresh load

  const paletteSize = await page.evaluate(() =>
    JSON.parse(window.__DagEditor.palette_json()).length
  );
  expect(paletteSize).toBe(20);
});
