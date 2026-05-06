# Native Config Integration Status Contract

## Summary

Yazelix reports native-config integration with explicit status labels instead of silently adopting user-managed files.

`settings.jsonc` remains the canonical semantic settings source. Tool-native files can be imported, used read-only where a tool-specific contract allows it, or left as user-owned native config. Import and export are explicit user actions.

## Status Vocabulary

The config UI and doctor should use the same status vocabulary.

| Status | User label | Meaning |
| --- | --- | --- |
| `canonical_settings` | Canonical Yazelix settings | `settings.jsonc` is the semantic source for Yazelix behavior |
| `managed_default` | Yazelix default | No managed sidecar exists; Yazelix generates behavior from packaged defaults and semantic settings |
| `managed_override` | Yazelix-managed override | A sidecar under `~/.config/yazelix/` customizes the generated runtime config |
| `imported_override` | Imported into Yazelix | A native file was copied through an explicit Yazelix import action and is now a managed override |
| `native_read_only` | Native read-only source | Yazelix reads a native tool config without taking ownership of it |
| `native_available` | Native config available to import | A native tool config exists but Yazelix is not using it |
| `native_missing` | Native config missing | A native source expected by an import or user-owned mode is absent |
| `native_required_missing` | Required native config missing | A setting explicitly requires a native file, but the file is absent |
| `home_manager_read_only` | Home Manager-managed | The active Yazelix config surface is read-only from the UI because Home Manager owns it |
| `generated_runtime` | Generated runtime output | A file under Yazelix state/data is generated and should not be edited directly |
| `native_user_owned` | User-owned native config | A native tool config exists outside Yazelix ownership and is only mentioned for context |
| `not_inspected` | Not inspected | Yazelix intentionally does not inspect that native surface |

`imported_override` must not be inferred by comparing file contents. It requires explicit import provenance metadata or an import command result in the current operation. Without provenance, the durable status is `managed_override`.

## Cross-Cutting Rules

1. Yazelix never moves, deletes, edits, or takes ownership of native files as a launch side effect.
2. `yzx import <tool>` copies native config into Yazelix-managed paths and writes backups before overwriting managed destinations when `--force` is used.
3. Missing native files are informational unless the user invoked an import command or selected a mode that requires native config.
4. Generated runtime files under `~/.local/share/yazelix/` are outputs, not user input surfaces.
5. Home Manager ownership makes UI editing read-only; remediation should point to the Home Manager module option or managed sidecar source.

## Per-Tool Status Rules

### Main Yazelix Settings

Surface:

- `~/.config/yazelix/settings.jsonc`

Status:

- `canonical_settings` when Yazelix owns the editable file
- `home_manager_read_only` when Home Manager owns the active settings surface

Rules:

- semantic settings always come from this surface or a launch-scoped explicit override snapshot
- native tool config must not bypass this source

### Zellij

Managed input:

- `~/.config/yazelix/zellij.kdl`

Native source:

- `~/.config/zellij/config.kdl`

Generated output:

- `~/.local/share/yazelix/configs/zellij/config.kdl`

Statuses:

- `managed_override` when the managed `zellij.kdl` exists
- `imported_override` only immediately after `yzx import zellij` or when future import provenance proves it
- `native_read_only` when managed `zellij.kdl` is absent and native `config.kdl` exists
- `managed_default` when neither managed nor native config exists and Yazelix uses Zellij defaults plus Yazelix overlays
- `generated_runtime` for the merged output file

Rules:

- native read-only fallback does not mean full keybinding ownership
- `keybinds clear-defaults=true` suppresses Yazelix semantic keybind generation only in explicit managed `~/.config/yazelix/zellij.kdl`
- `yzx import zellij` is the explicit adoption path

### Yazi

Managed inputs:

- `~/.config/yazelix/yazi.toml`
- `~/.config/yazelix/yazi_keymap.toml`
- `~/.config/yazelix/yazi_init.lua`

Native source:

- `~/.config/yazi/yazi.toml`
- `~/.config/yazi/keymap.toml`
- `~/.config/yazi/init.lua`

