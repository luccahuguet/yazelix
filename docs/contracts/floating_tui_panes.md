# Floating TUI Panes

## Summary

Yazelix supports popup flows for terminal UIs that are useful temporarily but do not deserve a persistent split. `yzx popup <command ...>` opens one-off transient popups. Persistent generated surfaces use `zellij.popup_commands` for built-in Yazelix popups and `zellij.custom_popups` for user-defined popups. Popup keys behave like managed session surfaces rather than spawning disposable duplicates forever.

## Why

Yazelix already had a floating command-palette popup, but no coherent popup model for transient TUIs. That left tools like lazygit or AI agents either taking over the current pane or requiring ad hoc Zellij commands. A dedicated popup surface makes that behavior explicit and configurable without turning every pane into a floating-pane policy problem.

## Scope

- Add `yzx popup <command ...>`
- Add `zellij.popup_commands` and `zellij.custom_popups` to `config.toml` / Home Manager
- Bind built-in popup commands to semantic bottom, top, and menu defaults
- Ship `zenith` as the default `zellij.custom_popups` process information monitor on `Alt+Shift+I`
- Keep the command-palette popup as a separate flow
- Reuse the configured `yzpp` popup model for popup, menu, and config UI panes
- Keep Yazelix-specific side effects, such as sidebar refresh, outside the plain
  popup contract through explicit hooks

## Contract Items

#### POP-001
- Type: behavior
- Status: live
- Owner: `yzx popup` plus `yzpp` popup config
- Statement: `yzx popup <command ...>` resolves one explicit argv list, not a
  shell string. No-argument `yzx popup` fails clearly because persistent
  configured popups live in `zellij.custom_popups`
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core yzx_control_popup_explicit_program_opens_through_yzpp_raw_request`;
  automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core yzx_control_popup_without_program_errors_clearly`

#### POP-002
- Type: failure_mode
- Status: live
- Owner: popup config validation and render-plan owners
- Statement: `zellij.popup_width_percent` and
  `zellij.popup_height_percent` must be integers in the range `1..100`.
  Invalid values fail fast as config errors instead of being coerced silently
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_zellij_config_pack`;
  main materialization tests; validator `yzx_repo_validator validate-contracts`

#### POP-003
- Type: behavior
- Status: live
- Owner: popup cwd resolution plus `yzpp` raw request adapter
- Statement: Popup panes launch in the current tab workspace root when one is
  known, otherwise they fall back to the current shell directory
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core yzx_control_popup_explicit_program_opens_through_yzpp_raw_request`

#### POP-004
- Type: ownership
- Status: live
- Owner: configured `yzpp` popup lifecycle
- Statement: `Alt+Shift+J` toggles the default bottom managed popup pane
  instead of spawning duplicates forever, while `Alt+Shift+M` toggles the
  configured `menu` popup command and `Alt+Shift+I` toggles the default
  `zenith` custom popup
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core yzpp_popup_specs_use_distinct_popup_commands`;
  automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core semantic_keybinds_route_popup_actions_to_yzpp`

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
  `yazelix-zellij-popup` gates `cargo test` and `nix build`; main package
  gate `nix build .#runtime`

#### POP-006
- Type: boundary
- Status: live
- Owner: first-party Zellij plugin package boundary
- Statement: Yazelix packaging consumes `yzpp.wasm` from a
  `yazelix-zellij-popup` package input instead of treating a copied
  `configs/zellij/plugins/yzpp.wasm` source file as the durable source of truth
- Verification: validator `yzx_repo_validator validate-contracts`

#### POP-007
- Type: non_goal
- Status: live
- Owner: workspace/editor/sidebar popup option boundary
- Statement: Yazelix does not add a default editor popup or Yazi popup picker
  in the current sidebar-first workspace. Yazi-driven file opens, including
  `open_from_yazi` flows, continue to target or create the managed `editor`
  pane. The default Yazi file-tree sidebar remains the picker surface. A Yazi
  popup picker can be reconsidered only with an accepted no-sidebar layout
  contract, and an editor popup would need a separate managed-editor identity
  design before it could replace stack insertion
- Verification: validator `yzx_repo_validator validate-contracts`

## Behavior

