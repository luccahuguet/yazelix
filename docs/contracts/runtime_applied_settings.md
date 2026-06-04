# Runtime-Applied Settings Contract

## Summary

Yazelix settings are not broadly live-applied during an active session. Each schema-known setting has an apply mode that describes when a saved value becomes active and which runtime owner, if any, may refresh it.

The config UI, doctor, and future `yzx_control` save/apply flows should use one shared vocabulary instead of ad hoc restart messages.

## Apply Mode Vocabulary

| Mode | User label | Meaning |
| --- | --- | --- |
| `live` | Takes effect now | The saved value is active in the current process without a pane or tool refresh |
| `live_with_pane_refresh` | Takes effect now | The saved value is pushed to a running Yazelix-owned pane/plugin owner during save and takes effect in the current session after that owner acknowledges the refresh |
| `generated_runtime_refresh` | Takes effect after generated config refresh | The saved value requires Yazelix to regenerate managed runtime files and restart or reload the affected tool |
| `tab_session_restart` | Takes effect after Yazelix restart | The saved value affects pane startup, layout, Zellij session behavior, or launch-time environment and is active only after a fresh Yazelix window starts |
| `shell_terminal_restart` | Takes effect after Yazelix restart | The saved value affects terminal profiles, terminal-native config, or shell startup and is active only after Yazelix starts a fresh window/environment |
| `package_home_manager_activation` | Takes effect after Home Manager switch | The active source is outside the editable runtime session and requires package rebuild, Home Manager activation, or editing the Home Manager module input |
| `never_live` | Not applicable | The surface is a native/import/generated ownership boundary and must not be mutated as a live apply side effect |

`live` is intentionally narrow. Most current settings are either launch-time inputs, generated-config inputs, or pane/plugin runtime inputs that need an explicit refresh owner.

## Save And Apply Semantics

1. Saving validates and persists the semantic setting before any runtime refresh is attempted.
2. Runtime refresh failure does not silently roll back the saved value. The caller receives per-field status showing that the value is saved but pending Yazelix restart, pane reopen, generated config refresh, or Home Manager activation.
3. Live refresh must be explicit and owner-scoped. The first accepted runtime owner is the pane orchestrator, reached through a versioned pipe/control message rather than by rewriting plugin state files directly.
4. A stale session snapshot, missing plugin permission, missing pipe, or plugin version mismatch makes live refresh fail closed with restart guidance.
5. Generated runtime files under Yazelix state/data are outputs. Applying a setting may regenerate them through Yazelix materializers, but the UI must not ask users to edit those files.
6. Native config files outside Yazelix ownership are never modified as a save/apply side effect. Native integration status remains governed by `docs/contracts/native_config_integration_status.md`.
7. Home Manager-owned settings are read-only from the config UI. Their apply mode is `package_home_manager_activation` because the editable source is the Home Manager module, not the runtime settings file.

## Current Schema Inventory