Generated output:

- `~/.local/share/yazelix/configs/yazi/`

Statuses:

- `managed_override` for existing managed Yazi sidecars
- `native_available` when native Yazi files exist but are not imported
- `managed_default` for absent managed sidecars
- `generated_runtime` for generated Yazi config, plugins, and flavors

Rules:

- Yazelix does not use native Yazi config read-only at launch
- native Yazi import is explicit through `yzx import yazi`
- the Yazelix-managed opener remains owned by Yazelix even when user sidecars exist

### Helix

Managed input:

- `~/.config/yazelix/helix.toml`

Native source:

- `~/.config/helix/config.toml`

Generated output:

- `~/.local/share/yazelix/configs/helix/config.toml`

Statuses:

- `managed_override` when the managed Helix sidecar exists
- `native_available` when native Helix config exists but is not imported
- `managed_default` when no managed sidecar exists
- `generated_runtime` for the generated managed Helix config

Rules:

- Yazelix-managed Helix sessions use the managed/generated Helix surface
- native Helix direct reuse is not a supported launch mode
- `yzx import helix` is the explicit adoption path

### Terminals

Managed inputs:

- `settings.jsonc` terminal fields
- optional `~/.config/yazelix/terminal_*.conf|toml|ini` sidecars

Native sources:

- terminal-specific native config paths, such as Ghostty, Kitty, Alacritty, or Foot config files

Generated output:

- `~/.local/share/yazelix/configs/terminal_emulators/`

Statuses:

- `managed_default` when `terminal.config_mode = "yazelix"` and no optional terminal sidecar exists
- `managed_override` when terminal sidecars exist in Yazelix config
- `native_read_only` when `terminal.config_mode = "user"` and the selected terminal native config exists
- `native_required_missing` when `terminal.config_mode = "user"` and no supported native config exists
- `generated_runtime` for generated terminal configs in Yazelix mode

Rules:

- `terminal.config_mode = "user"` is explicit native reuse, not import
- missing native terminal config is a launch-blocking error only in `user` mode
- terminal-native config remains owned by the terminal/user

### Shell Hooks

Managed inputs:

- `~/.config/yazelix/shell_bash.sh`
- `~/.config/yazelix/shell_zsh.zsh`
- `~/.config/yazelix/shell_fish.fish`
- `~/.config/yazelix/shell_nu.nu`

Native sources:

- user shell rc files such as `.bashrc`, `.zshrc`, Fish config, or Nushell config

Statuses:

- `managed_override` for existing Yazelix shell hooks
- `managed_default` when no Yazelix shell hook exists
- `not_inspected` for native shell rc files

Rules:

- Yazelix does not source native shell rc files implicitly
- Yazelix does not import native shell rc files
- shell hooks are opt-in managed sidecars scoped to Yazelix shells

## Config UI Display Rules

For each surface, the config UI should display:

- status label
- active input path, when one exists
- generated runtime path, when one exists
- native source path, when relevant
- allowed action: edit managed, import native, open read-only, or no direct action
- read-only reason, when Home Manager owns the surface

The UI should not offer direct editing for:

- generated runtime outputs
- native read-only fallback sources
- native shell rc files
- Home Manager-owned active settings

## Doctor Diagnostics

Doctor should classify native-config status as follows:

- error: `native_required_missing`, malformed managed sidecar, unreadable active managed surface, invalid generated runtime output
- warning: native read-only Zellij fallback is active, Home Manager read-only ownership prevents UI edits, import provenance is unknown for an existing managed override if provenance becomes required
- info: native config is available to import, managed sidecar is absent and defaults are active, generated runtime output path

Doctor should not warn merely because a native config is missing for Helix, Yazi, Zellij, or shell hooks unless the user requested import or enabled an explicit native-required mode.

## Verification

- `yzx_repo_validator validate-contracts`
- future status model tests for per-tool state classification
- future config UI tests for status labels and read-only actions
- future doctor tests for missing native terminal config and native-available informational diagnostics

## Traceability

- Defended by: `yzx_repo_validator validate-contracts`
