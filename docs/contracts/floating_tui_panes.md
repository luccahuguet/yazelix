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
- Reuse one shared floating-pane launch model for both popup surfaces
- Define the gate that must be met before popup is extracted as a standalone Zellij plugin or external package

## Contract Items

#### POP-001
- Type: behavior
- Status: live
- Owner: `yzx popup` plus transient-pane launch contract
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
- Owner: popup cwd resolution plus pane orchestrator contract
- Statement: Popup panes launch in the current tab workspace root when one is
  known, otherwise they fall back to the current shell directory
- Verification: automated `nu nushell/scripts/dev/test_yzx_popup_commands.nu`

#### POP-004
- Type: ownership
- Status: live
- Owner: pane orchestrator transient-pane lifecycle
- Statement: `Alt+t` toggles one managed popup pane instead of spawning
  duplicates forever, while `Alt+Shift+M` stays a separate command-palette flow
- Verification: automated `nu nushell/scripts/dev/test_yzx_popup_commands.nu`;
  automated `nu nushell/scripts/dev/test_zellij_plugin_contracts.nu`

#### POP-005
- Type: boundary
- Status: live
- Owner: popup standalone-capability decision boundary
- Statement: Popup must not be extracted from Yazelix until it has a generic
  non-Yazelix config surface, a stable request schema, a documented plain
  Zellij pipe or plugin command contract, tests that run without the full
  Yazelix runtime, and clear examples for plain Zellij users. Until every gate
  item is true, the in-repo Yazelix implementation remains the source of truth
- Verification: validator `yzx_repo_validator validate-contracts`; this contract

## Behavior

- `yzx popup` opens a floating Zellij pane using the configured `zellij.popup_program`.
- `zellij.popup_program` is an argv list, not a shell string.
- The default popup program is `["lazygit"]`.
- Popup geometry is user-configurable through `zellij.popup_width_percent` and `zellij.popup_height_percent`.
- Popup width and height percentages must be integers in the range `1..100`.
- The default popup width and height are both `90`.
- `yzx popup <command ...>` overrides the configured command for that invocation.
- The surviving popup/menu seam is one explicit transient-pane contract: kind, pane identity, wrapper path, mode env, argv, cwd, runtime, and geometry are resolved before the plugin open request is sent.
- The popup launches in the current tab workspace root when available; otherwise it uses the current shell directory.
- The popup closes on exit.
- `Alt+t` opens one managed popup pane when it is missing, focuses it when it exists but is unfocused, and closes it when it is focused.
- `Alt+Shift+M` continues to open the command-palette popup.
- Plain Zellij users have plausible future value from this capability: a reusable floating-pane toggle for one configured TUI command, stable pane identity, and duplicate-preventing focus/close behavior.
- That future value is not an extraction promise. Extraction is allowed only after the POP-005 gate is satisfied and examples prove the capability works without Yazelix-specific runtime paths, wrappers, config keys, or sidebar refresh hooks.
- The current Yazelix path remains canonical while the gate is unmet: `yzx popup`, `zellij.popup_program`, the transient-pane facts surface, and the pane-orchestrator transient contract define supported behavior.

## Non-goals

- General floating-pane support for every Yazelix action
- Converting all Yazi plugins to popup flows
- Background daemon management for long-running AI tools
- Splitting popup into a standalone repository or package before the POP-005 gate is met
- Treating Yazelix wrapper paths, runtime env, or sidebar refresh behavior as a plain-Zellij API

## Acceptance Cases

1. When a user presses `Alt+t` inside Yazelix, the configured popup program opens in one managed floating pane instead of replacing an existing workspace pane.
2. When `zellij.popup_program` is changed to another argv list, `yzx popup` launches that program without requiring shell-string parsing.
3. When `zellij.popup_width_percent` and `zellij.popup_height_percent` are set to valid values from `1` to `100`, `yzx popup` launches the popup with those dimensions.
4. When popup width or height is configured outside the valid `1..100` range, Yazelix fails fast with a clear config error.
5. When `yzx popup` runs from a tab with an explicit workspace root, the popup uses that root as its cwd.
6. Repeated popup-key presses do not create duplicate popup panes; they focus or close the existing managed popup instead.
7. When `Alt+Shift+M` is used, the command palette still opens separately from the popup-program flow.
8. A proposed popup extraction is rejected unless it supplies a generic config surface, stable request schema, documented Zellij command contract, runtime-independent tests, and plain-Zellij examples.
9. While the extraction gate is unmet, Yazelix docs and code should continue to identify the in-repo implementation as canonical.

## Verification

- unit tests: popup command/cwd resolution helpers
- unit tests: popup geometry config parsing and validation
- unit tests: popup lifecycle contract and transient-pane discovery in the pane orchestrator
- unit tests: popup-toggle wrapper decision path
- integration tests: `yzx popup` command routing and popup geometry arguments with a fake Zellij binary
- integration tests: generated Zellij config and permission cleanup remove stale popup-runner artifacts
- CI checks: `nu nushell/scripts/dev/test_yzx_commands.nu`
- contract validator: `yzx_repo_validator validate-contracts`
- manual verification: `Alt+t` toggles one managed popup and `Alt+Shift+M` still opens the menu

## Traceability
- Defended by: `nu nushell/scripts/dev/test_yzx_commands.nu`
- Defended by: `nu nushell/scripts/dev/test_yzx_popup_commands.nu`
- Defended by: `nu nushell/scripts/dev/test_zellij_plugin_contracts.nu`
- Defended by: `yzx_repo_validator validate-contracts`

## Open Questions

- Should Yazi’s lazygit binding eventually route through the same pane-orchestrated popup contract when inside Yazelix/Zellij?
- If the extraction gate is eventually met, should the standalone contract target only Zellij pipe commands first, or also a distributable plugin package?