| Settings | Apply mode | Notes |
| --- | --- | --- |
| `core.debug_mode` | `tab_session_restart` | Active scripts and plugins do not share a live debug flag owner |
| `core.skip_welcome_screen`, `core.show_macchina_on_welcome`, `core.game_of_life_cell_style`, `core.welcome_style`, `core.welcome_duration_seconds` | `tab_session_restart` | Welcome behavior is launch-time behavior |
| `helix.external` | `tab_session_restart` | Existing editor panes keep their launched Helix binary/runtime pair |
| `editor.command` | `tab_session_restart` | Existing editor panes keep their launched command |
| `editor.hide_sidebar_on_file_open` | `tab_session_restart` | Can move to `live_with_pane_refresh` only after the opener/session owner supports config reload |
| `workspace.left_sidebar.command`, `workspace.left_sidebar.args`, `workspace.left_sidebar.width_percent`, `workspace.right_sidebar.command`, `workspace.right_sidebar.args`, `workspace.right_sidebar.width_percent` | `tab_session_restart` | Sidebar layout and command are pane-startup inputs |
| `shell.default_shell` | `shell_terminal_restart` | Existing shells keep their process and startup environment |
| `terminal.config_mode`, `terminal.transparency` | `shell_terminal_restart` | Terminal config changes apply to newly launched terminal processes, or to Home Manager activation when Home Manager owns the source |
| `zellij.disable_tips`, `zellij.pane_frames`, `zellij.rounded_corners`, `zellij.support_kitty_keyboard_protocol`, `zellij.theme`, `zellij.default_mode`, `zellij.keybindings`, `zellij.native_keybindings` | `tab_session_restart` | These affect generated Zellij config or session-level behavior that is not safely reloadable in place |
| `zellij.popup_commands`, `zellij.custom_popups` | `tab_session_restart` | Applied to generated `yzpp` popup specs and active only after the generated Zellij config is refreshed and Yazelix restarts |
| `zellij.popup_width_percent`, `zellij.popup_height_percent` | `generated_runtime_refresh` | Applied to generated `yzpp` popup specs and active only after the generated Zellij config is refreshed and Yazelix restarts |
| `zellij.screen_saver_enabled`, `zellij.screen_saver_idle_seconds`, `zellij.screen_saver_style` | `live_with_pane_refresh` | Applied to the running pane orchestrator through the versioned runtime-config reload pipe |
| `zellij.widget_tray`, `zellij.tab_label_mode`, `zellij.codex_usage_display`, `zellij.codex_usage_periods`, `zellij.claude_usage_display`, `zellij.claude_usage_periods`, `zellij.opencode_go_usage_display`, `zellij.opencode_go_usage_periods`, `zellij.custom_text` | `generated_runtime_refresh` | Status-bar structure and usage rendering stay generated-runtime scoped until there is an explicit status-bar config reload owner |
| `yazi.command`, `yazi.ya_command`, `yazi.plugins`, `yazi.theme`, `yazi.sort_by` | `generated_runtime_refresh` | Requires managed Yazi config regeneration and a fresh Yazi/sidebar process |

When Home Manager owns the active settings source, every editable semantic setting is effectively `package_home_manager_activation` from the config UI perspective.

## Owner Boundary Tradeoffs

Apply modes follow the runtime owner, not the old implementation path. A setting can use `live_with_pane_refresh` only when the running owner of that behavior exposes a bounded reload protocol and can acknowledge success or failure.

Extraction and child-repo ownership can deliberately move a setting to a less-live apply mode. That is acceptable when the new owner removes main-repo lifecycle code or creates a smaller reusable boundary, but the config UI and command output must report the cost directly instead of pretending the saved value is active.

The popup settings are the canonical example. `zellij.popup_width_percent` and `zellij.popup_height_percent` used to fit the pane-orchestrator live-refresh bucket when the pane orchestrator owned popup lifecycle. Popup panes are `yzpp`-managed plugin specs, so geometry changes are generated Zellij config changes and become active after generated config refresh plus Yazelix restart.

## First Live Slice

The first safe live-apply implementation slice is restricted to bounded pane-orchestrator runtime fields:

- `zellij.screen_saver_enabled`
- `zellij.screen_saver_idle_seconds`
- `zellij.screen_saver_style`

These values are scalar or enum settings already owned by Yazelix runtime behavior. They do not require native config mutation, generated sidecar editing by users, shell restart, terminal restart, or Home Manager activation.

## Runtime Propagation

The first live-apply mechanism is:

1. `yzx_control` validates the semantic settings write and computes changed field apply modes.
2. `yzx_control` writes the canonical settings snapshot.
3. For `live_with_pane_refresh` fields, `yzx_control` sends `reload_runtime_config` to the pane orchestrator with schema version `1`, the generated Zellij config generation, and the bounded runtime config values.
4. The pane orchestrator validates the message, updates its in-memory runtime config, and returns an explicit success or failure.
5. The caller reports per-field status. Mixed results are visible; partial apply is not treated as success.

