# Contributing to hydrocomplete-dag

Thanks for your interest in improving HydroComplete's model builder!

## Development setup

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
npm ci

# Build the WASM bundle into www/pkg
wasm-pack build --target web --out-dir www/pkg --release

# Serve the standalone editor
npm run serve      # http://localhost:4173
```

## Before you open a PR

```bash
cargo test        # Rust unit tests
npm test          # Playwright E2E
```

CI runs these plus a release build on every push and pull request.

## Project structure

| Path              | What it holds                                                  |
| ----------------- | -------------------------------------------------------------- |
| `src/dag.rs`      | DAG model — nodes, edges, topological order, JSON round-trip   |
| `src/nodes.rs`    | Node catalog: kinds, categories, ports, config-field schema    |
| `src/interaction.rs` | Mouse/selection/drag state machine                          |
| `src/renderer.rs` | Canvas rendering + hit testing                                 |
| `src/chart.rs`    | Per-node result charts                                         |
| `src/lib.rs`      | `DagEditor` — the `wasm_bindgen` surface                       |
| `www/index.html`  | Editor UI, standalone JS engine, SVG/report exporters          |

## Guidelines

- **Keep the schema and the engine in sync.** The standalone JS engine in
  `www/index.html` mirrors the authoritative C# `DagExecutor`. When you add or
  change a node, make sure the config keys it reads and the output ports it writes
  match `src/nodes.rs` (`config_fields` and `outputs`). Mismatches silently fall
  back to defaults.
- **Node ids** serialize as bare integers from WASM and as `{"0": n}` from the C#
  host. Always read them through the `normId()` helper (JS) or the `read_id` helper
  (Rust) rather than assuming one shape.
- **Rebuild `www/pkg`** whenever you change Rust code, and commit the result — it
  is the artifact the app and tests load.
- **Escape untrusted strings** (labels, config values, loaded JSON) before putting
  them into `innerHTML` or exported markup; use the `escapeHtml()` helper.

## Reporting issues

Please include the steps to reproduce, the model JSON if relevant, your browser,
and whether you were running standalone or embedded in Civil 3D / Open CAD Studio.
