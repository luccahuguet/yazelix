# Yazelix Zellij Pane Orchestrator Extraction

## Summary

`yazelix-zellij-pane-orchestrator` is the extracted source owner for the Zellij pane-orchestrator plugin that began inside Yazelix.

The plugin must be usable in a plain Zellij setup without installing Yazelix. Yazelix may consume the same artifact and add first-party integration around it, but Yazelix-only helpers, runtime paths, and keybinding policy are extensions rather than hidden requirements for core plugin behavior.

## Names

- public project: `yazelix-zellij-pane-orchestrator`
- Rust package: `yazelix-zellij-pane-orchestrator`
- Rust crate: `yazelix_zellij_pane_orchestrator`
- public wasm artifact: `yazelix_zellij_pane_orchestrator.wasm`
- Yazelix runtime artifact name: `yazelix_pane_orchestrator.wasm`
- public Zellij alias example: `yazelix-zellij-pane-orchestrator`
- Yazelix internal alias: `yazelix_pane_orchestrator`

The Yazelix runtime artifact name and internal alias remain stable for existing generated layouts. The extracted project owns source and public artifact naming.

## Boundary

The Yazelix repository consumes the plugin through a pinned external package artifact boundary. It must not keep duplicate plugin source or copied first-party wasm as a fallback.

Yazelix must not keep:

- copied plugin Rust source
- copied first-party plugin wasm as source provenance
- duplicate contract modules that exist only to mirror extracted source internals
- compatibility wrappers that preserve the old in-repo ownership model
- local-only cache, path, or checkout assumptions as substitutes for a real source/artifact boundary

## Standalone Zellij Usage

A non-Yazelix user can build the plugin and load it with a normal Zellij plugin alias:

```kdl
plugins {
    yazelix-zellij-pane-orchestrator location="file:/absolute/path/to/yazelix_zellij_pane_orchestrator.wasm" {
        screen_saver_enabled false
    }
}
```

Standalone users can then target that alias from keybindings or pipes:

```kdl
bind "Alt Shift H" {
    MessagePlugin "yazelix-zellij-pane-orchestrator" {
        name "toggle_sidebar"
    }
}
```

Standalone mode must not require `YAZELIX_RUNTIME_DIR`, `YAZELIX_SESSION_CONFIG_PATH`, `yzx_control`, or Yazelix-managed config paths for core pane orchestration behavior.

## Public Pipe API

The public pipe API is the command name carried in the Zellij plugin message or pipe name.

Core standalone commands:

- `focus_editor`
- `focus_sidebar`
- `toggle_editor_sidebar_focus`
- `toggle_editor_right_sidebar_focus`
- `move_focus_left_or_tab`
- `move_focus_right_or_tab`
- `next_family`
- `previous_family`
- `toggle_sidebar`
- `hide_sidebar`
- `get_active_tab_session_state`
- `open_terminal_in_cwd`
- `open_workspace_terminal`

Yazelix integration commands:

- `smart_reveal`
- `open_file`
- `set_managed_editor_cwd`
- `register_sidebar_yazi_state`
- `register_ai_pane_activity`
- `retarget_workspace`
- `reload_runtime_config`

Maintainer/debug commands:

- `maintainer_debug_editor_state`
- `debug_write_literal`
- `debug_send_escape`

The extracted README must document payload shape, response tokens, and standalone support level for each public command. Debug commands must be documented as unsupported for ordinary users.

## Works Without Yazelix

The plugin must support these behaviors without Yazelix installed:

- tracking tab-local editor/sidebar pane identity from Zellij pane state
- focusing editor and sidebar panes
- toggling editor/left-sidebar and editor/right-agent focus
- moving horizontal focus or crossing tab boundaries
- switching layout families through Zellij layout state
- opening a terminal in the current tab/workspace when no Yazelix wrapper is required
- returning a versioned active-tab session-state JSON payload
- optional screen-saver orchestration when configured with a standalone command

Standalone tests and fixtures must prove these behaviors do not require Yazelix runtime paths.

## Yazelix-Only Integration

These behaviors remain Yazelix integration because they depend on Yazelix runtime conventions or companion tools:

- opening files in the Yazelix-managed editor wrapper
- synchronizing Yazelix sidebar Yazi state
- retargeting a Yazelix workspace and emitting Yazi DDS commands
- refreshing Yazelix runtime config live
- writing Yazelix status-bar cache facts through `yzx_control`
- using the Yazelix `yzx screen` renderer for the screen saver
- preserving generated Yazelix keybinding/action policy

Yazelix integration may call the standalone plugin API, but standalone plugin users must not need these integration paths for core behavior.

## Package Contract

The external source owner builds the public wasm package artifact. Yazelix consumes that package into the runtime plugin path instead of treating a copied artifact as durable source ownership.

The package boundary must prove:

- external source Git commit
- external source remote
- source project name
- stable package artifact path
- main runtime placement at `configs/zellij/plugins/yazelix_pane_orchestrator.wasm`

Local development uses an explicit flake override against an adjacent checkout. The committed main repo must not use a local path input.

## Rust Dependency Gate

Production crates:

- keep `zellij-tile`
- keep `serde`
- keep `serde_json`

Dev-only crates:

- none required for the extraction boundary

Build in-house:

- pipe command routing
- pane selection and tab-local state contracts
- standalone fixtures
- sync-stamp rendering

Rejected by default:

- RPC frameworks
- plugin command frameworks
- schema/codegen helpers
- compatibility crates that keep Yazelix source ownership alive

Packaging impact:

- Yazelix ships the packaged child wasm in its runtime tree
- source ownership moves to the external project
- Yazelix Rust ownership budget must remove or ratchet the old source family after source deletion

## No-go Conditions

Do not extract if the implementation would:

- preserve a duplicate copy of plugin source in Yazelix
- require Yazelix runtime paths for core standalone behavior
- add wrappers that hide the old in-repo ownership model instead of deleting it
- weaken pane/session behavior tests into help-output or command-discovery checks
- make ordinary Yazelix runtime startup depend on a mutable local checkout

## Verification

Standalone source owner:

- `cargo fmt --check`
- `cargo test --lib`
- `nix build github:luccahuguet/yazelix-zellij-pane-orchestrator#yazelix_zellij_pane_orchestrator --no-link`

Yazelix consumer:

- `nix build .#yazelix --override-input yazelixZellijPaneOrchestrator ../yazelix-zellij-pane-orchestrator --no-link`
- `cargo run -p yazelix_maintainer --bin yzx_repo_validator -- validate-rust-ownership-budget`
- focused Yazelix workspace/session validation for pipe commands and generated layout aliases

## Traceability

- Defended by: `docs/contracts/pane_orchestrator_component.md`
- Defended by: `docs/contracts/pane_orchestrator_tab_local_session_state_seam.md`
- Defended by: `packaging/mk_runtime_tree.nix`
- Defended by: `rust_core/yazelix_maintainer/src/workspace_session_contract.rs`