- `yzx popup <command ...>` opens a one-off floating Zellij pane using an explicit argv list.
- No-argument `yzx popup` fails clearly and points users at explicit commands or `zellij.custom_popups`.
- `zellij.popup_commands` is a map of built-in popup argv lists.
- The default named popup commands are `bottom_popup = ["lazygit"]`,
  `top_popup = ["yzx", "config", "ui"]` for Yazelix's ratconfig-backed config
  editor, and `menu = ["yzx", "menu"]`.
- `zellij.custom_popups` is a list of user-defined popup specs with `id`,
  `command`, `keybindings`, and optional `keep_alive`.
- The default custom popup is `{ id = "zenith", command = ["zenith"], keybindings = ["Alt Shift I"], keep_alive = true }`.
- `keep_alive = true` makes focused toggle suppress that popup pane without
  killing the child process or hiding the whole floating layer. Explicit close
  still closes the pane. Omitted `keep_alive` defaults to true for the
  default-shaped `zenith` popup and false for other custom popups.
- Popup geometry is user-configurable through `zellij.popup_width_percent` and `zellij.popup_height_percent`.
- Popup width and height percentages must be integers in the range `1..100`.
- The default popup width and height are both `90`.
- `yzx popup <command ...>` opens a transient popup without changing `config.toml`.
- The generated Yazelix `yzpp` specs own the stable pane identity, argv, cwd,
  runtime command path, geometry, and close hook for popup/menu/config panes.
- Yazelix generates `bottom_popup`, `top_popup`, `menu`, `config`, and every
  configured `zellij.custom_popups` spec. Explicit `yzx popup <command ...>`
  uses the transient `popup` raw request instead of a persisted spec.
- The popup launches in the current tab workspace root when available; otherwise it uses the current shell directory.
- Popup pane lifecycle is controlled by the popup keybinding and explicit
  `yzpp` `toggle` or `close` messages, not by child process exit.
- `Alt+Shift+J` opens one managed bottom popup pane when it is missing, focuses it when it exists but is unfocused, and closes it when it is focused.
- `Alt+Shift+K` does the same for the semantic top popup slot, which defaults
  to `yzx config ui`, Yazelix's Ratconfig-backed settings editor.
- `Alt+Shift+I` toggles the semantic Zenith popup slot. It defaults to the bundled
  Zenith process viewer through `zellij.custom_popups` and suppresses that pane
  instead of closing it on focused toggle so process graphs can keep their
  history.
- When `Alt+Shift+J` closes the configured popup pane, Yazelix runs `yzx sidebar
  refresh` through an `on_close` hook so lazygit-style workflows refresh the
  managed Yazi sidebar.
- `Alt+Shift+M` opens the configured `menu` popup command through `yzpp`.
- `Alt+Shift+C` opens the config UI popup through `yzpp`.
- Plain Zellij users get this capability through Yazelix Zellij Popup (`yzpp`): a reusable floating-pane toggle for configured TUI commands, stable pane identity, and duplicate-preventing focus/close behavior.
- The external Yazelix Zellij Popup plugin provides this capability without
  requiring Yazelix-specific runtime paths, wrappers, config keys, or sidebar
  refresh behavior; integrations can opt into generic `on_close` command hooks.
- The current Yazelix path remains canonical for the integrated product: `yzx
  popup <command ...>`, `zellij.popup_commands`, `zellij.custom_popups`,
  generated `yzpp` specs, and `yzx sidebar refresh` define supported Yazelix
  popup behavior.

### Adjacent Workspace Popup Decision

Yazelix keeps the current editor/sidebar model instead of adding editor or Yazi
picker popups by default.

- Keeping the current model is accepted because it preserves one persistent
  managed editor identity, one sidebar picker, and the existing `Ctrl+y`,
  `Alt+Shift+H`, `Alt+z`, and Yazi open behaviors without adding global keys
- A Yazi popup picker is rejected for the current sidebar-first default because
  it duplicates the file-tree sidebar and zoxide picker without removing a
  current pane or ownership boundary
- A Yazi popup picker may be reconsidered only if Yazelix accepts a no-sidebar
  layout where the popup replaces the missing persistent sidebar rather than
  duplicating it
