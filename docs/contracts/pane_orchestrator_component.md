# Pane Orchestrator Component

## Summary

The Zellij pane orchestrator is a standalone Zellij plugin consumed by Yazelix through a child package wasm artifact. Its source lives in the external `yazelix-zellij-pane-orchestrator` project, and the rest of Yazelix talks to it through one explicit pipe-command seam.

## Why

The orchestrator owns several high-value UX paths: managed editor/left-sidebar focus, editor/right-agent focus, layout-family changes, workspace retargeting, workspace terminal opening, screen-saver launch, status-cache facts, and active sidebar Yazi identity. Popup, menu, and config UI panes are owned by the integrated `yzpp` plugin instead.

This contract defines the Yazelix integration boundary for the extracted plugin. The external project owns source and standalone behavior; Yazelix owns generated layouts, runtime packaging, and integration commands.

## Scope

- External Rust source in `yazelix-zellij-pane-orchestrator`
- Runtime package artifact at `configs/zellij/plugins/yazelix_pane_orchestrator.wasm`
- Yazelix control transport in `rust_core/yazelix_core/src/pane_orchestrator_client.rs` and `rust_core/yazelix_core/src/zellij_commands/pipe.rs`
- Runtime wasm placement and permission-cache ownership in `rust_core/yazelix_core/src/zellij_materialization.rs`
- Generated Zellij keybinding/config wiring from `rust_core/yazelix_core/src/action_registry.rs` and `zellij_materialization.rs`
- Focused tests in `rust_core/yazelix_core/tests/yzx_control_workspace_surface.rs`, `rust_core/yazelix_core/src/zellij_materialization.rs`, and Rust unit tests in the orchestrator crate

## Contract Items

#### POC-001
- Type: boundary
- Status: live
- Owner: pane orchestrator pipe-command seam
- Statement: The pane orchestrator is an internal component reached through one
  explicit pipe-command seam. Yazelix resolves user intent first; the plugin is
  not a second CLI/parser surface
- Verification: automated
  `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_control_workspace_surface`; automated
  `yzx_repo_validator validate-workspace-session-contract`

#### POC-002
- Type: ownership
- Status: live
- Owner: generated Zellij config plus loaded plugin instance
- Statement: The generated Zellij config sets `runtime_dir` on the loaded
  plugin instance, and direct keybind messages or `zellij action pipe` calls
  target that alias instead of re-supplying runtime identity on each message
- Verification: automated
  `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core plugin_block_carries_runtime_and_popup_contract_once`

#### POC-003
- Type: invariant
- Status: live
- Owner: plugin pane-state model
- Statement: Managed editor/sidebar pane identity is plugin-owned. The
  orchestrator may recognize `yzpp`-managed popup/menu/config pane titles for
  status classification, but it does not own their lifecycle.
  Plugin panes, exited panes, and unrelated user panes must not count as
  managed Yazelix panes
- Verification: automated
  `cargo test --manifest-path ../yazelix-zellij-pane-orchestrator/Cargo.toml --lib`;
  automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_control_workspace_surface`

#### POC-004
- Type: ownership
- Status: live
- Owner: workspace mutation and sidebar snapshot contract
- Statement: `retarget_workspace` is the single live pipe command for workspace
  mutation, and any returned sidebar Yazi identity is active-tab state rather
  than a cache scan or session-global guess
- Verification: automated
  `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core yzx_control_cwd_retargets_workspace_and_syncs_sidebar`; automated
  `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core workspace_session::tests`

#### POC-005
- Type: boundary
- Status: live
- Owner: child package workflow
- Statement: Rust source edits are not live until the pane orchestrator child
  package is built and Yazelix is tested against that package through a lock or
  explicit local flake override. `cargo test` alone does not prove live plugin
  behavior
- Verification: automated `nix build .#runtime`; manual local
  `--override-input yazelixZellijPaneOrchestrator` smoke test

## Behavior

### Command Surface

The orchestrator accepts these pipe command names:

- `focus_editor`
- `focus_sidebar`
- `toggle_editor_sidebar_focus`
- `toggle_editor_right_sidebar_focus`
- `move_focus_left_or_tab`
- `move_focus_right_or_tab`
- `smart_reveal`
- `open_file`
- `set_managed_editor_cwd`
- `next_family`
- `previous_family`
- `toggle_sidebar`
- `hide_sidebar`
- `register_sidebar_yazi_state`
- `get_active_tab_session_state`
- `retarget_workspace`
- `open_terminal_in_cwd`
- `open_workspace_terminal`
- `maintainer_debug_editor_state`
- `debug_write_literal`
- `debug_send_escape`

These command names are the plugin API. Keybindings are not plugin semantics; they are generated Zellij policy that sends `MessagePlugin` calls to the loaded `yazelix_pane_orchestrator` instance. Yazelix ships `Ctrl+y` for `toggle_editor_sidebar_focus`, `Ctrl+Shift+Y` for `toggle_editor_right_sidebar_focus`, and `Alt+Shift+H` for the `toggle_left_sidebar` action, which sends the plugin command `toggle_sidebar`; users may remap those keys without changing the plugin contract as long as they keep sending the same command names.

