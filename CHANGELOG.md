# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Changed — standalone engine hydrology correctness
A shared `HYD` kernel (US-customary, mirrors the authoritative C# `DagExecutor`)
replaces the ad-hoc per-node math. All results now respond to their inputs:
- **SCS peak flow** (`detention_pond`, `pond_chain`, `unit_hydrograph`) — the `484`
  coefficient now uses **area in square miles** with `Tp ≈ 0.667·Tc`, fixing a
  ~640× overestimate (a 5-ac site went from ~4,000 cfs to ~38 cfs).
- **Detention pond** — real **Modified-Puls** routing (frustum stage-storage,
  orifice + emergency-weir stage-outflow, storage-indication routing) with
  inflow/outflow hydrograph arrays for the chart. Peak outflow now responds to
  orifice size, invert, and pond geometry.
- **RUSLE** — `R` from a regional isoerodent map and `LS` from slope length &
  steepness (McCool), so `region`/`slope_length_ft`/`slope_pct` finally matter.
- **Loss Method** — Green-Ampt, Horton, initial+constant, and constant-rate are
  implemented (driven by an SCS Type II design hyetograph) alongside curve number;
  `method` and `hsg` now change the result.
- **Continuous Sim** — regional annual rainfall + monthly CN accounting; honors
  `location` and `years`; outputs monthly arrays and corrected `ac-in/yr` units.
- **Sediment Basin** — trap efficiency from overflow-rate settling; consumes the
  soil-loss input / `sed_yield` to report trapped mass.
- **Output-port gaps filled** — Manning Pipe now reports flow depth (port 1);
  Treatment BMP/Train report effluent load (port 0); BMP Sizing sizes from the
  treated volume (WQV ÷ ponding depth) instead of a flat 5%.
- Added Playwright numeric test vectors covering the corrected physics.

### Fixed
- **Node-drag undo was a no-op.** The undo snapshot was cloned in `mouse_up`
  *after* `mouse_move` had already mutated node positions, so undo restored the
  post-drag state. The pre-drag snapshot is now captured at drag start and pushed
  only when a position actually changed — a plain select-click no longer adds a
  redundant undo entry either.
- **Config panel targeted the wrong node.** `NodeId` serializes as a bare integer
  (serde newtype), so reading `node.id['0']` in the UI was `undefined` and every
  config edit / chart was applied to node id 0. All id reads now go through a
  shared `normId()` normalizer that accepts both the WASM (bare integer) and C#
  host (`{"0": n}`) shapes.
- **Empty HTML report and broken SVG export.** The same `n.id[0]` assumption made
  report rows never match and drew every SVG edge from the first node with
  colliding clip-path ids. Fixed via `normId()`.
- **Multi-node copy/paste dropped internal edges.** `paste_group` parsed
  `["id"]["0"]` from bare-integer ids, collapsing every id to 0. Now parses the
  newtype directly (with a fallback to the object shape).
- **GVF profile used SI gravity (9.81)** inside US-customary equations; corrected
  to 32.2 ft/s².
- **Detention monthly chart** scaled runoff bars by the rainfall-only maximum,
  letting them overflow the plot; the axis now spans both series.
- **Chart peak marker** could panic on `NaN` (`partial_cmp().unwrap()`); now uses
  `total_cmp`.

### Security
- Escaped all untrusted strings (node labels, config keys/values, loaded-JSON
  outputs) in the node inspector and HTML report generator, closing stored-XSS
  vectors delivered via a crafted model file.

### Added
- **☕ Support button** in the editor toolbar linking to the project's funding
  page. The target is a single `SUPPORT_URL` constant — swap it to a Stripe
  Payment Link (`https://buy.stripe.com/…`) to take contributions directly.
- Exposed config fields the engine already read but the schema omitted, so user
  inputs stop being silently ignored: Pond Chain `Tc` and `Ponds in series`,
  GVF Profile `Reach length`, Treatment Train `BMP chain`.
- Robustness: guarded the file-load handler and wrapped WASM init with a visible
  error state.
- Project infrastructure: MIT `LICENSE`, Cargo/npm package metadata,
  `CONTRIBUTING.md`, this changelog, README features/badges/license section, and
  Rust tests locking the `NodeId` serialization invariant.

### Removed
- Two unreachable duplicate `switch` cases in the standalone engine
  (`water_quality_volume`, `rusle_erosion`).
