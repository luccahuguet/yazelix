# Standalone Yazelix Zellij Bar Distribution

## Summary

`yazelix_zellij_bar` is the standalone package and crate name for the Yazelix-branded Zellij status-bar preset. The public repository is `luccahuguet/yazelix-zellij-bar`.

The supported distribution shape is a zjstatus preset and bundle. It is not a zjstatus fork, not a first-party Zellij WASM plugin, and not a separate configuration framework.

## Package Shape

The child flake package is `github:luccahuguet/yazelix-zellij-bar#yazelix_zellij_bar`. The main Yazelix repo forwards the same package as `.#yazelix_zellij_bar`.

It installs:

- `bin/yazelix_zellij_bar_widget`
- `share/yazelix_zellij_bar/zjstatus.wasm`
- `share/yazelix_zellij_bar/yazelix_zellij_bar.kdl`
- `share/yazelix_zellij_bar/yazelix_zellij_bar.template.kdl`
- `share/yazelix_zellij_bar/examples/custom_command_widgets.kdl`
- `share/yazelix_zellij_bar/examples/standalone_zellij_layout.kdl`
- `share/yazelix_zellij_bar/examples/yazelix_runtime_widgets.kdl`
- `share/doc/yazelix_zellij_bar/README.md`

`yazelix_zellij_bar.kdl` is a ready-to-use Zellij layout plugin block with a package-local `file:` URL for the installed `zjstatus.wasm`.

`yazelix_zellij_bar.template.kdl` keeps the wasm placeholder for users or packagers who want to substitute a different pinned zjstatus binary.

The example snippets are not alternate generated presets. They are small blocks users can copy into the plugin body for command widgets or full-Yazelix cached widgets.

There is no standalone configuration generator binary and no central `~/.config/yazelix_zellij_bar/config.toml`. KDL is the public configuration surface.

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

Generic widgets are widgets that work in plain Zellij with only zjstatus and optional child-owned helper commands.

The generic preset keeps common zjstatus placeholders available without Yazelix:

- `{mode}`
- `{tabs}`
- `{session}`
- `{datetime}`

`yazelix_zellij_bar` owns standalone rendering and runnable widget commands for widgets that can run from explicit facts, paths, or user-supplied provider tools without a Yazelix session:

- shell
- editor
- term
- custom text
- compact/full tab labels and bar layout policy
- upstream zjstatus terminal-bell tab styling without marker text
- pure tab activity label rendering from explicit `idle`, `busy`, or `alert`
  facts
- CPU and RAM stdout widgets
- Claude usage display, cache, lock/backoff, and tokenusage probing from explicit cache paths or XDG defaults
- Codex usage display, cache, lock/backoff, and tokenusage probing from explicit cache paths or XDG defaults
- OpenCode Go usage display, cache, lock/backoff, and database probing from explicit cache/database paths or XDG defaults

Users configure order through `format_left`, `format_center`, and `format_right`. Users configure colors and labels by editing the inline style tags and the mode/tab format keys in KDL.

AI widgets are provider-driven widgets. A standalone user may run `yazelix_zellij_bar_widget` commands directly in zjstatus command widgets. Yazelix must not make provider commands mandatory in the generic preset.

Yazelix-specific widgets are widgets that depend on Yazelix runtime helpers, session snapshots, or cached facts:

- workspace
- pane-orchestrator all-tab activity snapshots and integrated activity rendering
- Yazelix-managed Claude/Codex/OpenCode Go cache path selection and session settings
- generated full-runtime command wiring

Those belong in the full Yazelix integration preset, not the generic default.

## Data-Source Contract

zjstatus command widgets should render short plain text on stdout. The preset owns style markup in KDL.

Expensive provider commands should be cached or throttled outside zjstatus. The generic standalone preset should not poll AI providers by default.

Standalone users can use the same widget contract without Yazelix by using `yazelix_zellij_bar_widget` commands directly in zjstatus command widgets. The minimal standalone contract is explicit paths/env in, styled text out; `yazelix_zellij_bar` must not require `~/.config/yazelix`, `~/.local/share/yazelix`, `yzx_control`, or launch-scoped Yazelix cache paths for non-workspace widget behavior.

