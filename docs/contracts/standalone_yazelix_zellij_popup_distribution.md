# Standalone Yazelix Zellij Popup Distribution

## Summary

`yazelix_zellij_popup` is the selected standalone name for the Yazelix-branded Zellij floating TUI popup plugin.

The first supported distribution shape is a small Zellij WASM plugin plus a plain-Zellij KDL example. It is not the full Yazelix pane orchestrator and does not include workspace, sidebar, editor, command-palette, config UI, or Home Manager behavior.

## Package Shape

The flake package is `.#yazelix_zellij_popup`.

It installs:

- `share/yazelix_zellij_popup/yazelix_zellij_popup.wasm`
- `share/yazelix_zellij_popup/examples/gitui.kdl`
- `share/yazelix_zellij_popup/examples/gitui.template.kdl`
- `share/doc/yazelix_zellij_popup/README.md`

`gitui.kdl` is a ready-to-use plain-Zellij example with a package-local `file:` URL for the standalone plugin wasm.

`gitui.template.kdl` keeps the wasm placeholder for users or packagers who want to substitute another pinned plugin path.

## Request Contract

The standalone plugin listens for `MessagePlugin` pipe messages named `transient_popup`.

The payload is strict JSON with:

- `action`: `toggle`, `open`, `focus`, or `close`
- `spec.id`
- `spec.pane_title`
- optional `spec.command_marker`
- `spec.command` as argv
- optional `spec.cwd`
- `spec.width_percent` and `spec.height_percent`
- optional invocation `args`
- optional invocation `cwd`

The command is argv, not a shell string. Width and height must be integers from `1` through `100`. Blank ids, pane titles, command paths, and command markers are invalid.

`toggle` opens the command when the managed pane is missing, focuses it when it exists but is not focused, and closes it while hiding the floating layer when the managed pane is focused.

## Permission Contract

The standalone plugin requests only the Zellij permissions needed for the generic popup flow:

- `ReadApplicationState`
- `ChangeApplicationState`
- `OpenTerminalsOrPlugins`
- `RunCommands`
- `ReadCliPipes`

Those permissions cover active-pane discovery, opening command panes, focusing and closing managed panes, and receiving pipe requests.

## Yazelix Adapter Boundary

The standalone plugin owns only generic popup identity, request parsing, geometry validation, command-pane opening, focus, close, and duplicate prevention.

The full Yazelix runtime keeps ownership of:

- `yzx popup`
- popup/menu/config semantic kinds
- `settings.jsonc`
- Home Manager options
- generated Yazelix keybindings
- runtime wrapper scripts
- workspace-root cwd snapshots
- sidebar Yazi refresh hooks

The standalone package must not depend on `yzx`, Yazelix runtime roots, Yazi, Helix, Home Manager, or Yazelix generated state.

## Verification

- `nix build .#yazelix_zellij_popup`
- `cargo test --manifest-path rust_plugins/zellij_pane_orchestrator/Cargo.toml transient_pane_contract`
- `yzx_repo_validator validate-contracts`

## Traceability

- Defended by: `cargo test --manifest-path rust_plugins/zellij_pane_orchestrator/Cargo.toml transient_pane_contract`
- Defended by: `yzx_repo_validator validate-contracts`
