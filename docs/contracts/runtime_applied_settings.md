# Runtime-Applied Settings Contract

## Summary

Yazelix settings are not broadly live-applied during an active session. Each schema-known setting has an apply mode that describes when a saved value becomes active and which runtime owner, if any, may refresh it.

The config UI, doctor, and future `yzx_control` save/apply flows should use one shared vocabulary instead of ad hoc restart messages.

## Apply Mode Vocabulary

| Mode | User label | Meaning |
| --- | --- | --- |
| `live` | Applies now | The saved value is active in the current process without a pane or tool refresh |
| `live_with_pane_refresh` | Applies after pane refresh | The saved value can be pushed to a running Yazelix-owned pane/plugin owner and takes effect in the current session after that owner acknowledges the refresh |
| `generated_runtime_refresh` | Saved, refresh generated config | The saved value requires Yazelix to regenerate managed runtime files and restart or reload the affected tool |
| `tab_session_restart` | Saved, restart this tab/session | The saved value affects pane startup, layout, Zellij session behavior, or launch-time environment and is active only after a fresh Yazelix tab/session |
| `shell_terminal_restart` | Saved, restart terminal/shell | The saved value affects terminal profiles, terminal-native config, or shell startup and is active only in newly launched terminal or shell processes |
| `package_home_manager_activation` | Saved, activate package/Home Manager | The active source is outside the editable runtime session and requires package rebuild, Home Manager activation, or editing the Home Manager module input |
| `never_live` | Not live-applicable | The surface is a native/import/generated ownership boundary and must not be mutated as a live apply side effect |

`live` is intentionally narrow. Most current settings are either launch-time inputs, generated-config inputs, or pane/plugin runtime inputs that need an explicit refresh owner.

## Save And Apply Semantics

1. Saving validates and persists the semantic setting before any runtime refresh is attempted.
2. Runtime refresh failure does not silently roll back the saved value. The caller receives per-field status showing that the value is saved but pending restart, pane refresh, generated config refresh, terminal restart, or Home Manager activation.
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
| `helix.runtime_path` | `tab_session_restart` | Existing editor panes keep their launched runtime |
| `editor.command` | `tab_session_restart` | Existing editor panes keep their launched command |
| `editor.hide_sidebar_on_file_open` | `tab_session_restart` | Can move to `live_with_pane_refresh` only after the opener/session owner supports config reload |
| `editor.sidebar_width_percent`, `editor.sidebar_command`, `editor.sidebar_args` | `tab_session_restart` | Sidebar layout and command are pane-startup inputs |
| `shell.default_shell` | `shell_terminal_restart` | Existing shells keep their process and startup environment |
| `terminal.terminals`, `terminal.config_mode`, `terminal.transparency` | `shell_terminal_restart` | Terminal config changes apply to newly launched terminal processes, or to Home Manager activation when Home Manager owns the source |
| `zellij.disable_tips`, `zellij.pane_frames`, `zellij.rounded_corners`, `zellij.support_kitty_keyboard_protocol`, `zellij.theme`, `zellij.default_mode`, `zellij.keybindings` | `tab_session_restart` | These affect generated Zellij config or session-level behavior that is not safely reloadable in place |
| `zellij.popup_program` | `tab_session_restart` | A future pane-orchestrator reload may narrow this, but command changes are excluded from the first live slice |
| `zellij.popup_width_percent`, `zellij.popup_height_percent` | `live_with_pane_refresh` | Accepted first live slice once the pane-orchestrator reload pipe exists |
| `zellij.screen_saver_enabled`, `zellij.screen_saver_idle_seconds`, `zellij.screen_saver_style` | `live_with_pane_refresh` | Accepted first live slice once the pane-orchestrator reload pipe exists |
| `zellij.widget_tray`, `zellij.tab_label_mode`, `zellij.codex_usage_display`, `zellij.claude_usage_display`, `zellij.claude_usage_periods`, `zellij.opencode_go_usage_display`, `zellij.opencode_go_usage_periods`, `zellij.custom_text` | `generated_runtime_refresh` | Can move to `live_with_pane_refresh` only after the status bar has an explicit refresh owner |
| `yazi.command`, `yazi.ya_command`, `yazi.plugins`, `yazi.theme`, `yazi.sort_by` | `generated_runtime_refresh` | Requires managed Yazi config regeneration and a fresh Yazi/sidebar process |

When Home Manager owns the active settings source, every editable semantic setting is effectively `package_home_manager_activation` from the config UI perspective.

## First Live Slice

The first safe live-apply implementation slice is restricted to bounded pane-orchestrator runtime fields:

- `zellij.popup_width_percent`
- `zellij.popup_height_percent`
- `zellij.screen_saver_enabled`
- `zellij.screen_saver_idle_seconds`
- `zellij.screen_saver_style`

These values are scalar or enum settings already owned by Yazelix runtime behavior. They do not require native config mutation, generated sidecar editing by users, shell restart, terminal restart, or Home Manager activation.

## Runtime Propagation

The first live-apply mechanism should be:

1. `yzx_control` validates the semantic settings write and computes changed field apply modes.
2. `yzx_control` writes the canonical settings snapshot.
3. For `live_with_pane_refresh` fields, `yzx_control` sends a versioned pane-orchestrator pipe message with the changed values and the current session/config generation.
4. The pane orchestrator validates the message, updates its in-memory runtime config, and returns an explicit success or failure.
5. The caller reports per-field status. Mixed results are visible; partial apply is not treated as success.

Generated-runtime refreshes should use existing Rust materializers and should return explicit restart guidance for the affected tool. They should not be smuggled through pane-orchestrator live state.

## Config UI Status Copy

The config UI should display saved-versus-active status with these labels:

- `Applies now`
- `Applies after pane refresh`
- `Saved, refresh generated config`
- `Saved, restart this tab/session`
- `Saved, restart terminal/shell`
- `Saved, activate package/Home Manager`
- `Not live-applicable`

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
- `yzx_repo_validator validate-contracts`
