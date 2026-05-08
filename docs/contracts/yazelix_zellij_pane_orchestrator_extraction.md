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

The Yazelix repository consumes the plugin through a pinned external source/artifact boundary. It must not keep duplicate plugin source as a fallback.

Yazelix may keep:

- the tracked runtime wasm artifact when package/runtime distribution needs an in-repo artifact
- a sync stamp that records the external source hash and tracked wasm hash
- validators that check the consumed artifact and Yazelix integration contract
- integration code that sends documented pipe messages to the plugin

Yazelix must not keep:

- copied plugin Rust source
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
bind "Alt y" {
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
- toggling editor/sidebar focus
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

## Build And Sync Contract

The external source owner builds the public wasm artifact. Yazelix syncs that artifact into the tracked runtime artifact path when packaging or maintainer validation needs an in-repo wasm.

The sync stamp must prove:

- external source content hash
- public wasm artifact hash
- tracked Yazelix runtime wasm hash
- source project name

If external source is unavailable, Yazelix validators may still verify the tracked wasm against the sync stamp. Commands that build or sync the plugin must fail fast with an actionable error that names the required external checkout or configured source path.

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

- Yazelix may keep shipping a tracked wasm artifact
- source ownership moves to the external project
- Yazelix Rust ownership budget must remove or ratchet the old `pane_orchestrator_plugin` source family after source deletion

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
- build `wasm32-wasip1` release artifact

Yazelix consumer:

- `cargo run -p yazelix_maintainer --bin yzx_repo_validator -- validate-pane-orchestrator-sync`
- `cargo run -p yazelix_maintainer --bin yzx_repo_validator -- validate-rust-ownership-budget`
- focused Yazelix workspace/session validation for pipe commands and generated layout aliases

## Traceability

- Defended by: `docs/contracts/pane_orchestrator_component.md`
- Defended by: `docs/contracts/pane_orchestrator_tab_local_session_state_seam.md`
- Defended by: `rust_core/yazelix_maintainer/src/repo_plugin_build.rs`
- Defended by: `rust_core/yazelix_maintainer/src/workspace_session_contract.rs`
