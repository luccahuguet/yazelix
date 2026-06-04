# Keybinding Action Ownership Contract

## Summary

Yazelix keybinding configuration is owner-scoped. Yazelix exposes semantic remaps only for actions whose behavior Yazelix owns; broader application keymaps stay in the owning tool's native config surface.

The current implemented semantic surfaces are `zellij.keybindings` and `yazi.keybindings` in `settings.jsonc`. They cover Yazelix-owned integration actions that route to the pane orchestrator, generated Yazi keymap commands, or Yazelix control-plane helpers.

`zellij.native_keybindings` is a separate curated native-policy surface. It is not a generic Zellij keymap DSL; it only exposes Yazelix's shipped native Zellij conflict-remap defaults and convenience policy.

Future Yazi and editor action remaps should use the same ownership rule without turning Yazelix into a generic cross-application keybinding DSL.

## Ownership Rule

Yazelix may provide a semantic action remap when all of these are true:

- Yazelix owns the behavior being invoked
- Yazelix can generate the backend binding safely
- Yazelix can validate duplicate or disabled bindings before launch
- the action id can stay stable across backend implementation changes

Yazelix should not provide a semantic remap when another tool owns the behavior. Those bindings stay in the native sidecar or native config:

- curated Yazelix native Zellij conflict policy belongs in `zellij.native_keybindings`
- full native Zellij keymap ownership belongs to plain `zellij` outside Yazelix
- arbitrary Yazi file-manager actions belong in `~/.config/yazelix/yazi/keymap.toml`
- arbitrary Helix editor preferences belong in `~/.config/yazelix/helix/config.toml` for managed Helix sessions, or in the user's native Helix config outside Yazelix
- terminal-emulator shortcuts belong in the terminal emulator config

## Current Implemented Surface

`zellij.keybindings` owns fixed Yazelix semantic actions:

```jsonc
{
  "zellij": {
    "keybindings": {
      "bottom_popup": ["Alt Shift J"],
      "top_popup": ["Alt Shift K"],
      "menu": ["Alt Shift M"],
      "toggle_editor_right_sidebar_focus": ["Ctrl Shift Y"],
      "toggle_left_sidebar": ["Alt Shift H"],
      "open_codex_agent_right": ["Alt Shift L"]
    }
  }
}
```

Rules:

- omitted actions keep Yazelix defaults
- an empty list disables that Yazelix-owned action binding
- duplicate keys across the semantic map are rejected before launch
- generated binds are emitted without matching `unbind` lines for the same key
- managed `~/.config/yazelix/zellij.kdl` must not contain `keybinds` blocks
- read-only fallback from `~/.config/zellij/config.kdl` does not imply full Yazelix keybinding ownership, even if that native file uses `clear-defaults=true`

`zellij.native_keybindings` is stable for Yazelix's curated native Zellij policy:

```jsonc
{
  "zellij": {
    "native_keybindings": {
      "scroll_mode_unbind": ["Ctrl s"],
      "scroll_mode": ["Ctrl Alt s"],
      "session_mode_unbind": ["Ctrl o"],
      "session_mode": ["Ctrl Alt o"]
    }
  }
}
```

Rules:

- omitted entries keep Yazelix defaults
- an empty list disables one native policy bind or unbind entry
- bind and unbind entries are adjacent in the default template so users can reason about remaps as one policy
- arbitrary native Zellij keymap ownership belongs to plain `zellij`; Yazelix-managed sessions expose only the curated native policy above

`yazi.keybindings` covers only generated Yazelix-owned Yazi integration actions that are not native Yazi defaults:

```jsonc
{
  "yazi": {
    "keybindings": {
      "open_zoxide_in_editor": ["<A-z>"],
      "open_directory_as_workspace_pane": ["<A-p>"]
    }
  }
}
```

Rules:

- omitted actions keep Yazelix defaults
- an empty list disables that generated Yazelix-owned Yazi integration binding
- multiple entries generate multiple alternate bindings for the same Yazelix-owned action, not a native Yazi key sequence
- duplicate keys across the semantic Yazi map are rejected before keymap generation
- native open-selected keys such as `<Enter>` and `o` are not part of this map; they remain Yazi-native `open` bindings even though Yazelix owns the generated `edit` opener target
- arbitrary Yazi-native keymap ownership remains in `~/.config/yazelix/yazi/keymap.toml`

## Action Registry Shape

The Rust action registry is the shared source for Yazelix-owned action metadata that can feed generated bindings, `yzx keys`, doctor/config UI diagnostics, and future docs metadata. Registry entries use scoped action ids:

- `zellij.bottom_popup`
- `zellij.top_popup`
- `zellij.menu`
- `zellij.toggle_editor_right_sidebar_focus`
- `zellij.toggle_left_sidebar`
- `zellij.open_workspace_terminal`
- `yazi.open_directory_as_workspace_pane`
- `yazi.open_zoxide_in_editor`
- `editor.reveal_in_sidebar`

User-defined popup bindings live in `zellij.custom_popups`, not the static action registry. The default `btm` popup is a custom popup entry with `keybindings = ["Alt Shift B"]`.

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

The current implemented registry slices are the Zellij semantic action set, the curated native Zellij policy set, and the generated Yazi integration action set. Editor entries should be added only when their ownership and backend generation contracts are explicit.

## Profiles

Profiles are preset inputs, not another runtime keybinding owner.

Supported profile names, if profiles are implemented:

- `default`
- `emacs_friendly`
- `minimal`

A profile should expand into ordinary owner-scoped action maps. Explicit action entries must win over the profile. Profiles must not rewrite native sidecar files.

## Zellij Boundary

Yazelix owns semantic Zellij bindings only for Yazelix actions such as popup/menu/sidebar/workspace helpers and layout-family switching.

Yazelix does not own arbitrary Zellij native mode bindings. Managed `~/.config/yazelix/zellij.kdl` is a native settings sidecar, not a keymap sidecar, and any `keybinds` block there is a config error. Users who want full native Zellij keymap ownership should run plain `zellij` outside Yazelix.

The `Ctrl-g`, `Ctrl-s`, `Ctrl-o`, Helix `Alt` conflict, tab jump, and pane-grouping defaults are handled as curated native Zellij policy in `zellij.native_keybindings`, not as semantic Yazelix actions. Native keybinding behavior outside that curated policy is not a managed Yazelix session surface.

Yazelix does not manage arbitrary Zellij keymaps, full Zellij mode binding ownership, or generated runtime config edits. `~/.config/zellij/config.kdl` belongs to plain Zellij and is only a read-only fallback or explicit import source for Yazelix.

## Yazi Boundary

Yazelix may expose semantic Yazi bindings for Yazelix-owned integration actions, such as opening a selected directory in a workspace pane or retargeting the workspace from Yazi zoxide.

Yazelix should not expose semantic bindings for arbitrary Yazi-native behavior. Those stay in `~/.config/yazelix/yazi/keymap.toml`.

The generated Yazi opener remains Yazelix-owned. User `yazi.toml` overrides must not replace the managed editor opener accidentally.

`open_selected_in_editor` is intentionally not a semantic `yazi.keybindings` action. Yazi's native manager keymap binds `o` and `<Enter>` to the native `open` command, and that command chooses the Yazelix-owned `edit` opener for editable files. A semantic remap would be misleading because:

- setting `yazi.keybindings.open_selected_in_editor = []` could not honestly disable the native Yazi `open` defaults
- assigning another key would leave `o` and `<Enter>` active unless Yazelix started generating native shadow/no-op bindings
- shadowing or removing those defaults would make Yazelix own part of Yazi's native manager keymap, which is exactly what `~/.config/yazelix/yazi/keymap.toml` is for

Users who want `e`, `<Enter>`, `o`, or another key to run Yazi's native `open` command should set that in `~/.config/yazelix/yazi/keymap.toml`. The resulting `open` behavior still routes editable files through Yazelix's managed editor opener.

## Editor Boundary

Yazelix may expose editor-local semantic actions only when the action invokes Yazelix-owned integration behavior, such as revealing the current file in the managed Yazi sidebar.

Yazelix does not own general Helix or Neovim keymaps.

Managed Helix file-open and cwd commands depend on Helix command mode being reachable through `:`. Yazelix-managed Helix materialization therefore enforces `":" = "command_mode"` after merging user overrides, the same way it enforces the `A-r` reveal binding. Users may add another command-mode key, but managed sessions cannot repurpose `:` without breaking the Yazi-to-Helix command transport. Doctor should report stale generated Helix configs that do not contain this binding.

## Diagnostics

Yazelix should diagnose only conflicts it can prove:

- duplicate keys inside one semantic action map
- unsupported semantic action ids
- malformed key strings
- disabled required actions, when an action is required for a managed workflow
- managed `~/.config/yazelix/zellij.kdl` files that contain `keybinds` blocks
- known backend precondition violations, such as managed Helix command-mode entry

Yazelix should not claim to fully diagnose conflicts inside arbitrary native tool config.

## Verification

- `yzx dev rust test zellij_materialization`
- `yzx dev rust test yazi_materialization`
- `yzx dev rust test config_normalize`
- `yzx_repo_validator validate-config-surface-contract`
- `yzx_repo_validator validate-contracts`

## Traceability

- Defended by: `yzx_repo_validator validate-contracts`
