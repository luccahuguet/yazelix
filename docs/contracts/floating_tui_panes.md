# Floating TUI Panes

## Summary

Yazelix should support one explicit popup flow for terminal UIs that are useful temporarily but do not deserve a persistent split. The first supported surface is `yzx popup`, backed by a user-configured `zellij.popup_program` that defaults to `lazygit`. The popup key should behave like a managed session surface rather than spawning disposable duplicates forever.

## Why

Yazelix already had a floating command-palette popup, but no coherent popup model for transient TUIs. That left tools like lazygit or AI agents either taking over the current pane or requiring ad hoc Zellij commands. A dedicated popup surface makes that behavior explicit and configurable without turning every pane into a floating-pane policy problem.

## Scope

- Add `yzx popup`
- Add `zellij.popup_program` to `settings.jsonc` / Home Manager
- Bind the configured popup to a dedicated key
- Keep the command-palette popup as a separate flow
- Reuse the configured `yzpp` popup model for popup, menu, and config UI panes
- Keep Yazelix-specific side effects, such as sidebar refresh, outside the plain
  popup contract through explicit hooks

## Contract Items

#### POP-001
- Type: behavior
- Status: live
- Owner: `yzx popup` plus `yzpp` popup config
- Statement: `yzx popup` resolves one argv list, not a shell string. The
  default popup program is `["lazygit"]`, and a per-invocation command override
  replaces that argv list for only the current popup
- Verification: automated `nu nushell/scripts/dev/test_yzx_popup_commands.nu`

#### POP-002
- Type: failure_mode
- Status: live
- Owner: popup config validation and render-plan owners
- Statement: `zellij.popup_width_percent` and
  `zellij.popup_height_percent` must be integers in the range `1..100`.
  Invalid values fail fast as config errors instead of being coerced silently
- Verification: automated `nu nushell/scripts/dev/test_yzx_popup_commands.nu`;
  validator `yzx_repo_validator validate-contracts`

#### POP-003
- Type: behavior
- Status: live
- Owner: popup cwd resolution plus `yzpp` raw request adapter
- Statement: Popup panes launch in the current tab workspace root when one is
  known, otherwise they fall back to the current shell directory
- Verification: automated `nu nushell/scripts/dev/test_yzx_popup_commands.nu`

#### POP-004
- Type: ownership
- Status: live
- Owner: configured `yzpp` popup lifecycle
- Statement: `Alt+t` toggles one managed popup pane instead of spawning
  duplicates forever, while `Alt+Shift+M` stays a separate command-palette flow
- Verification: automated `nu nushell/scripts/dev/test_yzx_popup_commands.nu`;
  automated `nu nushell/scripts/dev/test_zellij_plugin_contracts.nu`

#### POP-005
- Type: boundary
- Status: live
- Owner: popup standalone and integrated packaging boundary
- Statement: Plain-Zellij popup behavior belongs in the external
  `yazelix-zellij-popup` source repository. Yazelix packages the `yzpp.wasm`
  artifact and uses configured `yzpp` specs for popup, menu, and config UI
  panes. Yazelix Zellij Popup owns the standalone plugin, KDL-native popup
  specs, optional command hooks, the raw generated pipe contract, and
  plain-Zellij examples. The in-repo Yazelix pane orchestrator owns
  workspace/sidebar/editor/session state, not popup pane opening or closing
- Verification: validator `yzx_repo_validator validate-contracts`; external
  `yazelix-zellij-popup` gates `cargo test` and `nix build`

## Behavior

- `yzx popup` opens a floating Zellij pane using the configured `zellij.popup_program`.
- `zellij.popup_program` is an argv list, not a shell string.
- The default popup program is `["lazygit"]`.
- Popup geometry is user-configurable through `zellij.popup_width_percent` and `zellij.popup_height_percent`.
- Popup width and height percentages must be integers in the range `1..100`.
- The default popup width and height are both `90`.
- `yzx popup <command ...>` overrides the configured command for that invocation.
- The generated Yazelix `yzpp` specs own the stable pane identity, argv, cwd,
  runtime command path, geometry, and close hook for popup/menu/config panes.
- The popup launches in the current tab workspace root when available; otherwise it uses the current shell directory.
- Popup pane lifecycle is controlled by the popup keybinding and explicit
  `yzpp` `toggle` or `close` messages, not by child process exit.
- `Alt+t` opens one managed popup pane when it is missing, focuses it when it exists but is unfocused, and closes it when it is focused.
- When `Alt+t` closes the configured popup pane, Yazelix runs `yzx sidebar
  refresh` through an `on_close` hook so lazygit-style workflows refresh the
  managed Yazi sidebar.