## Main Runtime Consumption

The full Yazelix runtime consumes the `yazelix_zellij_bar` child package command surface for integrated zjstatus plugin-block rendering, simple fact widgets, CPU/RAM, cached provider usage widgets, and tab-label formatting helpers. Integrated layout materialization calls `yazelix_zellij_bar_widget render-yazelix-runtime` with typed runtime bar config; the child renders its runtime KDL template and Yazelix inserts the returned plugin block. The integrated template keeps zjstatus `{tabs}` as the default live tab source because it is event-driven by Zellij `TabUpdate` events and supports upstream terminal-bell styling without a Yazelix command-widget tab strip.

The standalone package installs `zjstatus.wasm` from the child repo's pinned `zjstatus` flake input. The main Yazelix flake makes `yazelixZellijBar.inputs.zjstatus` follow the main repo's `zjstatus` input when forwarding `.#yazelix_zellij_bar`, so the forwarded standalone package uses the same selected pin as the integrated Yazelix runtime.

The main runtime ships `configs/zellij/plugins/zjstatus.wasm` from the locked `zjstatus` package output for integrated Zellij layouts.

Yazelix must not use zjstatus's URL-based automated installation for the integrated runtime. The locally cloned upstream wiki at `/home/flexnetos/FlexNetOS/src/upstream/dj95/zjstatus.wiki/1-‐-Installation.md` documents `plugin location="https://github.com/dj95/zjstatus/releases/latest/download/zjstatus.wasm"` as an automated path, but also warns that Zellij can corrupt the download when multiple tabs fetch the plugin concurrently. Yazelix therefore treats `file:` paths to the packaged `zjstatus.wasm` as the release contract.

Yazelix keeps these integration-only responsibilities:

- launch-scoped status-cache paths
- refresh command scheduling
- session snapshot hydration
- pane-orchestrator all-tab activity snapshot transport
- `yzx_control` transport
- generated layout policy for full Yazelix sessions
- workspace facts until a generic fallback exists

## Refresh Ownership

Standalone `luccahuguet/yazelix-zellij-bar` updates own the standalone package pin:

- update the child repo's `zjstatus` flake input
- run `nix build .#yazelix_zellij_bar`
- run `cargo test`
- publish the child commit

Main Yazelix runtime updates own the integrated runtime pin:

- update the main repo's `zjstatus` flake input
- run the normal `yzx dev update` flow and package validation so the runtime consumes the new locked `zjstatus` package artifact
- update the `yazelixZellijBar` flake input when the child package contract changes

Do not manually copy `zjstatus.wasm` between the main repo and the child repo. Both package surfaces consume their selected `zjstatus` flake input as an artifact.

## Current Limit

zjstatus layout blocks do not provide a native include or variable layer. The current distribution therefore favors one small generic preset and copyable KDL snippets over a growing family of generated files.

Raw KDL remains the escape hatch for lower-level zjstatus keys.

The pinned zjstatus tabs widget renders each tab from Zellij `TabInfo`
placeholders, including native terminal-bell fields. The integrated Yazelix
runtime additionally configures `tab_activity_pipe_name` so the pinned tabs
widget can merge all-tabs activity state into each live `TabInfo.name` label
without renaming native Zellij tabs. Both the generic standalone preset and the
integrated Yazelix runtime keep `{tabs}` for live tab identity, focus,
creation/deletion updates, click handling, and style-only terminal-bell state.
The `yazelix_zellij_bar_widget tabs` command is a renderer probe for the all-tab
activity snapshot contract, not the default integrated tab strip.

## Verification

- `nix build .#yazelix_zellij_bar`
- `cargo test` in `luccahuguet/yazelix-zellij-bar`
- `yzx dev update --yes --activate none` for a main runtime input refresh
- `yzx_repo_validator validate-contracts`

## Traceability

- Defended by: `yzx_repo_validator validate-contracts`
