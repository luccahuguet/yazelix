# yzx Command Surface Backend Coupling

## Summary

Yazelix should classify the public `yzx` command surface by primary ownership instead of treating every subcommand as equally backend-coupled. The important buckets are:

- backend-required control-plane commands
- backend-agnostic workspace/config UX
- runtime-owned or distribution-owned surfaces
- genuinely mixed seams that should be refactored before a future `Yazelix Core` boundary or backend experiment

This audit is based on the real exported command surface from `nushell/scripts/core/yazelix.nu` and `nushell/scripts/yzx/*.nu`, not on docs text alone.

## Why

The recent backend and preflight contract work made one thing obvious: Yazelix does not have one flat CLI.

Some commands are fundamentally about backend activation and rebuild semantics:

- `yzx env`
- `yzx run`
- internal generated-state repair helpers

Some are mostly workspace or config UX:

- `yzx cwd`
- `yzx reveal`
- `yzx edit`
- `yzx import`

Some are really about installed runtime or distribution state:

- `yzx desktop install`

And a few important families still braid multiple owners together:

- `yzx launch`
- `yzx enter`
- `yzx restart`
- `yzx status`
- `yzx doctor`
- `yzx menu`

Without a written audit:

- future `Yazelix Core` planning will stay fuzzy
- backend experiments will keep arguing about the wrong command subset
- later Rust migration work will have no clear first-class seam
- commands like `status`, `doctor`, and `launch` will keep absorbing unrelated responsibilities

## Scope

- audit the public exported `yzx` command surface
- fold aliases into canonical command families
- classify each family as:
  - backend-required
  - backend-agnostic
  - runtime-owned/distribution
  - mixed/refactor-needed
- identify the most important mixed seams for later refactor work
- record likely keep/narrow/drop expectations for a future `Yazelix Core` discussion

## Source Of Truth

This audit uses the actual exported command surface from:

- `nushell/scripts/core/yazelix.nu`
- `nushell/scripts/yzx/*.nu`

Sanity check command:

```bash
nu -c 'use nushell/scripts/core/yazelix.nu *; help commands | where name =~ "^yzx( |$)" | select name description | sort-by name'
```

The audit intentionally excludes:

- maintainer-only `yzx dev *`
- exported helper functions that are not user-facing commands, such as:
  - `resolve_yzx_cwd_target`
  - `resolve_yzx_popup_command`
  - `resolve_yzx_popup_cwd`
  - `resolve_yzx_screen_style`
  - `get_yzx_screen_cycle_frames`

## Behavior

### Classification Rules

- `backend-required`
  - the command cannot honestly work without backend activation, rebuild, pack, or environment-materialization semantics
- `backend-agnostic`
  - the command primarily owns workspace UX or config-surface behavior and should survive a future backend swap with the same command meaning
- `runtime-owned/distribution`
  - the command primarily owns installed runtime state, generated runtime outputs, or distribution/integration surfaces rather than backend control-plane behavior
- `mixed/refactor-needed`
  - the command currently braids more than one of those owners together and should be treated as a refactor target rather than as a clean precedent

### Command-Family Matrix

| Family | Canonical commands \(aliases folded\) | Bucket | Why | Likely `Yazelix Core` disposition | Main code evidence |
| --- | --- | --- | --- | --- | --- |
| Root and informational surface | `yzx`, `yzx why`, `yzx sponsor`, `yzx whats_new` | backend-agnostic | These commands are informational, promotional, or release-summary surfaces. They do not depend on backend control-plane semantics. | Keep | `nushell/scripts/core/yazelix.nu`, `nushell/scripts/yzx/whats_new.nu` |
| Workspace actions | `yzx cwd`, `yzx reveal`, `yzx popup`, `yzx screen` | backend-agnostic | These commands primarily act on the current workspace/session, managed sidebar, popup pane, or visual UX. They may rely on session state or configured tools, but not on backend activation semantics as their main owner. | Keep | `nushell/scripts/core/yazelix.nu`, `nushell/scripts/yzx/popup.nu`, `nushell/scripts/yzx/screen.nu` |
| Discoverability and training | `yzx keys`, `yzx keys yzx`, `yzx keys yazi`, `yzx keys hx`, `yzx keys helix`, `yzx keys nu`, `yzx keys nushell`, `yzx tutor`, `yzx tutor hx`, `yzx tutor helix`, `yzx tutor nu`, `yzx tutor nushell` | backend-agnostic | These are educational and discoverability surfaces. Their meaning should survive any backend reshaping. | Keep | `nushell/scripts/yzx/keys.nu`, `nushell/scripts/yzx/tutor.nu` |
| Config-surface management | `yzx config`, `yzx config reset`, `yzx edit`, `yzx edit config`, `yzx edit packs`, `yzx import`, `yzx import zellij`, `yzx import yazi`, `yzx import helix` | backend-agnostic | These commands primarily own user-managed config surfaces and migration/import flows. They may read shipped templates or migration metadata, but they are not backend control-plane commands. | Keep | `nushell/scripts/yzx/config.nu`, `nushell/scripts/yzx/edit.nu`, `nushell/scripts/yzx/import.nu` |
| Backend control plane | `yzx env`, `yzx run` | backend-required | These commands directly own environment activation and noninteractive runtime entry semantics. Their behavior is defined by the backend contract. | Narrow, not drop | `nushell/scripts/yzx/env.nu`, `nushell/scripts/yzx/run.nu`, `nushell/scripts/utils/environment_bootstrap.nu`, `nushell/scripts/utils/generated_runtime_state.nu` |
| Backend package and store surfaces | `yzx packs` | backend-required | This command remains tightly coupled to backend/package composition and current Nix/runtime store semantics. | Likely narrow sharply | `nushell/scripts/yzx/packs.nu` |
| Installed runtime and distribution maintenance | `yzx desktop install`, `yzx desktop uninstall`, `yzx desktop launch`, `yzx update`, `yzx update nix` | runtime-owned/distribution | These commands own desktop-entry integration, runtime/distribution guidance, or adjacent repair guidance. They are about shipped/runtime distribution state more than backend activation semantics. | Keep selected surfaces; `update nix` depends on future product policy | `nushell/scripts/yzx/desktop.nu`, `nushell/scripts/core/yazelix.nu`, `nushell/scripts/setup/zellij_plugin_paths.nu` |
| Session launch and restart | `yzx launch`, `yzx enter`, `yzx restart` | mixed/refactor-needed | `yzx launch` owns new-window startup and terminal dispatch; `yzx enter` owns current-terminal startup. They still share backend refresh/re-entry and workspace/session bootstrap concerns with `yzx restart`, but the public command surface is clearer once current-terminal startup stops living under `launch`. | Keep, but split internally | `nushell/scripts/yzx/launch.nu`, `nushell/scripts/yzx/enter.nu`, `nushell/scripts/core/start_yazelix.nu`, `nushell/scripts/core/yazelix.nu` |
| Health and inspection | `yzx status`, `yzx doctor` | mixed/refactor-needed | `status` mixes config summary and backend/runtime freshness. `doctor` mixes shared runtime preflight, install/distribution health, version drift, and workspace/plugin diagnostics. | Keep, but split responsibilities more clearly | `nushell/scripts/core/yazelix.nu`, `nushell/scripts/utils/doctor.nu`, `nushell/scripts/utils/runtime_contract_checker.nu` |
| Command palette | `yzx menu` | mixed/refactor-needed | The picker UI is backend-agnostic, but the command dispatch path shells back into the runtime command module and spans every other family. It is a thin mixed seam today. | Keep, but reduce dispatch coupling | `nushell/scripts/yzx/menu.nu` |

