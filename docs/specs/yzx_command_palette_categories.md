# yzx Command Palette Categories

## Summary

`yzx menu` should not discover commands through a narrow hand-filtered allowlist. It should classify the public `yzx` surface into practical palette categories and allow most user-facing commands by default.

The palette is a command-discovery and dispatch surface, not a second shell. So the right rule is:

- broadly include user-facing `yzx` commands that make sense as direct actions
- explicitly exclude commands whose semantics are pane-scoped, tab-scoped, argument-driven, or maintainer-only

This spec complements backend-coupling analysis. It does not replace it.

## Why

The current `yzx menu` implementation in `nushell/scripts/yzx/menu.nu` discovers commands from `help commands` and then layers a small set of ad hoc exclusions on top. That has two problems:

1. palette eligibility is implicit and fragile
2. command grouping is mixed with implementation ownership

Recent command-surface cleanup made the problem more obvious:

- `yzx enter` is now distinct from `yzx launch`
- `yzx menu` itself is due for a thin-wrapper refactor
- the public `yzx` surface is already documented elsewhere by backend ownership, but not by palette UX

Without a palette-specific contract, `menu.nu` will keep accreting one-off filters.

## Scope

- define practical palette categories for public `yzx` commands
- define default palette eligibility rules
- define explicit exclusions
- explain how palette categories relate to backend-coupling buckets
- provide a source-of-truth grouping that `yzx menu` can consume later

## Source Of Truth

The command inventory for this spec comes from the real exported surface:

- `nushell/scripts/core/yazelix.nu`
- `nushell/scripts/yzx/*.nu`

Sanity check:

```bash
nu -c 'use nushell/scripts/core/yazelix.nu *; help commands | where name =~ "^yzx( |$)" | select name description | sort-by name'
```

This spec intentionally excludes:

- maintainer-only `yzx dev *`
- helper exports that are not user-facing commands

## Behavior

### Palette Eligibility Rule

A public `yzx` command is palette-eligible by default when all of these are true:

- it is a normal user-facing command
- it can be invoked meaningfully without requiring arbitrary free-form extra arguments
- its semantics make sense as a direct command-palette action
- it is not inherently scoped to “the current pane” or “the current tab” as a low-level session mutation

A command is not palette-eligible by default when any of these are true:

- it is pane-scoped or tab-scoped operational control
- it primarily expects arbitrary trailing shell arguments
- it is a maintainer surface
- it is the palette itself

### Categories

The palette should group eligible commands into these categories.

#### Session

Commands that start, enter, or restart a Yazelix session.

Included:

- `yzx launch`
- `yzx enter`
- `yzx restart`

#### Workspace

Commands that act on the current working workspace or visible UX surface.

Included:

- `yzx reveal`
- `yzx popup`
- `yzx screen`

Excluded from normal palette handling:

- `yzx cwd`

Reason:

`yzx cwd` is a tab-scoped mutation with current-tab semantics and interactive path/query input. It is real user functionality, but it should not be treated as a normal palette item in the same way as simple direct commands.

#### Config

Commands that show, edit, import, migrate, or reset user-managed config surfaces.

Included:

- `yzx config`
- `yzx config migrate`
- `yzx config reset`
- `yzx edit`
- `yzx edit config`
- `yzx edit packs`
- `yzx import`
- `yzx import helix`
- `yzx import yazi`
- `yzx import zellij`

#### Runtime And System

Commands that inspect, update, repair, or maintain the installed/runtime surface.

Included:

- `yzx doctor`
- `yzx status`
- `yzx packs`
- `yzx gc`
- `yzx update`
- `yzx update nix`
- `yzx repair`
- `yzx repair zellij-permissions`
- `yzx desktop install`
- `yzx desktop launch`
- `yzx desktop uninstall`

Note:

These may be heavier or more operational, but they are still legitimate palette actions because they are direct user-facing commands with no arbitrary trailing-argument surface.

#### Help And Discovery

Commands that teach, explain, or summarize the product.

Included:

- `yzx`
- `yzx why`
- `yzx sponsor`
- `yzx whats_new`
- `yzx keys`
- `yzx keys yzx`
- `yzx keys yazi`
- `yzx keys hx`
- `yzx keys helix`
- `yzx keys nu`
- `yzx keys nushell`
- `yzx tutor`
- `yzx tutor hx`
- `yzx tutor helix`
- `yzx tutor nu`
- `yzx tutor nushell`

### Explicit Exclusions

These commands are outside the normal palette contract:

- `yzx menu`
  - the palette should not list itself
- `yzx dev *`
  - maintainer-only surface
- `yzx env`
  - shell-ownership/control-plane command, not a palette action
- `yzx run`
  - argument-driven shell command runner, not a palette action
- `yzx cwd`
  - tab-scoped workspace mutation, not a normal palette item

### Relation To Backend-Coupling Buckets

This spec is about palette UX, not backend architecture.

`yzx_command_surface_backend_coupling.md` answers:

- which command families are backend-bound
- which are runtime-owned
- which are mixed seams

This spec answers:

- which commands belong in the command palette
- how they should be grouped for human discovery
- which commands are explicitly excluded even if they are public

The two models should agree on inventory, but they intentionally optimize for different questions.

## Acceptance Cases

1. When `yzx menu` is refactored, it can derive grouping and eligibility from this spec instead of layering more string filters over `help commands`.
2. When a new public `yzx` command is added, maintainers can decide whether it belongs in the palette by checking these eligibility rules rather than guessing from precedent.
3. When a user asks why `yzx env`, `yzx run`, or `yzx cwd` are not normal palette items, the answer is explicit and intentional.
4. When the menu surface is thinned, it can still present most public commands without treating the palette as a second shell.

## Verification

- manual review against:
  - [yzx_command_surface_backend_coupling.md](./yzx_command_surface_backend_coupling.md)
  - [architecture_map.md](../architecture_map.md)
- command-surface sanity check:
  - `nu -c 'use nushell/scripts/core/yazelix.nu *; help commands | where name =~ "^yzx( |$)" | select name description | sort-by name'`
- spec validation:
  - `nu nushell/scripts/dev/validate_specs.nu`

## Traceability

- Bead: `yazelix-qrnq.1`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