Yazelix control code must resolve user intent before calling the plugin. For workspace changes, the surviving mutation command is `retarget_workspace`; older split commands for "set workspace root" and "set workspace root plus focused pane cd" are intentionally not part of the component contract.

### Plugin Configuration

The plugin reads these configuration keys from Zellij plugin configuration:

- `runtime_dir`
- `screen_saver_enabled`
- `screen_saver_idle_seconds`
- `screen_saver_style`

`runtime_dir` is session-local plugin state. The generated Zellij config must set it on the loaded pane-orchestrator plugin instance for that session, and direct `MessagePlugin` bindings or `zellij action pipe` calls must target that loaded instance by alias instead of re-supplying `runtime_dir` on each message. Popup geometry belongs to the generated `yzpp` specs, not the pane-orchestrator plugin config.

The screen-saver keys are opt-in. When enabled, the plugin watches Zellij-wide input activity and opens a full-tab `yzx screen` command pane after the configured idle threshold. The plugin owns only inactivity/session orchestration; the `yzx screen` process remains the single renderer and animation contract.

### Runtime And Wrapper Paths

The plugin derives runtime-owned helper paths from `runtime_dir`:

- launcher: `shells/posix/yzx_cli.sh`

The session-loaded plugin instance is the runtime source of truth for direct keybind messages and Yazelix control transport calls. Those message paths must address the pane orchestrator by alias only so multiple Yazelix sessions can stay self-contained even when they were launched from different runtime roots.

### Pane Identity Invariants

The plugin is authoritative for managed pane identity inside a Zellij session:

- managed editor panes are terminal panes titled `editor`
- managed sidebar panes are terminal panes titled `sidebar`
- popup panes are `yzpp`-managed floating terminal panes titled `yzx_popup`
- menu panes are `yzpp`-managed floating terminal panes titled `yzx_menu`
- config UI panes are `yzpp`-managed floating terminal panes titled `yzx_config`
- the pane orchestrator may report popup/menu pane identity for status surfaces, but it does not open or close those panes
- plugin panes, exited panes, and unrelated user panes must not count as managed editor/sidebar/popup panes
- workspace state is tab-local and explicit workspace retargets are stronger than bootstrap state
- sidebar Yazi identity returned from `retarget_workspace` is active-tab state, not a session-global cache scan

### Package Ownership

Rust source edits are not live until the child package builds and Yazelix consumes that package:

```bash
nix build ../yazelix-zellij-pane-orchestrator#yazelix_zellij_pane_orchestrator --no-link
nix build .#runtime --override-input yazelixZellijPaneOrchestrator ../yazelix-zellij-pane-orchestrator --no-link
```

After the child commit is pushed, update the main Yazelix lock to that child revision and rerun the main runtime build without local overrides. After a plugin behavior change, validate in a fresh Yazelix session or with `yzx restart`; do not treat `cargo test` alone as proof of live plugin behavior.

## Non-goals

- Reintroducing legacy workspace pipe commands for compatibility
- Moving Yazi adapter command execution into Rust
- Moving all generated Zellij config ownership into Rust

## Acceptance Cases

1. A maintainer can list the plugin pipe commands, plugin configuration keys, runtime wrapper assumptions, and pane identity invariants from this contract without searching across Rust, Nushell, and KDL files.
2. Workspace mutation flows use `retarget_workspace` as the single live pipe command for tab workspace changes.
3. Popup, menu, and config UI panes are identified as `yzpp`-managed panes, not pane-orchestrator-opened panes.
4. Sidebar/editor focus, workspace-terminal opening, and layout-family commands remain keyed through generated Zellij `MessagePlugin` entries that target the loaded plugin alias without re-supplying `runtime_dir`.
5. Rust source changes are packaged and consumed by the main runtime before runtime behavior is claimed fixed.

## Verification

- `cargo test --manifest-path ../yazelix-zellij-pane-orchestrator/Cargo.toml --lib`
- `nix build ../yazelix-zellij-pane-orchestrator#yazelix_zellij_pane_orchestrator --no-link`
- `nix build .#runtime --override-input yazelixZellijPaneOrchestrator ../yazelix-zellij-pane-orchestrator --no-link`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_control_workspace_surface`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core zellij_materialization`
- `yzx_repo_validator validate-contracts`

## Traceability
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_control_workspace_surface`
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core zellij_materialization`
- Defended by: `yzx_repo_validator validate-workspace-session-contract`

## Open Questions

- Should the orchestrator source get a narrower internal module layout once the v16 Rust rewrite starts touching adjacent session logic?
- Should debug-only pipe commands stay in the long-term command surface, or move behind an explicit debug prefix?