### Alias Folding Notes

- `yzx keys yzx` is the alias form of the default `yzx keys` view.
- `yzx keys helix` is the alias of `yzx keys hx`.
- `yzx keys nushell` is the alias of `yzx keys nu`.
- `yzx tutor helix` is the alias of `yzx tutor hx`.
- `yzx tutor nushell` is the alias of `yzx tutor nu`.
- `yzx edit config` and `yzx edit packs` are specialized leaves of the same `yzx edit` family.
- `yzx import zellij`, `yzx import yazi`, and `yzx import helix` are leaves of the same import family.
- `yzx update nix` is the only remaining concrete update leaf under the broader `yzx update` distribution-guidance family.

### Mixed-Seam Shortlist

The highest-value mixed families to refactor later are:

1. `yzx launch` / `yzx enter` / `yzx restart`
   - keep `launch` scoped to new-window behavior, `enter` scoped to current-terminal startup, and continue splitting backend refresh/re-entry from workspace bootstrap and terminal dispatch
2. `yzx status` / `yzx doctor`
   - split concise runtime status from heavier install/integration diagnostics
3. `yzx menu`
   - keep the picker UX backend-agnostic while making dispatch and family metadata more explicit

### First-Pass Downstream Guidance

- `yazelix-qv8c`
  - should treat the `backend-agnostic` and selected `runtime-owned/distribution` families as the strongest `Yazelix Core` candidates
- `yazelix-qow`
  - should care primarily about the `backend-required` and `mixed/refactor-needed` families
- later backend-adapter and Rust work
  - should focus first on the mixed families rather than rewriting already-clean backend-agnostic surfaces

## Non-goals

- defining backend capability buckets again
- defining launch-time dependency ownership again
- promising final `Yazelix Core` command parity
- auditing maintainer-only `yzx dev *` in the same matrix
- rewriting the mixed seams yet

## Acceptance Cases

1. When a later bead asks whether a command family is fundamentally backend-bound, the answer can be taken from this matrix instead of guessed from implementation trivia.
2. When a future `Yazelix Core` discussion asks which commands are the best keep candidates, the answer clearly favors backend-agnostic and selected runtime-owned/distribution families.
3. When a later refactor asks which public commands still mix backend, runtime, and workspace concerns, the shortlist clearly identifies `launch`, `enter`, `restart`, `status`, `doctor`, and `menu`.
4. When a later backend experiment evaluates itself against the CLI, it does not need to treat the whole `yzx` surface as one undifferentiated requirement.

## Verification

- manual review against:
  - [backend_capability_contract.md](./backend_capability_contract.md)
  - [runtime_dependency_preflight_contract.md](./runtime_dependency_preflight_contract.md)
  - [architecture_map.md](../architecture_map.md)
- manual command-surface sanity check:
  - `nu -c 'use nushell/scripts/core/yazelix.nu *; help commands | where name =~ "^yzx( |$)" | select name description | sort-by name'`
- manual code review of the main family owners:
  - `nushell/scripts/core/yazelix.nu`
  - `nushell/scripts/yzx/launch.nu`
  - `nushell/scripts/yzx/menu.nu`
  - `nushell/scripts/yzx/env.nu`
  - `nushell/scripts/yzx/run.nu`
  - `nushell/scripts/yzx/packs.nu`
  - `nushell/scripts/yzx/desktop.nu`
  - `nushell/scripts/utils/doctor.nu`
- CI/spec check:
  - `nu nushell/scripts/dev/validate_specs.nu`

## Traceability

- Bead: `yazelix-d4pw`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`

## Open Questions

- Should `yzx packs` remain part of the long-term public product surface if `Yazelix Core` becomes a fixed default-stack bundle rather than a flexible pack-driven environment?
- Should `yzx update nix` stay part of the main public surface, or should it eventually move behind a narrower Nix-distribution or maintainer boundary?
