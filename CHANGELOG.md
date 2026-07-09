# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Fixed
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
