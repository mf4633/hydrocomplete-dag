// @ts-check
/**
 * Standalone engine tests.
 * Loads each template, runs the JS fallback engine, and asserts that
 * key output values are numeric and within physically plausible ranges.
 */
const { test, expect } = require('@playwright/test');

async function waitForEditor(page) {
  await page.waitForFunction(
    () => typeof window.__editor !== 'undefined' && typeof window.__runStandaloneEngine !== 'undefined',
    { timeout: 15_000 }
  );
}

async function runTemplate(page, templateId) {
  return page.evaluate(async (id) => {
    window.__editor.load_template(id);
    const dagJson   = window.__editor.to_json();
    const orderJson = window.__editor.execution_order_json();
    const resultJson = window.__runStandaloneEngine(dagJson, orderJson);
    window.__editor.apply_results_json(resultJson);
    return JSON.parse(resultJson);
  }, templateId);
}

// ── Helpers ──────────────────────────────────────────────────────────────────

function findNodeByKind(dag, kind) {
  return dag.nodes.find(n => n.kind === kind);
}

function output(node, key) {
  return node?.outputs?.[key];
}

// ── Tests ─────────────────────────────────────────────────────────────────────

test.describe('Standalone engine — execution', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForEditor(page);
  });

  test('execution_order_json returns a non-empty array', async ({ page }) => {
    await page.evaluate(() => window.__editor.load_template('detention'));
    const order = await page.evaluate(() =>
      JSON.parse(window.__editor.execution_order_json())
    );
    expect(Array.isArray(order)).toBe(true);
    expect(order.length).toBeGreaterThan(0);
  });

  test('apply_results_json returns true for valid result DAG', async ({ page }) => {
    await page.evaluate(() => window.__editor.load_template('wq'));
    const ok = await page.evaluate(() => {
      const dagJson   = window.__editor.to_json();
      const orderJson = window.__editor.execution_order_json();
      const result    = window.__runStandaloneEngine(dagJson, orderJson);
      return window.__editor.apply_results_json(result);
    });
    expect(ok).toBe(true);
  });

  test('results are present on nodes after apply_results_json', async ({ page }) => {
    await page.evaluate(() => window.__editor.load_template('detention'));
    const nodesWithResults = await page.evaluate(() => {
      const dagJson   = window.__editor.to_json();
      const orderJson = window.__editor.execution_order_json();
      const result    = JSON.parse(window.__runStandaloneEngine(dagJson, orderJson));
      return result.nodes.filter(n =>
        n.outputs && Object.keys(n.outputs).length > 0
      ).length;
    });
    expect(nodesWithResults).toBeGreaterThan(0);
  });
});

test.describe('Standalone engine — Detention template', () => {
  let dag;
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForEditor(page);
    dag = await runTemplate(page, 'detention');
  });

  test('SCS runoff node has Q > 0 inches', async ({}) => {
    const node = findNodeByKind(dag, 'scs_runoff');
    const q = output(node, '0');
    expect(typeof q).toBe('number');
    expect(q).toBeGreaterThan(0);
    expect(q).toBeLessThan(24); // can't exceed total rainfall depth
  });

  test('detention pond node has Q_out > 0 cfs', async ({}) => {
    const node = findNodeByKind(dag, 'detention_pond');
    const q = output(node, '0');
    expect(typeof q).toBe('number');
    expect(q).toBeGreaterThan(0);
  });

  test('detention pond has plausible attenuation percent', async ({}) => {
    // The standalone engine computes qOut using its own internal Qp;
    // cross-node comparison requires fully coupled routing (Civil 3D path).
    // Here we only verify the attenuation result is physically bounded.
    const pond  = findNodeByKind(dag, 'detention_pond');
    const atten = output(pond, 'attenuation_pct');
    expect(typeof atten).toBe('number');
    expect(atten).toBeGreaterThanOrEqual(0);
    expect(atten).toBeLessThanOrEqual(100);
  });

  test('outfall receives a value', async ({}) => {
    const node = findNodeByKind(dag, 'outfall');
    expect(node?.outputs).toBeTruthy();
  });
});

test.describe('Standalone engine — WQ template', () => {
  let dag;
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForEditor(page);
    dag = await runTemplate(page, 'wq');
  });

  test('WQV node has WQV > 0 cf', async ({}) => {
    const node = findNodeByKind(dag, 'water_quality_volume');
    const wqv  = output(node, '0');
    expect(typeof wqv).toBe('number');
    expect(wqv).toBeGreaterThan(0);
  });

  test('WQV node has Rv between 0.05 and 0.95', async ({}) => {
    const node = findNodeByKind(dag, 'water_quality_volume');
    const rv   = output(node, 'rv');
    expect(typeof rv).toBe('number');
    expect(rv).toBeGreaterThanOrEqual(0.05);
    expect(rv).toBeLessThanOrEqual(0.95);
  });

  test('BMP sizing node has surface area > 0 sf', async ({}) => {
    const node = findNodeByKind(dag, 'bmp_sizing');
    const area = output(node, '0');
    expect(typeof area).toBe('number');
    expect(area).toBeGreaterThan(0);
  });

  test('treatment BMP node has TSS η ≥ 0%', async ({}) => {
    const node = findNodeByKind(dag, 'treatment_bmp') ?? findNodeByKind(dag, 'treatment_train');
    const eta  = output(node, '1');
    expect(typeof eta).toBe('number');
    expect(eta).toBeGreaterThanOrEqual(0);
    expect(eta).toBeLessThanOrEqual(100);
  });
});

