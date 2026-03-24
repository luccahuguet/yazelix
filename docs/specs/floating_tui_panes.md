# Floating TUI Panes

## Summary

Yazelix should support one explicit popup flow for terminal UIs that are useful temporarily but do not deserve a persistent split. The first supported surface is `yzx popup`, backed by a user-configured `zellij.popup_program` that defaults to `lazygit`. The popup key should behave like a managed session surface rather than spawning disposable duplicates forever.

## Why

Yazelix already had a floating command-palette popup, but no coherent popup model for transient TUIs. That left tools like lazygit or AI agents either taking over the current pane or requiring ad hoc Zellij commands. A dedicated popup surface makes that behavior explicit and configurable without turning every pane into a floating-pane policy problem.

## Scope

- Add `yzx popup`
- Add `zellij.popup_program` to `yazelix.toml` / Home Manager
- Bind the configured popup to a dedicated key
- Keep the command-palette popup as a separate flow
- Reuse one shared floating-pane launch model for both popup surfaces

## Behavior

- `yzx popup` opens a floating Zellij pane using the configured `zellij.popup_program`.
- `zellij.popup_program` is an argv list, not a shell string.
- The default popup program is `["lazygit"]`.
- `yzx popup <command ...>` overrides the configured command for that invocation.
- The popup launches in the current tab workspace root when available; otherwise it uses the current shell directory.
- The popup closes on exit.
- `Alt+t` opens one managed popup pane when it is missing, focuses it when it exists but is unfocused, and closes it when it is focused.
- `Alt+Shift+M` continues to open the command-palette popup.

## Non-goals

- General floating-pane support for every Yazelix action
- Converting all Yazi plugins to popup flows
- Background daemon management for long-running AI tools

## Acceptance Cases

1. When a user presses `Alt+t` inside Yazelix, the configured popup program opens in one managed floating pane instead of replacing an existing workspace pane.
2. When `zellij.popup_program` is changed to another argv list, `yzx popup` launches that program without requiring shell-string parsing.
3. When `yzx popup` runs from a tab with an explicit workspace root, the popup uses that root as its cwd.
4. Repeated popup-key presses do not create duplicate popup panes; they focus or close the existing managed popup instead.
5. When `Alt+Shift+M` is used, the command palette still opens separately from the popup-program flow.

## Verification

- unit tests: popup command/cwd resolution helpers
- unit tests: popup lifecycle contract and popup-pane discovery in the popup runner
- unit tests: popup-toggle wrapper decision path
- integration tests: `yzx popup` command routing with a fake Zellij binary
- CI checks: `nu nushell/scripts/dev/test_yzx_commands.nu`
- manual verification: `Alt+t` toggles one managed popup and `Alt+Shift+M` still opens the menu

## Traceability

- Bead: `yazelix-2v0`
- Defended by: `nu nushell/scripts/dev/test_yzx_commands.nu`
- Defended by: `nu nushell/scripts/dev/test_yzx_popup_commands.nu`
- Defended by: `cargo test --manifest-path rust_plugins/zellij_popup_runner/Cargo.toml --lib`

## Open Questions

- Should Yazi’s lazygit binding eventually route through the same popup runner when inside Yazelix/Zellij?
- Should popup geometry become user-configurable later, or remain a Yazelix-owned default?
