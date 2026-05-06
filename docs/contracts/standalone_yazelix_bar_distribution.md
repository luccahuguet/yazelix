# Standalone Yazelix Bar Distribution

## Summary

`yazelix_bar` is the selected standalone name for the Yazelix-branded Zellij status-bar preset.

Alternatives considered:

- `yazelix_zjstatus_bar`
- `yazelix_status_bar`

The first supported distribution shape is a zjstatus preset and bundle, not a zjstatus fork and not a first-party Zellij WASM plugin.

## Package Shape

The flake package is `.#yazelix_bar`.

It installs:

- `share/yazelix_bar/zjstatus.wasm`
- `share/yazelix_bar/yazelix_bar.kdl`
- `share/yazelix_bar/yazelix_bar.template.kdl`
- `share/doc/yazelix_bar/README.md`

`yazelix_bar.kdl` is a ready-to-use Zellij layout plugin block with a package-local `file:` URL for the bundled `zjstatus.wasm`.

`yazelix_bar.template.kdl` keeps the wasm placeholder for users or packagers who want to substitute a different pinned zjstatus binary.

## Generic Default

The standalone default must work without a Yazelix runtime.

It may include:

- mode
- tabs
- session
- datetime
- Yazelix-branded colors
- tab overflow and compact status-bar policy

It must not require:

- `yzx`
- `yzx_control`
- `IN_YAZELIX_SHELL`
- the pane orchestrator
- Yazelix status-cache files
- Nushell
- tokenusage
- full Yazelix installation

## Widget Boundary

Generic widgets are widgets that work in plain Zellij with only zjstatus.

AI widgets are provider-driven widgets. A standalone user may configure command widgets for Claude, Codex, OpenCode Go, or another provider, but Yazelix must not make those commands mandatory in the generic preset.

Yazelix-specific widgets are widgets that depend on Yazelix runtime helpers, session snapshots, or cached facts:

- workspace
- cursor
- Claude usage through Yazelix cache readers
- Codex usage through Yazelix cache readers
- OpenCode Go usage through Yazelix cache readers
- CPU/RAM through Yazelix runtime scripts

Those belong in the full Yazelix integration preset, not the generic default.

## Data-Source Contract

zjstatus command widgets should render short plain text on stdout. The preset owns style markup in KDL.

Expensive provider commands should be cached or throttled outside zjstatus. The generic standalone preset should not poll AI providers by default.

Inside full Yazelix, AI widgets should keep using cached status-widget commands such as `yzx_control zellij status-cache-widget codex_usage` so the bar does not create high-frequency provider or pane-orchestrator pressure.

## Main Runtime Consumption

The full Yazelix runtime consumes the `rust_core/yazelix_bar` crate for widget-tray and tab-label rendering.

The standalone package consumes the same vendored `configs/zellij/plugins/zjstatus.wasm` source as the full runtime. Maintainers refresh that wasm through the normal repo update workflow rather than copying standalone artifacts manually.

If the standalone preset needs generated variants, the next step is to add a generator that consumes the same `yazelix_bar` renderer. Do not maintain a second hand-copied generated artifact set.

## Verification

- `nix build .#yazelix_bar`
- `yzx dev rust test yazelix_bar`
- `yzx_repo_validator validate-contracts`

## Traceability

- Defended by: `yzx_repo_validator validate-contracts`