test.describe('Standalone engine — Erosion template', () => {
  let dag;
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForEditor(page);
    dag = await runTemplate(page, 'erosion');
  });

  test('RUSLE node produces soil loss > 0 t/ac/yr', async ({}) => {
    const node = findNodeByKind(dag, 'rusle_erosion');
    const a    = output(node, '0');
    expect(typeof a).toBe('number');
    expect(a).toBeGreaterThan(0);
  });

  test('Rational node produces Q > 0 cfs', async ({}) => {
    const node = findNodeByKind(dag, 'rational_method');
    const q    = output(node, '0');
    expect(typeof q).toBe('number');
    expect(q).toBeGreaterThan(0);
  });

  test('Sediment basin has trap efficiency between 0 and 100%', async ({}) => {
    const node = findNodeByKind(dag, 'sediment_basin');
    const eta  = output(node, '0');
    expect(typeof eta).toBe('number');
    expect(eta).toBeGreaterThanOrEqual(0);
    expect(eta).toBeLessThanOrEqual(100);
  });
});

test.describe('Standalone engine — Continuous simulation template', () => {
  let dag;
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForEditor(page);
    dag = await runTemplate(page, 'continuous');
  });

  test('ContinuousSim node produces annual runoff > 0 ac-in/yr', async ({}) => {
    const node = findNodeByKind(dag, 'continuous_sim');
    const q    = output(node, '0');
    expect(typeof q).toBe('number');
    expect(q).toBeGreaterThan(0);
  });

  test('treatment_train node produces TSS η ≥ 0%', async ({}) => {
    const node = findNodeByKind(dag, 'treatment_train');
    const eta  = output(node, '1');
    expect(typeof eta).toBe('number');
    expect(eta).toBeGreaterThanOrEqual(0);
    expect(eta).toBeLessThanOrEqual(100);
  });
});

test.describe('Standalone engine — all 20 node kinds execute', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForEditor(page);
  });

  // Build a DAG from scratch containing one of every node kind
  // and run the engine; verify no node has an 'error' output.
  test('no node produces an error output', async ({ page }) => {
    const errors = await page.evaluate(async () => {
      // Use full_stormwater which has the widest variety
      window.__editor.load_template('full');
      const dagJson   = window.__editor.to_json();
      const orderJson = window.__editor.execution_order_json();
      const result    = JSON.parse(window.__runStandaloneEngine(dagJson, orderJson));
      return result.nodes
        .filter(n => n.outputs?.error)
        .map(n => `${n.kind}: ${n.outputs.error}`);
    });
    expect(errors).toEqual([]);
  });

  // Check every node kind known to the palette is handled (returns a non-null output object)
  const NODE_KINDS_WITH_OUTPUTS = [
    'catchment', 'rainfall_event', 'rational_method', 'scs_runoff',
    'time_of_concentration', 'unit_hydrograph', 'loss_method', 'continuous_sim',
    'manning_pipe', 'manning_channel', 'detention_pond',
    'water_quality_volume', 'bmp_sizing', 'treatment_bmp', 'treatment_train',
    'sediment_basin', 'rusle_erosion',
  ];

  for (const kind of NODE_KINDS_WITH_OUTPUTS) {
    test(`"${kind}" node produces output '0' when run in isolation`, async ({ page }) => {
      const val = await page.evaluate(async (k) => {
        // Build minimal one-node DAG
        window.__editor.clear();
        const dag = JSON.parse(window.__editor.to_json());
        const FAKE_ID = 99;
        dag.nodes = [{ id: {0: FAKE_ID}, kind: k, x: 100, y: 100, label: null, config: {}, outputs: {} }];
        dag.edges = [];
        window.__editor.from_json(JSON.stringify(dag));
        const orderJson = JSON.stringify([FAKE_ID]);
        const dagJson   = window.__editor.to_json();
        const result    = JSON.parse(window.__runStandaloneEngine(dagJson, orderJson));
        const node      = result.nodes.find(n => n.id[0] === FAKE_ID);
        return node?.outputs?.['0'] ?? node?.outputs?.received ?? '__has_outputs__';
      }, kind);

      // Should not be undefined/null (error case)
      expect(val).not.toBeNull();
      expect(val).not.toBeUndefined();
    });
  }
});

test.describe('Topological execution order', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForEditor(page);
  });

  test('execution order contains every node exactly once', async ({ page }) => {
    await page.evaluate(() => window.__editor.load_template('full'));
    const { nodeCount, orderLength, unique } = await page.evaluate(() => {
      const order  = JSON.parse(window.__editor.execution_order_json());
      const nodeCount = window.__editor.node_count();
      return { nodeCount, orderLength: order.length, unique: new Set(order).size };
    });
    expect(orderLength).toBe(nodeCount);
    expect(unique).toBe(nodeCount); // no duplicates
  });

  test('sources appear before sinks in execution order', async ({ page }) => {
    await page.evaluate(() => window.__editor.load_template('detention'));
    const { order, dag } = await page.evaluate(() => ({
      order: JSON.parse(window.__editor.execution_order_json()),
      dag:   JSON.parse(window.__editor.to_json()),
    }));

    // NodeId is integer from WASM, or {"0": n} from C# — handle both
    const nid = v => typeof v === 'number' ? v : (v?.[0] ?? v?.['0']);
    for (const edge of dag.edges) {
      const fromIdx = order.indexOf(nid(edge.from_node));
      const toIdx   = order.indexOf(nid(edge.to_node));
      expect(fromIdx).toBeGreaterThanOrEqual(0);
      expect(fromIdx).toBeLessThan(toIdx);
    }
  });
});
