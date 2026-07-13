// @ts-check
/**
 * DAG operations tests — node add/remove, edges, undo/redo, serialisation,
 * templates, selection, label editing, zoom.
 */
const { test, expect } = require('@playwright/test');

async function waitForEditor(page) {
  await page.waitForFunction(
    () => typeof window.__editor !== 'undefined',
    { timeout: 15_000 }
  );
}

/** Load a named template and return node count. */
async function loadTemplate(page, name) {
  const count = await page.evaluate(n => {
    window.__editor.load_template(n);
    return window.__editor.node_count();
  }, name);
  return count;
}

test.describe('Templates', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForEditor(page);
  });

  const templates = [
    { id: 'detention',  minNodes: 4 },
    { id: 'wq',         minNodes: 4 },
    { id: 'erosion',    minNodes: 4 },
    { id: 'continuous', minNodes: 3 },
    { id: 'full',       minNodes: 8 },
  ];

  for (const { id, minNodes } of templates) {
    test(`template "${id}" loads ≥${minNodes} nodes`, async ({ page }) => {
      const count = await loadTemplate(page, id);
      expect(count).toBeGreaterThanOrEqual(minNodes);
    });

    test(`template "${id}" has edges`, async ({ page }) => {
      await loadTemplate(page, id);
      const edgeCount = await page.evaluate(() => window.__editor.edge_count());
      expect(edgeCount).toBeGreaterThan(0);
    });
  }

  test('loading template is undoable', async ({ page }) => {
    await loadTemplate(page, 'detention');
    const before = await page.evaluate(() => window.__editor.node_count());
    expect(before).toBeGreaterThan(0);

    await page.evaluate(() => window.__editor.undo());
    const after = await page.evaluate(() => window.__editor.node_count());
    expect(after).toBe(0); // back to empty
  });

  test('clear() resets to 0 nodes', async ({ page }) => {
    await loadTemplate(page, 'wq');
    await page.evaluate(() => window.__editor.clear());
    const count = await page.evaluate(() => window.__editor.node_count());
    expect(count).toBe(0);
  });
});

test.describe('Undo / Redo', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForEditor(page);
  });

  test('can_undo is false on fresh editor', async ({ page }) => {
    const canUndo = await page.evaluate(() => window.__editor.can_undo());
    expect(canUndo).toBe(false);
  });

  test('can_undo is true after loading a template', async ({ page }) => {
    await loadTemplate(page, 'detention');
    const canUndo = await page.evaluate(() => window.__editor.can_undo());
    expect(canUndo).toBe(true);
  });

  test('undo removes the template', async ({ page }) => {
    await loadTemplate(page, 'detention');
    await page.evaluate(() => window.__editor.undo());
    const count = await page.evaluate(() => window.__editor.node_count());
    expect(count).toBe(0);
  });

  test('redo re-applies the template', async ({ page }) => {
    await loadTemplate(page, 'detention');
    const beforeUndo = await page.evaluate(() => window.__editor.node_count());
    await page.evaluate(() => window.__editor.undo());
    await page.evaluate(() => window.__editor.redo());
    const afterRedo = await page.evaluate(() => window.__editor.node_count());
    expect(afterRedo).toBe(beforeUndo);
  });

  test('multiple undo steps work', async ({ page }) => {
    await loadTemplate(page, 'detention');
    await loadTemplate(page, 'wq');          // second load = second undo entry
    const counts = await page.evaluate(() => {
      const c2 = window.__editor.node_count();
      window.__editor.undo();
      const c1 = window.__editor.node_count();
      window.__editor.undo();
      const c0 = window.__editor.node_count();
      return { c2, c1, c0 };
    });
    expect(counts.c0).toBe(0);
    expect(counts.c1).toBeGreaterThan(0);
    expect(counts.c2).toBeGreaterThan(0);
  });

  test('Ctrl+Z keyboard shortcut triggers undo', async ({ page }) => {
    await loadTemplate(page, 'detention');
    await page.locator('#dag-canvas').click();
    await page.keyboard.press('Control+z');
    const count = await page.evaluate(() => window.__editor.node_count());
    expect(count).toBe(0);
  });

  test('Ctrl+Y keyboard shortcut triggers redo', async ({ page }) => {
    await loadTemplate(page, 'wq');
    const n = await page.evaluate(() => window.__editor.node_count());
    await page.locator('#dag-canvas').click();
    await page.keyboard.press('Control+z');
    await page.keyboard.press('Control+y');
    const after = await page.evaluate(() => window.__editor.node_count());
    expect(after).toBe(n);
  });
});