Generated-runtime refreshes should use existing Rust materializers and should return explicit restart guidance for the affected tool. They should not be smuggled through pane-orchestrator live state.

`yzx config set`, `yzx config unset`, and config UI saves use the contract apply mode after a successful write. For `generated_runtime_refresh` settings, Yazelix regenerates managed runtime state through the existing materializers, reports the affected tool owner such as Yazi or Zellij, and tells users whether to reopen the affected pane or restart Yazelix. A materialization failure does not undo the saved setting; it is returned as a visible save/apply error with the underlying materializer code and remediation.

## Status Bar Refresh Decision

Status-bar settings remain `generated_runtime_refresh`.

| Setting family | Runtime owner | Why it is not live-applied |
| --- | --- | --- |
| `zellij.widget_tray` | Yazelix passes typed runtime bar config to `yazelix_zellij_bar_widget render-yazelix-runtime`, then inserts the child-rendered runtime KDL template output into generated layout KDL before zjstatus starts | Changing the tray changes the loaded zjstatus `format_right` string and command-widget placeholders, not just command output |
| `zellij.tab_label_mode` | Yazelix passes the tab-label mode to the child runtime KDL template renderer | Changing tab label mode changes loaded zjstatus tab format configuration |
| `zellij.custom_text` | Yazelix passes custom text to the child runtime KDL template renderer | Changing the value changes a pre-rendered static segment in the plugin block |
| `zellij.codex_usage_display`, `zellij.codex_usage_periods`, `zellij.claude_usage_display`, `zellij.claude_usage_periods`, `zellij.opencode_go_usage_display`, `zellij.opencode_go_usage_periods` | `yzx_control` command widgets render provider/cache text from the launch session config snapshot | The command output can refresh on its interval, but the display policy and periods come from the active session snapshot; changing those settings without a new snapshot would make saved-versus-active state ambiguous |

Zellij plugin configuration is supplied to a plugin at load time. The current zjstatus pipe protocol can rerun command widgets, send notifications, or update pipe-widget content, but it does not replace the loaded module config, widget map, tab formats, or command-widget definitions. The pane orchestrator also must not proxy these settings into zjstatus because that would create a hidden second owner for bar configuration.

Save-time behavior for these fields is therefore:

1. Save the semantic setting.
2. Regenerate managed Zellij runtime files through the normal materializer.
3. Report that the saved value is pending a Yazelix restart.
4. If regeneration fails, keep the saved setting and report `generated_config_refresh_failed` with remediation.

Moving any of these fields to `live_with_pane_refresh` requires a status-bar owner that can acknowledge a versioned reload and define stale-generation, permission, partial-apply, and rollback behavior. Acceptable future owners are a dedicated Yazelix status-bar plugin or an upstream-supported zjstatus configuration reload protocol; a broad Zellij reconfigure call or pane-orchestrator side effect is not sufficient.

## Config UI Status Copy

The config UI should display saved-versus-active status with these labels:

- `now`
- `after pane reopen`
- `after Yazelix restart`
- `after Home Manager switch`
- `not applicable`

The UI should prefer specific remediation text from native-config status when a setting is read-only or generated, such as editing the Home Manager module or importing native config explicitly.

## Risks

- A stale session snapshot can make a live message target the wrong tab or plugin generation
- Zellij plugin permissions can block pipe delivery or state updates
- Runtime sidecar drift can make generated files differ from the saved semantic settings
- Partial apply failures can leave some settings active and others pending unless the caller reports per-field status
- Broad command or keybinding reloads can break active sessions, so command surfaces and semantic keymaps stay restart-only until a narrower contract exists

## Verification

- `yzx dev rust test config_ui`
- `yzx dev rust test doctor_commands`
- `yzx dev rust test zellij_materialization`
- `cargo test --manifest-path ../yazelix-zellij-pane-orchestrator/Cargo.toml`
- `nix build .#runtime --override-input yazelixZellijPaneOrchestrator ../yazelix-zellij-pane-orchestrator --no-link`
- `yzx_repo_validator validate-contracts`
