# hydrocomplete-dag

[![CI](https://github.com/mf4633/hydrocomplete-dag/actions/workflows/ci.yml/badge.svg)](https://github.com/mf4633/hydrocomplete-dag/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-wasm32-orange.svg)](https://www.rust-lang.org/)

WASM model builder for HydroComplete — visual hydrology/hydraulics DAG editor used by Civil 3D (`HC_NETWORK_EDIT`) and Open CAD Studio.

Drag hydrology, hydraulics, and water-quality nodes onto a canvas, wire their
ports into a directed acyclic graph, configure each node, and run the model to
see results and charts. Export a diagram (SVG) or an analysis report (HTML).

## Features

- **20 node types** across Hydrology, Hydraulics/Routing, Water Quality, and Sink.
- **Live DAG editing** — drag-to-connect with cycle/duplicate/occupied-port
  rejection, rubber-band select, multi-select copy/paste, auto-layout,
  snap-to-grid, minimap, and 50-step undo/redo.
- **Two run modes** — a built-in standalone JavaScript engine for quick browser
  use, or the full Civil 3D / Open CAD Studio engine when embedded in a host.
- **Export** — HTML analysis report and SVG vector diagram; models save/load as
  JSON and autosave to `localStorage`.

## Layout

- `src/` — Rust/WASM engine (canvas, palette, templates, undo)
- `www/index.html` — standalone editor shell
- `www/pkg/` — prebuilt `wasm-pack` output (committed for CI and bundle staging)

## Build

```bash
rustup target add wasm32-unknown-unknown
wasm-pack build --target web --out-dir www/pkg --release
```

`--release` runs `wasm-opt` automatically when [binaryen](https://github.com/WebAssembly/binaryen)
is available. `www/pkg/` is committed, so re-run this and commit the result
whenever the Rust in `src/` changes, otherwise the browser bundle and the
E2E tests will run against stale code.

Serve locally (uses `serve.js`, which sends the correct `application/wasm`
MIME type):

```bash
npm run serve        # http://127.0.0.1:7777
```

## Tests

Rust unit tests:

```bash
cargo test
```

Playwright E2E specs under `tests/` exercise the DAG editor UI against the
committed `www/pkg/` bundle:

```bash
npm ci
npx playwright install chromium
npm test
```

Both suites run in CI (`.github/workflows/ci.yml`) on every push and pull
request. HydroComplete OCS integration tests live in
`opencad-hydrocomplete-plugin/tests/frontend/`.

## Contributing

Issues and pull requests are welcome — see [CONTRIBUTING.md](CONTRIBUTING.md).
When adding or changing a node, keep the config keys and output ports in the
standalone engine (`www/index.html`) in sync with `src/nodes.rs`.

## License

[MIT](LICENSE) © HydroComplete

## Support

If HydroComplete's DAG editor is useful to you, please consider [sponsoring the project](https://github.com/sponsors/mf4633). Your support helps keep it maintained.