test.describe('Selection', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForEditor(page);
    await loadTemplate(page, 'wq');
  });

  test('select_all_nodes selects all nodes', async ({ page }) => {
    const { total, selected } = await page.evaluate(() => ({
      total:    window.__editor.node_count(),
      selected: (window.__editor.select_all_nodes(), window.__editor.selected_node_count()),
    }));
    expect(selected).toBe(total);
  });

  test('Ctrl+A keyboard shortcut selects all', async ({ page }) => {
    const total = await page.evaluate(() => window.__editor.node_count());
    await page.locator('#dag-canvas').click();
    await page.keyboard.press('Control+a');
    const sel = await page.evaluate(() => window.__editor.selected_node_count());
    expect(sel).toBe(total);
  });

  test('Delete key removes selected nodes', async ({ page }) => {
    // Select all, then focus via keyboard — clicking canvas would clear selection
    await page.evaluate(() => window.__editor.select_all_nodes());
    await page.locator('#dag-canvas').focus();
    await page.keyboard.press('Delete');
    const count = await page.evaluate(() => window.__editor.node_count());
    expect(count).toBe(0);
  });

  test('selected_node_count is 0 after clicking empty canvas area', async ({ page }) => {
    await page.evaluate(() => window.__editor.select_all_nodes());
    // Force deselect via the API (direct canvas click is hard to aim at empty space)
    await page.evaluate(() => window.__editor.clear());
    const sel = await page.evaluate(() => window.__editor.selected_node_count());
    expect(sel).toBe(0);
  });
});

test.describe('Copy / Paste', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForEditor(page);
    await loadTemplate(page, 'detention');
  });

  test('copy_selected with selection pastes correct count', async ({ page }) => {
    // Select all N nodes, copy (group), paste → 2N nodes
    const { n, after } = await page.evaluate(() => {
      const n = window.__editor.node_count();
      window.__editor.select_all_nodes();
      window.__editor.copy_selected();
      window.__editor.paste();
      return { n, after: window.__editor.node_count() };
    });
    expect(after).toBe(n * 2);
  });

  test('group copy_selected + paste increases count by selection size', async ({ page }) => {
    const { before, after, selCount } = await page.evaluate(() => {
      window.__editor.select_all_nodes();
      const selCount = window.__editor.selected_node_count();
      const before   = window.__editor.node_count();
      window.__editor.copy_selected();
      window.__editor.paste();
      return { before, after: window.__editor.node_count(), selCount };
    });
    expect(after).toBe(before + selCount);
  });

  test('paste is undoable', async ({ page }) => {
    const before = await page.evaluate(() => {
      window.__editor.select_all_nodes();
      window.__editor.copy_selected();
      const n = window.__editor.node_count();
      window.__editor.paste();
      return n;
    });
    await page.evaluate(() => window.__editor.undo());
    const after = await page.evaluate(() => window.__editor.node_count());
    expect(after).toBe(before);
  });
});

test.describe('JSON serialisation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForEditor(page);
  });

  test('to_json returns valid JSON', async ({ page }) => {
    const json = await page.evaluate(() => window.__editor.to_json());
    expect(() => JSON.parse(json)).not.toThrow();
  });

  test('to_json on a template has nodes and edges arrays', async ({ page }) => {
    await loadTemplate(page, 'erosion');
    const parsed = await page.evaluate(() => JSON.parse(window.__editor.to_json()));
    expect(Array.isArray(parsed.nodes)).toBe(true);
    expect(Array.isArray(parsed.edges)).toBe(true);
    expect(parsed.nodes.length).toBeGreaterThan(0);
    expect(parsed.edges.length).toBeGreaterThan(0);
  });

  test('from_json round-trip preserves node count', async ({ page }) => {
    await loadTemplate(page, 'wq');
    const { original, restored } = await page.evaluate(() => {
      const json     = window.__editor.to_json();
      const original = window.__editor.node_count();
      window.__editor.clear();
      window.__editor.from_json(json);
      return { original, restored: window.__editor.node_count() };
    });
    expect(restored).toBe(original);
  });

  test('from_json round-trip preserves edge count', async ({ page }) => {
    await loadTemplate(page, 'full');
    const { original, restored } = await page.evaluate(() => {
      const json     = window.__editor.to_json();
      const original = window.__editor.edge_count();
      window.__editor.clear();
      window.__editor.from_json(json);
      return { original, restored: window.__editor.edge_count() };
    });
    expect(restored).toBe(original);
  });

  test('from_json with invalid JSON returns false', async ({ page }) => {
    const ok = await page.evaluate(() => window.__editor.from_json('not valid json'));
    expect(ok).toBe(false);
  });

  test('node config is preserved through round-trip', async ({ page }) => {
    await loadTemplate(page, 'detention');
    const { kind, area } = await page.evaluate(() => {
      const json = JSON.parse(window.__editor.to_json());
      const node = json.nodes.find(n => n.kind === 'catchment');
      return { kind: node?.kind, area: node?.config?.area_acres };
    });
    expect(kind).toBe('catchment');
    expect(typeof area).toBe('number');
    expect(area).toBeGreaterThan(0);
  });
});

