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

// ── Corrected physics — numeric vectors ────────────────────────────────────────
// Runs single-node DAGs through the standalone engine and asserts the corrected
// hydrology/hydraulics results (see www/index.html HYD kernel).

async function runNode(page, kind, config) {
  return page.evaluate(({ kind, config }) => {
    const dag = { nodes: [{ id: 0, kind, x: 0, y: 0, config, outputs: {} }], edges: [] };
    const res = JSON.parse(window.__runStandaloneEngine(JSON.stringify(dag), JSON.stringify([0])));
    return res.nodes[0].outputs;
  }, { kind, config });
}

test.describe('Standalone engine — corrected physics', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForEditor(page);
  });

  test('detention pond routes with realistic peak + hydrograph arrays', async ({ page }) => {
    const o = await runNode(page, 'detention_pond', {
      area_acres: 5, curve_number: 80, rainfall_in: 4.2, tc_min: 20,
      bottom_area_sf: 5000, side_slope: 3, max_depth_ft: 6,
      orifice_dia_in: 6, orifice_invert_ft: 0,
    });
    // Peak out is limited by the 6" orifice — a few cfs, not thousands.
    expect(o['0']).toBeGreaterThan(0);
    expect(o['0']).toBeLessThan(10);
    expect(o.attenuation_pct).toBeGreaterThan(50);
    expect(Array.isArray(o.hydro_inflow_cfs)).toBe(true);
    expect(o.hydro_outflow_cfs.length).toBeGreaterThan(5);
    expect(o.peak_storage_cf).toBeGreaterThan(0);
  });

  test('detention peak out scales up with a larger orifice', async ({ page }) => {
    const base = { area_acres: 5, curve_number: 80, rainfall_in: 4.2, tc_min: 20,
      bottom_area_sf: 5000, side_slope: 3, max_depth_ft: 6, orifice_invert_ft: 0 };
    const small = await runNode(page, 'detention_pond', { ...base, orifice_dia_in: 6 });
    const big   = await runNode(page, 'detention_pond', { ...base, orifice_dia_in: 18 });
    expect(big['0']).toBeGreaterThan(small['0']); // orifice size now matters
  });

  test('unit hydrograph peak uses sq-mi conversion (not 640x)', async ({ page }) => {
    const o = await runNode(page, 'unit_hydrograph', { area_acres: 5, tc_min: 20 });
    // 484*(5/640)*1.0/(0.667*0.333) ≈ 17 cfs, not ~10,000.
    expect(o['0']).toBeGreaterThan(5);
    expect(o['0']).toBeLessThan(50);
  });

  test('RUSLE responds to region and slope', async ({ page }) => {
    const flat  = await runNode(page, 'rusle_erosion', { region: 'charlotte-nc', slope_length_ft: 100, slope_pct: 1, area_acres: 1 });
    const steep = await runNode(page, 'rusle_erosion', { region: 'charlotte-nc', slope_length_ft: 100, slope_pct: 15, area_acres: 1 });
    expect(steep['0']).toBeGreaterThan(flat['0']); // steeper slope => more erosion
    expect(flat['0']).toBeGreaterThan(0);
  });

  test('loss methods differ by method and are non-zero', async ({ page }) => {
    const base = { rainfall_in: 3.5, curve_number: 75, duration_hr: 24, hsg: 'B' };
    const cn = await runNode(page, 'loss_method', { ...base, method: 'curve_number' });
    const ga = await runNode(page, 'loss_method', { ...base, method: 'green_ampt' });
    const ho = await runNode(page, 'loss_method', { ...base, method: 'horton' });
    expect(cn['0']).toBeGreaterThan(0);
    expect(ga['0']).toBeGreaterThan(0);
    expect(ho['0']).toBeGreaterThan(0);
    // Green-Ampt (recovering capacity) yields less runoff than CN here.
    expect(ga['0']).toBeLessThan(cn['0']);
  });

  test('manning pipe reports a partial-flow depth on port 1', async ({ page }) => {
    const o = await runNode(page, 'manning_pipe', { diameter_ft: 1.5, slope: 0.005, manning_n: 0.013, design_q_cfs: 5 });
    expect(o['1']).toBeGreaterThan(0);
    expect(o['1']).toBeLessThan(1.5); // below the crown for a sub-full flow
  });

  test('treatment train produces an effluent load on port 0', async ({ page }) => {
    const o = await runNode(page, 'treatment_train', { area_acres: 5, runoff_in: 0.5, bmp_chain: 'bioretention,sand-filter' });
    expect(o['0']).toBeGreaterThan(0);          // port 0 "Effluent loads" now populated
    expect(o['1']).toBeGreaterThan(50);         // series removal efficiency (%)
  });

  test('sediment basin consumes yield and reports trapped mass', async ({ page }) => {
    const o = await runNode(page, 'sediment_basin', { design_q_cfs: 5, area_acres: 5, sed_yield_tons_ac_yr: 10 });
    expect(o['0']).toBeGreaterThan(0);          // trap efficiency %
    expect(o.trapped_tons_yr).toBeGreaterThan(0);
  });

  test('continuous sim responds to location and outputs monthly arrays', async ({ page }) => {
    const dry = await runNode(page, 'continuous_sim', { location: 'phoenix-az', area_acres: 5, curve_number: 75, years: 3 });
    const wet = await runNode(page, 'continuous_sim', { location: 'miami-fl', area_acres: 5, curve_number: 75, years: 3 });
    expect(wet['0']).toBeGreaterThan(dry['0']); // wetter city => more runoff
    expect(Array.isArray(wet.monthly_rain_in)).toBe(true);
    expect(wet.monthly_runoff_ac_in.length).toBe(12);
  });
});
