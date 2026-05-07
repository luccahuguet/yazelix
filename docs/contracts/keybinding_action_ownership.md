# Keybinding Action Ownership Contract

## Summary

Yazelix keybinding configuration is owner-scoped. Yazelix exposes semantic remaps only for actions whose behavior Yazelix owns; broader application keymaps stay in the owning tool's native config surface.

The current implemented semantic surface is `zellij.keybindings` in `settings.jsonc`. It covers Yazelix-owned Zellij actions that route to the pane orchestrator or Yazelix control-plane helpers.

Future Yazi and editor action remaps should use the same ownership rule without turning Yazelix into a generic cross-application keybinding DSL.

## Ownership Rule

Yazelix may provide a semantic action remap when all of these are true:

- Yazelix owns the behavior being invoked
- Yazelix can generate the backend binding safely
- Yazelix can validate duplicate or disabled bindings before launch
- the action id can stay stable across backend implementation changes

Yazelix should not provide a semantic remap when another tool owns the behavior. Those bindings stay in the native sidecar or native config:

- native Zellij mode and pane actions belong in `~/.config/yazelix/zellij.kdl`
- arbitrary Yazi file-manager actions belong in `~/.config/yazelix/yazi_keymap.toml`
- arbitrary Helix editor preferences belong in `~/.config/yazelix/helix.toml` for managed Helix sessions, or in the user's native Helix config outside Yazelix
- terminal-emulator shortcuts belong in the terminal emulator config

## Current Implemented Surface

`zellij.keybindings` is stable and remains backward-compatible:

```jsonc
{
  "zellij": {
    "keybindings": {
      "popup": ["Alt t"],
      "menu": ["Alt Shift M"],
      "toggle_sidebar": ["Alt y"]
    }
  }
}
```

Rules:

- omitted actions keep Yazelix defaults
- an empty list disables that Yazelix-owned action binding
- duplicate keys across the semantic map are rejected before launch
- generated binds are emitted without matching `unbind` lines for the same key
- explicit managed `keybinds clear-defaults=true` in `~/.config/yazelix/zellij.kdl` gives the user full native Zellij keybinding ownership and suppresses semantic Yazelix keybind generation
- read-only fallback from `~/.config/zellij/config.kdl` does not imply full Yazelix keybinding ownership, even if that native file uses `clear-defaults=true`

## Action Registry Shape

The Rust action registry is the shared source for Yazelix-owned action metadata that can feed generated bindings, `yzx keys`, doctor/config UI diagnostics, and future docs metadata. Registry entries use scoped action ids:

- `zellij.popup`
- `zellij.menu`
- `zellij.toggle_sidebar`
- `zellij.open_workspace_terminal`
- `yazi.open_selected_in_editor`
- `yazi.open_zoxide_in_editor`
- `editor.reveal_in_sidebar`

The persisted config may stay owner-scoped for compatibility, such as `zellij.keybindings.popup`, while the registry presents full ids in shared views like `yzx keys`, doctor diagnostics, and the config UI.

Each action registry entry includes:

- stable id
- owner-local id used by legacy owner-scoped config maps
- human label
- owner subsystem
- supported backend or backends
- default binding
- generated backend command
- whether an empty binding list is allowed
- diagnostics Yazelix can prove reliably

The current implemented registry slice is the Zellij semantic action set. Yazi and editor entries should be added only when their ownership and backend generation contracts are explicit.

## Profiles

Profiles are preset inputs, not another runtime keybinding owner.

Supported profile names, if profiles are implemented:

- `default`
- `emacs_friendly`
- `minimal`

A profile should expand into ordinary owner-scoped action maps. Explicit action entries must win over the profile. Profiles must not rewrite native sidecar files.

## Zellij Boundary

Yazelix owns semantic Zellij bindings only for Yazelix actions such as popup/menu/sidebar/workspace helpers and layout-family switching.

Yazelix does not own arbitrary Zellij native mode bindings such as `SwitchToMode "Locked"`. Users who want full native mode ownership should use an explicit managed `~/.config/yazelix/zellij.kdl` keybind block, and `keybinds clear-defaults=true` when they want to replace Zellij defaults.

The `Ctrl-g` conflict is handled as a Zellij-native ownership issue, not as a Yazelix semantic action. Yazelix may document and ship its default remap, but a user-owned replacement belongs in the managed Zellij sidecar.

## Yazi Boundary

Yazelix may expose semantic Yazi bindings for Yazelix-owned integration actions, such as opening selected files through the managed editor opener or retargeting the workspace from Yazi zoxide.

Yazelix should not expose semantic bindings for arbitrary Yazi-native behavior. Those stay in `~/.config/yazelix/yazi_keymap.toml`.

The generated Yazi opener remains Yazelix-owned. User `yazi.toml` overrides must not replace the managed editor opener accidentally.

## Editor Boundary

Yazelix may expose editor-local semantic actions only when the action invokes Yazelix-owned integration behavior, such as revealing the current file in the managed Yazi sidebar.

Yazelix does not own general Helix or Neovim keymaps.

Managed Helix file-open and cwd commands currently depend on Helix command mode being reachable through `:`. Remapping `:` away from command mode is unsupported until Yazelix implements an explicit backend command-mode entry setting or another robust editor command transport. Until then, Yazelix should fail visibly through doctor/config diagnostics rather than silently typing command text into the buffer.

## Diagnostics

Yazelix should diagnose only conflicts it can prove:

- duplicate keys inside one semantic action map
- unsupported semantic action ids
- malformed key strings
- disabled required actions, when an action is required for a managed workflow
- known backend precondition violations, such as managed Helix command-mode entry

Yazelix should not claim to fully diagnose conflicts inside arbitrary native tool config.

## Verification

- `yzx dev rust test zellij_materialization`
- `yzx dev rust test config_normalize`
- `yzx_repo_validator validate-config-surface-contract`
- `yzx_repo_validator validate-contracts`

## Traceability

- Defended by: `yzx_repo_validator validate-contracts`
