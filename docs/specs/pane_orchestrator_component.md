# Pane Orchestrator Component

## Summary

The Zellij pane orchestrator is an internal Yazelix component that owns session-local pane behavior which cannot be represented cleanly in Nushell alone. It lives in the monorepo as Rust source plus a tracked wasm artifact, and the rest of Yazelix should talk to it through one explicit pipe-command seam.

## Why

The orchestrator now owns several high-value UX paths: popup/menu transient panes, managed editor/sidebar focus, layout-family changes, workspace retargeting, workspace terminal opening, and active sidebar Yazi identity. Those paths used to look scattered because Rust code, Nushell client wrappers, generated Zellij config, and runtime wrapper scripts each carried part of the behavior.

This spec defines the component boundary without extracting it to a separate repository. The plugin and its consumers still change together, so extraction would add versioning cost without improving the current product.

## Scope

- Rust source under `rust_plugins/zellij_pane_orchestrator/`
- Tracked runtime artifact at `configs/zellij/plugins/yazelix_pane_orchestrator.wasm`
- Nushell client transport in `nushell/scripts/integrations/zellij.nu`
- Runtime wasm sync and permission-cache ownership in `nushell/scripts/setup/zellij_plugin_paths.nu`
- Generated Zellij keybind/config wiring in `configs/zellij/yazelix_overrides.kdl`
- Transient-pane wrapper scripts under `nushell/scripts/zellij_wrappers/`
- Focused tests in `nushell/scripts/dev/test_zellij_plugin_contracts.nu`, `nushell/scripts/dev/test_yzx_generated_configs.nu`, `nushell/scripts/dev/test_yzx_workspace_commands.nu`, `nushell/scripts/dev/test_yzx_popup_commands.nu`, and Rust unit tests in the orchestrator crate

## Behavior

### Command Surface

The orchestrator accepts these pipe command names:

- `focus_editor`
- `focus_sidebar`
- `toggle_editor_sidebar_focus`
- `move_focus_left_or_tab`
- `move_focus_right_or_tab`
- `smart_reveal`
- `open_file`
- `set_managed_editor_cwd`
- `next_family`
- `previous_family`
- `toggle_sidebar`
- `register_sidebar_yazi_state`
- `get_active_sidebar_yazi_state`
- `get_active_tab_session_state`
- `retarget_workspace`
- `open_terminal_in_cwd`
- `open_workspace_terminal`
- `open_transient_pane`
- `toggle_transient_pane`
- `debug_editor_state`
- `debug_write_literal`
- `debug_send_escape`

Nushell must resolve user intent before calling the plugin. For workspace changes, the surviving mutation command is `retarget_workspace`; older split commands for "set workspace root" and "set workspace root plus focused pane cd" are intentionally not part of the component contract.

### Plugin Configuration

The plugin reads these configuration keys from Zellij plugin configuration:

- `runtime_dir`
- `popup_width_percent`
- `popup_height_percent`
- `widget_tray_segment`
- `custom_text_segment`
- `sidebar_width_percent`

`runtime_dir` is session-local plugin state. The generated Zellij config must set it on the loaded pane-orchestrator plugin instance for that session, and direct `MessagePlugin` bindings or `zellij action pipe` calls must target that loaded instance by alias instead of re-supplying `runtime_dir` on each message. Transient-pane payloads may still carry an explicit runtime override when the caller intentionally wants the wrapper launch to use a different runtime root. The geometry keys are percentages and default to the Rust-side transient-pane defaults when absent or outside the accepted runtime range.

### Runtime And Wrapper Paths

The plugin does not probe the local filesystem for wrapper discovery. It derives paths from `runtime_dir`:

- launcher: `shells/posix/yazelix_nu.sh`
- popup wrapper: `nushell/scripts/zellij_wrappers/yzx_popup_program.nu`
- menu wrapper: `nushell/scripts/zellij_wrappers/yzx_menu_popup.nu`

The session-loaded plugin instance is the runtime source of truth for direct keybind messages and Nushell transport calls. Those message paths must address the pane orchestrator by alias only so multiple Yazelix sessions can stay self-contained even when they were launched from different runtime roots.

### Pane Identity Invariants

The plugin is authoritative for managed pane identity inside a Zellij session:

- managed editor panes are terminal panes titled `editor`
- managed sidebar panes are terminal panes titled `sidebar`
- transient popup panes are floating terminal panes titled `yzx_popup` or launched through `yzx_popup_program.nu`
- transient menu panes are floating terminal panes titled `yzx_menu` or launched through `yzx_menu_popup.nu`
- Nushell and Rust share one explicit transient-pane identity contract for popup/menu title, wrapper marker, and wrapper path; wrapper-mode env ownership stays on the Nushell side of that seam
- plugin panes, exited panes, and unrelated user panes must not count as managed editor/sidebar/transient panes
- workspace state is tab-local and explicit workspace retargets are stronger than bootstrap state
- sidebar Yazi identity returned from `retarget_workspace` is active-tab state, not a session-global cache scan

### Build And Sync Ownership

Rust source edits are not live until the wasm is rebuilt and synced:

```bash
yzx dev build_pane_orchestrator --sync
```

The sync step updates the tracked wasm, the stable runtime wasm path, and the generated Zellij config. After a synced plugin change, validate in a fresh Yazelix session or with `yzx restart`; do not treat `cargo test` alone as proof of live plugin behavior.

## Non-goals

- Extracting the orchestrator into a separate repository now
- Making the wasm a public reusable plugin API
- Reintroducing legacy workspace pipe commands for compatibility
- Moving Yazi adapter command execution into Rust
- Moving all generated Zellij config ownership into Rust

## Acceptance Cases

1. A maintainer can list the plugin pipe commands, plugin configuration keys, runtime wrapper assumptions, and pane identity invariants from this spec without searching across Rust, Nushell, and KDL files.
2. Workspace mutation flows use `retarget_workspace` as the single live pipe command for tab workspace changes.
3. Popup and menu transient panes use the same helperless pane-orchestrator launch/toggle model and the same geometry configuration.
4. Sidebar/editor focus, workspace-terminal opening, transient-pane toggles, and layout-family commands remain keyed through generated Zellij `MessagePlugin` entries that target the loaded plugin alias without re-supplying `runtime_dir`.
5. Rust source changes are rebuilt and synced before runtime behavior is claimed fixed.

## Verification

- `cargo test --manifest-path rust_plugins/zellij_pane_orchestrator/Cargo.toml --lib`
- `yzx dev build_pane_orchestrator --sync`
- `nu nushell/scripts/dev/test_zellij_plugin_contracts.nu`
- `nu nushell/scripts/dev/test_yzx_generated_configs.nu`
- `nu nushell/scripts/dev/test_yzx_commands.nu`
- `nu nushell/scripts/dev/validate_specs.nu`

## Traceability

- Bead: `yazelix-1q7x`
- Defended by: `nu nushell/scripts/dev/test_zellij_plugin_contracts.nu`
- Defended by: `nu nushell/scripts/dev/test_yzx_generated_configs.nu`
- Defended by: `nu nushell/scripts/dev/test_yzx_commands.nu`

## Open Questions

- Should the orchestrator source get a narrower internal module layout once the v16 Rust rewrite starts touching adjacent session logic?
- Should debug-only pipe commands stay in the long-term command surface, or move behind an explicit debug prefix?
