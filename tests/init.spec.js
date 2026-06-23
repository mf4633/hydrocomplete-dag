// @ts-check
/**
 * Initialisation and DOM tests.
 * Verifies the WASM loads, the palette renders, and all UI elements are present.
 */
const { test, expect } = require('@playwright/test');

// Helper: wait for the WASM editor to be ready on window.__editor
async function waitForEditor(page) {
  await page.waitForFunction(
    () => typeof window.__editor !== 'undefined' && typeof window.__DagEditor !== 'undefined',
    { timeout: 15_000 }
  );
}

test.describe('Initialisation', () => {
  test.beforeEach(async ({ page }) => {
    // Suppress console errors from the page to keep test output clean
    page.on('pageerror', err => { if (!err.message.includes('autosave')) throw err; });
    await page.goto('/');
    await waitForEditor(page);
  });

  test('DagEditor WASM class is available', async ({ page }) => {
    const ok = await page.evaluate(() => typeof window.__DagEditor === 'function');
    expect(ok).toBe(true);
  });

  test('editor instance is a live object', async ({ page }) => {
    const ok = await page.evaluate(() =>
      window.__editor !== null && typeof window.__editor.node_count === 'function'
    );
    expect(ok).toBe(true);
  });

  test('palette_json returns exactly 20 node kinds', async ({ page }) => {
    const count = await page.evaluate(() =>
      JSON.parse(window.__DagEditor.palette_json()).length
    );
    expect(count).toBe(20);
  });

  test('palette_json covers the 4 expected categories', async ({ page }) => {
    const cats = await page.evaluate(() => {
      const items = JSON.parse(window.__DagEditor.palette_json());
      return [...new Set(items.map(i => i.category))].sort();
    });
    expect(cats).toEqual(['Hydraulics', 'Hydrology', 'Sink', 'Water Quality']);
  });

  test('every node kind has at least one config field (except Outfall)', async ({ page }) => {
    const bad = await page.evaluate(() => {
      const items = JSON.parse(window.__DagEditor.palette_json());
      return items
        .filter(i => i.kind !== 'outfall' && i.config_fields.length === 0)
        .map(i => i.kind);
    });
    expect(bad).toEqual([]);
  });

  test('all select-type config fields have at least one option', async ({ page }) => {
    const bad = await page.evaluate(() => {
      const items = JSON.parse(window.__DagEditor.palette_json());
      const failures = [];
      for (const item of items) {
        for (const f of item.config_fields) {
          if (f.field_type === 'select' && f.options.length === 0) {
            failures.push(`${item.kind}.${f.key}`);
          }
        }
      }
      return failures;
    });
    expect(bad).toEqual([]);
  });

  test('canvas element is present and has non-zero dimensions', async ({ page }) => {
    const dims = await page.evaluate(() => {
      const c = document.getElementById('dag-canvas');
      return { w: c?.width ?? 0, h: c?.height ?? 0 };
    });
    expect(dims.w).toBeGreaterThan(100);
    expect(dims.h).toBeGreaterThan(100);
  });

  test('minimap canvas is present', async ({ page }) => {
    await expect(page.locator('#minimap-canvas')).toBeVisible();
  });
});

test.describe('DOM — toolbar', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForEditor(page);
  });

  const buttons = ['btn-new', 'btn-save', 'btn-load', 'btn-report', 'btn-svg',
                   'btn-run', 'btn-del', 'btn-undo', 'btn-redo',
                   'btn-layout', 'btn-fit', 'tmpl-select'];

  for (const id of buttons) {
    test(`#${id} is visible`, async ({ page }) => {
      await expect(page.locator(`#${id}`)).toBeVisible();
    });
  }
});

test.describe('DOM — palette', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForEditor(page);
  });

  test('palette renders all 20 node cards', async ({ page }) => {
    const count = await page.locator('.pnode').count();
    expect(count).toBe(20);
  });

  test('category headers are rendered', async ({ page }) => {
    const headers = await page.locator('.cat-hdr').allTextContents();
    const expected = ['Hydrology', 'Hydraulics', 'Water Quality', 'Sink'];
    for (const h of expected) {
      expect(headers.some(t => t.includes(h))).toBe(true);
    }
  });

  test('palette search input is present and focusable', async ({ page }) => {
    const inp = page.locator('#palette-search');
    await expect(inp).toBeVisible();
    await inp.focus();
    await expect(inp).toBeFocused();
  });

  test('palette search filters nodes', async ({ page }) => {
    await page.fill('#palette-search', 'manning');
    // Wait for filter to apply
    await page.waitForTimeout(100);
    const visible = await page.locator('.pnode:visible').count();
    const total   = await page.locator('.pnode').count();
    expect(visible).toBeLessThan(total);
    expect(visible).toBeGreaterThan(0);
  });

  test('palette search clear (Escape) restores all nodes', async ({ page }) => {
    await page.fill('#palette-search', 'pond');
    await page.waitForTimeout(100);
    await page.keyboard.press('Escape');
    await page.waitForTimeout(100);
    const visible = await page.locator('.pnode:visible').count();
    expect(visible).toBe(20);
  });

  test('Templates dropdown has 5 entries', async ({ page }) => {
    const opts = await page.locator('#tmpl-select option').count();
    expect(opts).toBe(6); // 1 placeholder + 5 templates
  });
});

test.describe('DOM — status footer', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForEditor(page);
  });

  test('footer shows node count', async ({ page }) => {
    await expect(page.locator('#ft-nodes')).toBeVisible();
  });

  test('footer shows zoom percent', async ({ page }) => {
    const text = await page.locator('#ft-zoom').textContent();
    expect(text).toMatch(/\d+%/);
  });

  test('snap button is present', async ({ page }) => {
    await expect(page.locator('#snap-btn')).toBeVisible();
  });
});

test.describe('DOM — overlays', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForEditor(page);
  });

  test('shortcuts overlay is hidden by default', async ({ page }) => {
    await expect(page.locator('#shortcuts-overlay')).not.toHaveClass(/open/);
  });

  test('pressing ? opens the shortcuts overlay', async ({ page }) => {
    await page.locator('#dag-canvas').click(); // focus
    await page.keyboard.press('?');
    await expect(page.locator('#shortcuts-overlay')).toHaveClass(/open/);
  });

  test('clicking outside shortcuts overlay closes it', async ({ page }) => {
    await page.locator('#dag-canvas').click();
    await page.keyboard.press('?');
    await expect(page.locator('#shortcuts-overlay')).toHaveClass(/open/);
    await page.locator('#shortcuts-overlay').click({ position: { x: 5, y: 5 } });
    await expect(page.locator('#shortcuts-overlay')).not.toHaveClass(/open/);
  });

  test('label input is hidden by default', async ({ page }) => {
    await expect(page.locator('#label-input')).toBeHidden();
  });
});