- An editor popup is rejected because it creates a second editor identity,
  makes Yazi open routing ambiguous, and weakens workspace cwd sync unless it
  first becomes the managed `editor` pane through a separate identity contract
- Adding both popups is rejected because it multiplies keybindings, pane states,
  and user memory burden without deleting the existing editor/sidebar model
- Yazi-driven opens continue to target an existing managed `editor` pane or
  create one through the normal managed editor flow; they do not route to a
  transient popup editor

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
- Adding a default Yazi popup picker to the sidebar-first workspace
- Adding a popup editor as an alternate managed editor owner
- Background daemon management for long-running AI tools
- Reabsorbing Yazelix Zellij Popup source into Yazelix core
- Treating Yazelix wrapper paths, runtime env, or sidebar refresh behavior as a plain-Zellij API

## Acceptance Cases

1. When a user presses `Alt+Shift+J` inside Yazelix, `zellij.popup_commands.bottom_popup` opens in one managed floating pane instead of replacing an existing workspace pane.
2. When `yzx popup gitui` is run, the explicit command launches without requiring shell-string parsing.
3. When `zellij.popup_width_percent` and `zellij.popup_height_percent` are set to valid values from `1` to `100`, transient and generated popup specs use those dimensions.
4. When popup width or height is configured outside the valid `1..100` range, Yazelix fails fast with a clear config error.
5. When `yzx popup` runs from a tab with an explicit workspace root, the popup uses that root as its cwd.
6. Repeated popup-key presses do not create duplicate popup panes; they focus or close the existing managed popup instead.
7. When `Alt+Shift+M` is used, the command palette still opens separately from transient `yzx popup` requests.
8. When `Alt+Shift+K` is used, `zellij.popup_commands.top_popup` opens through the same duplicate-preventing lifecycle as the bottom popup slot.
9. The extracted `yazelix-zellij-popup` source stays in its child repository while Yazelix packages and integrates its `yzpp.wasm` artifact.
10. The standalone plugin supports KDL-native configured popup specs and keeps raw JSON pipe requests only for generated integrations.
11. Full Yazelix docs and code identify `yzpp` as the popup/menu/config pane owner and the pane orchestrator as the workspace/sidebar/editor/session owner.
12. When a file is opened from Yazi, Yazelix targets or creates the managed `editor` pane instead of opening a transient popup editor.
13. In the sidebar-first workspace, Yazelix does not add a separate Yazi popup picker key that duplicates the persistent Yazi file tree.

## Verification

- unit tests: popup command/cwd resolution helpers
- unit tests: popup geometry config parsing and validation
- unit tests: popup lifecycle identity in the pane orchestrator
- external `yazelix-zellij-popup` unit tests: KDL-native popup specs, optional
  `on_close` hooks, and raw generated pipe request compatibility
- external package gate: `nix build` in `yazelix-zellij-popup`
- package integration gate: `nix build .#runtime`
- integration tests: `yzx popup <command ...>` routes transient popup requests to `yzpp`
  with a fake Zellij binary
- integration tests: generated Zellij config contains the integrated `yzpp`
  plugin block, bottom_popup/top_popup/menu/zenith/config specs, and sidebar
  refresh hook
- CI checks: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_control_workspace_surface`
- contract validator: `yzx_repo_validator validate-contracts`
- manual verification: `Alt+Shift+J` toggles the bottom managed popup,
  `Alt+Shift+K` toggles the top managed popup, `Alt+Shift+M` opens the menu,
  `Alt+Shift+I` toggles the keep-alive Zenith process information viewer, and `Alt+Shift+C`
  opens the config UI

## Traceability
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_control_workspace_surface`
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core zellij_materialization`
- Defended by: `cargo test --manifest-path ../yazelix-zellij-pane-orchestrator/Cargo.toml transient_pane_contract`
- Defended by: `yzx_repo_validator validate-contracts`

## Open Questions

- Should Yazi’s lazygit binding eventually route through the same configured
  `yzpp` popup contract when inside Yazelix/Zellij?
- Should Yazelix Zellij Popup and Yazelix eventually share a small popup contract crate, or is duplication acceptable while their release cadences differ?