- `Alt+Shift+M` opens the command-palette popup through `yzpp`.
- `Alt+Shift+C` opens the config UI popup through `yzpp`.
- Plain Zellij users get this capability through Yazelix Zellij Popup (`yzpp`): a reusable floating-pane toggle for configured TUI commands, stable pane identity, and duplicate-preventing focus/close behavior.
- The external Yazelix Zellij Popup plugin provides this capability without
  requiring Yazelix-specific runtime paths, wrappers, config keys, or sidebar
  refresh behavior; integrations can opt into generic `on_close` command hooks.
- The current Yazelix path remains canonical for the integrated product: `yzx
  popup`, `zellij.popup_program`, generated `yzpp` specs, and `yzx sidebar
  refresh` define supported Yazelix popup behavior.

## Standalone Boundary

The external Yazelix Zellij Popup user config surface keeps popup specs in plugin config and uses short `MessagePlugin` messages:

```kdl
plugins {
    yzpp location="file:/path/to/yzpp.wasm" {
        popups {
            gitui {
                command "gitui"
                pane_title "gitui_popup"
                command_marker "gitui"
                cwd "."
                width_percent 90
                height_percent 85
            }
        }
    }
}

load_plugins {
    yzpp
}

keybinds {
    normal {
        bind "Alt g" {
            MessagePlugin "yzpp" {
                name "toggle"
                payload "gitui"
            }
        }
    }
}
```

The `yzpp` raw pipe path still accepts generated JSON through `name "transient_popup"`, but that shape is not the recommended hand-written config surface.

## Non-goals

- General floating-pane support for every Yazelix action
- Converting all Yazi plugins to popup flows
- Background daemon management for long-running AI tools
- Reabsorbing Yazelix Zellij Popup source into Yazelix core
- Treating Yazelix wrapper paths, runtime env, or sidebar refresh behavior as a plain-Zellij API

## Acceptance Cases

1. When a user presses `Alt+t` inside Yazelix, the configured popup program opens in one managed floating pane instead of replacing an existing workspace pane.
2. When `zellij.popup_program` is changed to another argv list, `yzx popup` launches that program without requiring shell-string parsing.
3. When `zellij.popup_width_percent` and `zellij.popup_height_percent` are set to valid values from `1` to `100`, `yzx popup` launches the popup with those dimensions.
4. When popup width or height is configured outside the valid `1..100` range, Yazelix fails fast with a clear config error.
5. When `yzx popup` runs from a tab with an explicit workspace root, the popup uses that root as its cwd.
6. Repeated popup-key presses do not create duplicate popup panes; they focus or close the existing managed popup instead.
7. When `Alt+Shift+M` is used, the command palette still opens separately from the popup-program flow.
8. The extracted `yazelix-zellij-popup` source stays in its child repository while Yazelix packages and integrates its `yzpp.wasm` artifact.
9. The standalone plugin supports KDL-native configured popup specs and keeps raw JSON pipe requests only for generated integrations.
10. Full Yazelix docs and code identify `yzpp` as the popup/menu/config pane owner and the pane orchestrator as the workspace/sidebar/editor/session owner.

## Verification

- unit tests: popup command/cwd resolution helpers
- unit tests: popup geometry config parsing and validation
- unit tests: popup lifecycle identity in the pane orchestrator
- external `yazelix-zellij-popup` unit tests: KDL-native popup specs, optional
  `on_close` hooks, and raw generated pipe request compatibility
- external package gate: `nix build` in `yazelix-zellij-popup`
- integration tests: `yzx popup` routes generated popup requests to `yzpp`
  with a fake Zellij binary
- integration tests: generated Zellij config contains the integrated `yzpp`
  plugin block, popup/menu/config specs, and sidebar refresh hook
- CI checks: `nu nushell/scripts/dev/test_yzx_commands.nu`
- contract validator: `yzx_repo_validator validate-contracts`
- manual verification: `Alt+t` toggles one managed popup, `Alt+Shift+M`
  opens the menu, and `Alt+Shift+C` opens the config UI

## Traceability
- Defended by: `nu nushell/scripts/dev/test_yzx_commands.nu`
- Defended by: `nu nushell/scripts/dev/test_yzx_popup_commands.nu`
- Defended by: `nu nushell/scripts/dev/test_zellij_plugin_contracts.nu`
- Defended by: `cargo test --manifest-path rust_plugins/zellij_pane_orchestrator/Cargo.toml transient_pane_contract`
- Defended by: `yzx_repo_validator validate-contracts`

## Open Questions

- Should Yazi’s lazygit binding eventually route through the same configured
  `yzpp` popup contract when inside Yazelix/Zellij?
- Should Yazelix Zellij Popup and Yazelix eventually share a small popup contract crate, or is duplication acceptable while their release cadences differ?
