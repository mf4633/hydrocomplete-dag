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

Serve locally:

```bash
npx --yes serve www -l 4173
```

## Tests

Playwright specs under `tests/` (DAG editor UI). HydroComplete OCS integration tests live in `opencad-hydrocomplete-plugin/tests/frontend/`.