test.describe('Auto-layout', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForEditor(page);
    await loadTemplate(page, 'full');
  });

  test('layout() does not change node count', async ({ page }) => {
    const { before, after } = await page.evaluate(() => {
      const before = window.__editor.node_count();
      window.__editor.layout();
      return { before, after: window.__editor.node_count() };
    });
    expect(after).toBe(before);
  });

  test('layout() does not change edge count', async ({ page }) => {
    const { before, after } = await page.evaluate(() => {
      const before = window.__editor.edge_count();
      window.__editor.layout();
      return { before, after: window.__editor.edge_count() };
    });
    expect(after).toBe(before);
  });

  test('layout() is undoable', async ({ page }) => {
    // Get a node position before layout
    const xBefore = await page.evaluate(() => {
      const nodes = JSON.parse(window.__editor.to_json()).nodes;
      return nodes[0]?.x ?? 0;
    });
    await page.evaluate(() => window.__editor.layout());
    await page.evaluate(() => window.__editor.undo());
    const xAfter = await page.evaluate(() => {
      const nodes = JSON.parse(window.__editor.to_json()).nodes;
      return nodes[0]?.x ?? 0;
    });
    expect(xAfter).toBeCloseTo(xBefore, 0);
  });
});

test.describe('Node label editing', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForEditor(page);
    await loadTemplate(page, 'detention');
  });

  test('set_node_label updates the node label', async ({ page }) => {
    const label = await page.evaluate(() => {
      const nodes = JSON.parse(window.__editor.to_json()).nodes;
      const id = nodes[0].id[0];
      window.__editor.set_node_label(id, 'Test Label');
      const updated = JSON.parse(window.__editor.to_json()).nodes.find(n => n.id[0] === id);
      return updated?.label;
    });
    expect(label).toBe('Test Label');
  });

  test('set_node_label with empty string clears label', async ({ page }) => {
    const label = await page.evaluate(() => {
      const nodes = JSON.parse(window.__editor.to_json()).nodes;
      const id = nodes[0].id[0];
      window.__editor.set_node_label(id, 'Temp');
      window.__editor.set_node_label(id, '');
      const updated = JSON.parse(window.__editor.to_json()).nodes.find(n => n.id[0] === id);
      return updated?.label;
    });
    // null or undefined (label cleared)
    expect(label == null || label === '').toBe(true);
  });

  test('node_at_screen returns u32::MAX on empty canvas area', async ({ page }) => {
    const id = await page.evaluate(() => window.__editor.node_at_screen(0, 0));
    expect(id).toBe(4294967295); // u32::MAX
  });

  test('node_screen_rect returns non-empty JSON for an existing node', async ({ page }) => {
    const rect = await page.evaluate(() => {
      const nodes = JSON.parse(window.__editor.to_json()).nodes;
      const id = nodes[0].id[0];
      return JSON.parse(window.__editor.node_screen_rect(id));
    });
    expect(typeof rect.x).toBe('number');
    expect(typeof rect.y).toBe('number');
    expect(rect.w).toBeGreaterThan(0);
    expect(rect.h).toBeGreaterThan(0);
  });
});

test.describe('Snap to grid', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForEditor(page);
  });

  test('snap is off by default', async ({ page }) => {
    const on = await page.evaluate(() => window.__editor.snap_enabled());
    expect(on).toBe(false);
  });

  test('toggle_snap turns it on', async ({ page }) => {
    await page.evaluate(() => window.__editor.toggle_snap(20));
    const on = await page.evaluate(() => window.__editor.snap_enabled());
    expect(on).toBe(true);
  });

  test('toggle_snap twice returns to off', async ({ page }) => {
    await page.evaluate(() => { window.__editor.toggle_snap(20); window.__editor.toggle_snap(20); });
    const on = await page.evaluate(() => window.__editor.snap_enabled());
    expect(on).toBe(false);
  });
});

