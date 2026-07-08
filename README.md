# hydrocomplete-dag

WASM model builder for HydroComplete — visual hydrology/hydraulics DAG editor used by Civil 3D (`HC_NETWORK_EDIT`) and Open CAD Studio.

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

## Support

If HydroComplete's DAG editor is useful to you, please consider [sponsoring the project](https://github.com/sponsors/mf4633). Your support helps keep it maintained.