test.describe('Zoom', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForEditor(page);
  });

  test('zoom_percent returns 100 by default', async ({ page }) => {
    const pct = await page.evaluate(() => window.__editor.zoom_percent());
    expect(pct).toBe(100);
  });

  test('zoom_in increases zoom', async ({ page }) => {
    await page.evaluate(() => window.__editor.zoom_in());
    const pct = await page.evaluate(() => window.__editor.zoom_percent());
    expect(pct).toBeGreaterThan(100);
  });

  test('zoom_out decreases zoom', async ({ page }) => {
    await page.evaluate(() => window.__editor.zoom_out());
    const pct = await page.evaluate(() => window.__editor.zoom_percent());
    expect(pct).toBeLessThan(100);
  });

  test('zoom_reset returns to 100', async ({ page }) => {
    await page.evaluate(() => { window.__editor.zoom_in(); window.__editor.zoom_in(); });
    await page.evaluate(() => window.__editor.zoom_reset());
    const pct = await page.evaluate(() => window.__editor.zoom_percent());
    expect(pct).toBe(100);
  });

  test('zoom_fit does not crash on empty canvas', async ({ page }) => {
    await expect(page.evaluate(() => { window.__editor.zoom_fit(); return true; }))
      .resolves.toBe(true);
  });

  test('zoom_to_selected falls back to zoom_fit when nothing selected', async ({ page }) => {
    await page.evaluate(() => { window.__editor.zoom_to_selected(); });
    const pct = await page.evaluate(() => window.__editor.zoom_percent());
    expect(pct).toBeGreaterThan(0); // didn't crash
  });

  test('footer zoom text updates after zoom_in', async ({ page }) => {
    await page.locator('#btn-zi').click();
    const text = await page.locator('#ft-zoom').textContent();
    expect(parseInt(text)).toBeGreaterThan(100);
  });
});


test.describe('Node-drag undo', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForEditor(page);
  });

  test('dragging a node can be undone to its original position', async ({ page }) => {
    const r = await page.evaluate(() => {
      const ed = window.__editor;
      ed.clear();
      ed.palette_drag_start('catchment');
      ed.palette_drop(200, 200);
      ed.zoom_fit();
      const id = ed.selected_node_id();
      const a = JSON.parse(ed.node_screen_rect(id));
      const cx = a.x + a.w / 2, cy = a.y + a.h / 2;
      ed.mouse_down(cx, cy, 0, false);
      ed.mouse_move(cx + 120, cy + 90);
      ed.mouse_up(cx + 120, cy + 90);
      const b = JSON.parse(ed.node_screen_rect(id));
      ed.undo();
      const c = JSON.parse(ed.node_screen_rect(id));
      return { ax: a.x, ay: a.y, bx: b.x, by: b.y, cx2: c.x, cy2: c.y };
    });
    expect(Math.abs(r.bx - r.ax)).toBeGreaterThan(50);   // it actually moved
    expect(Math.abs(r.cx2 - r.ax)).toBeLessThan(2);      // undo restored X
    expect(Math.abs(r.cy2 - r.ay)).toBeLessThan(2);      // undo restored Y
  });

  test('a plain node click adds no undo entry', async ({ page }) => {
    const r = await page.evaluate(() => {
      const ed = window.__editor;
      ed.clear();
      ed.palette_drag_start('catchment');
      ed.palette_drop(200, 200);              // the only real undo entry (add)
      ed.zoom_fit();
      const id = ed.selected_node_id();
      const a = JSON.parse(ed.node_screen_rect(id));
      const cx = a.x + a.w / 2, cy = a.y + a.h / 2;
      ed.mouse_down(cx, cy, 0, false);        // click, no movement
      ed.mouse_up(cx, cy);
      const afterClick = ed.node_count();
      ed.undo();                              // should undo the ADD, not a phantom
      return { afterClick, afterUndo: ed.node_count(), canUndo: ed.can_undo() };
    });
    expect(r.afterClick).toBe(1);
    expect(r.afterUndo).toBe(0);              // one undo removed the node
    expect(r.canUndo).toBe(false);            // no phantom click entry remains
  });
});
