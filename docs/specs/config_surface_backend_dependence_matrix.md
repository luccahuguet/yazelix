# Config Surface Backend Dependence Matrix

> Status: Historical pre-v15-trim planning note.
> This matrix was written before the trimmed v15 branch removed `yazelix_packs.toml`, backend-owned rebuild knobs, and related pack-graph semantics from the current user contract.
> Do not treat it as the current branch contract. See [v15_trimmed_runtime_contract.md](./v15_trimmed_runtime_contract.md).

## Summary

Yazelix should classify its config surfaces by execution owner, not just by file.

The important split is:

- `yazelix.toml` is mixed: some settings are workspace/session UX, some are backend/devenv inputs, and some are host-tool locator seams
- `yazelix_packs.toml` is currently backend-owned across the board because the whole surface shapes the materialized package graph
- package-runtime-only Yazelix keeps both config files
- backend-free workspace-only Yazelix keeps only the settings that still have a clear owner without backend materialization

## Why

This audit is needed for the simplification lane because “keep the config files” is still too vague.

The real planning questions are:

- which settings are rebuild-relevant today
- which settings still make sense when installation ownership is deleted but backend ownership remains
- which settings would have to narrow, move, or disappear if backend ownership is also deleted

## Sources

This matrix is grounded in the maintained source-of-truth artifacts:

- [main_config_contract.toml](../../config_metadata/main_config_contract.toml)
- [pack_catalog_contract.toml](../../config_metadata/pack_catalog_contract.toml)
- [config_state.nu](../../nushell/scripts/utils/config_state.nu)
- [config_parser.nu](../../nushell/scripts/utils/config_parser.nu)
- [config_surface_and_launch_profile_contract.md](./config_surface_and_launch_profile_contract.md)
- [runtime_ownership_reduction_matrix.md](./runtime_ownership_reduction_matrix.md)
- [backend_free_workspace_slice.md](./backend_free_workspace_slice.md)

## Classification Rules

Use these buckets:

1. Workspace/session-owned
   - still meaningful if Yazelix stops materializing the backend
2. Backend/devenv-owned
   - meaningful only while Yazelix still owns environment materialization, rebuild, or package composition
3. Host-tool locator seam
   - points at an external tool and may survive a backend-free slice, but its current rebuild behavior may be an implementation detail rather than an intrinsic requirement
4. Launch/integration policy
   - tied to entry, terminal dispatch, or runtime-facing host integration; these usually survive package-runtime-only but become questionable in a backend-free slice

## Matrix

| Family | Keys | Current owner | Rebuild-relevant today | Package-runtime-only | Backend-free workspace-only |
| --- | --- | --- | --- | --- | --- |
| Backend graph and build knobs | `core.recommended_deps`, `core.yazi_extensions`, `core.yazi_media`, `core.max_jobs`, `core.build_cores` | backend/devenv-owned | yes | keep | move out of Yazelix, narrow sharply, or drop |
| Helix runtime sourcing | `helix.mode`, `helix.runtime_path` | backend plus editor runtime sourcing | yes | keep | narrow or move; only survives if a future host-owned editor/runtime contract replaces the current backend owner |
| Editor command locator | `editor.command` | host-tool locator seam | yes today | keep | keep, but stop treating it as backend-owned rebuild input by default |
| Editor/sidebar workspace UX | `editor.initial_sidebar_state`, legacy `editor.enable_sidebar`, `editor.sidebar_width_percent` | workspace/session-owned | no | keep | keep |
| Shell entry choice | `shell.default_shell` | launch/integration policy | no | keep | narrow or drop unless current-terminal entry remains a first-class product behavior |
| Extra shell package graph | `shell.extra_shells` | backend/devenv-owned | yes | keep | move out of Yazelix or drop |
| Terminal package and dispatch selection | `terminal.terminals` | backend plus launch/integration | yes | keep | narrow or move unless Yazelix still owns launch dispatch |
| Terminal host-integration policy | `terminal.manage_terminals`, `terminal.config_mode` | launch/integration policy | no | keep | narrow or drop |
| Terminal visual styling | `terminal.ghostty_trail_color`, `terminal.ghostty_trail_effect`, `terminal.ghostty_trail_duration`, `terminal.ghostty_mode_effect`, `terminal.ghostty_trail_glow`, `terminal.transparency` | integration/UI policy | no | keep | keep only if Yazelix still owns terminal config generation; otherwise move outward |
| Welcome and runtime UX | `core.debug_mode`, `core.skip_welcome_screen`, `core.show_macchina_on_welcome`, `core.welcome_style`, `core.welcome_duration_seconds` | runtime UX | no | keep | mostly keep if Yazelix still owns startup UI; otherwise narrow |
| Generated-state repair UX | `core.refresh_output` | backend/devenv-owned | no | keep | drop if the public refresh surface disappears |
| Zellij workspace/session UX | all `zellij.*` keys | workspace/session-owned | no | keep | mostly keep |
| Yazi command locators | `yazi.command`, `yazi.ya_command` | host-tool locator seam | no | keep | keep if Yazi remains a host/runtime prerequisite |
| Yazi workspace UX | `yazi.plugins`, `yazi.theme`, `yazi.sort_by` | workspace/session-owned | no | keep | keep if Yazi remains in the surviving workspace slice |
| Pack sidecar surface | `packs.enabled`, `packs.user_packages`, `packs.declarations` from `yazelix_packs.toml` | backend/devenv-owned | yes | keep | move out, narrow heavily, or drop |

## Exact Rebuild-Relevant Inputs Today

The current rebuild-sensitive config set is small and explicit.

Main config fields marked `rebuild_required = true` in the main contract:

- `core.recommended_deps`
- `core.yazi_extensions`
- `core.yazi_media`
- `core.max_jobs`
- `core.build_cores`
- `helix.mode`
- `helix.runtime_path`
- `editor.command`
- `shell.extra_shells`
- `terminal.terminals`

Pack-sidecar rebuild paths from the main-contract `rebuild.extra_paths` list:

- `packs.enabled`
- `packs.declarations`
- `packs.user_packages`

`config_state.nu` then combines those config hashes with:

- `devenv.lock`
- `devenv.nix`
- `devenv.yaml`
- the active runtime-root hash

Everything else in the current config surface is intentionally not rebuild-sensitive today.

## Reading Guidance

### Package-Runtime-Only Yazelix

This cut keeps backend ownership.

That means:

- both config files still make sense
- all current rebuild-relevant settings still have a live execution owner
- most of the work is deleting installer/distribution ownership, not redesigning config meaning

### Backend-Free Workspace-Only Yazelix

This cut deletes backend ownership too.

That means:

- the entire pack sidecar becomes unstable as a first-class Yazelix surface
- backend graph knobs and refresh/build knobs lose meaning
- workspace/session and tool-locator settings are the main survivors

## Important Current Mismatch

`editor.command` is the clearest field where current rebuild ownership and long-term meaning diverge.

Today:

- it is rebuild-relevant because the current backend/launch materialization path consumes it

But conceptually:

- it is also a durable editor-selection surface that still matters in a backend-free workspace slice

So this field should be read as:

- keep the setting
- question the current rebuild ownership if backend reduction continues

## Non-goals

- splitting the config file format now
- changing any current parser behavior in this spec alone
- dropping packs in the current product
- claiming every launch/integration policy knob survives a backend-free future unchanged

## Acceptance Cases

1. A maintainer can answer which config families are backend-bound versus workspace-bound without re-reading parser code.
2. A maintainer can answer which fields are rebuild-relevant today from one explicit list.
3. A maintainer can explain why package-runtime-only keeps both config files while backend-free work threatens packs.
4. Later simplification work can point at one stable matrix instead of reclassifying the config surface ad hoc.

## Verification

- contract review:
  - [main_config_contract.toml](../../config_metadata/main_config_contract.toml)
  - [pack_catalog_contract.toml](../../config_metadata/pack_catalog_contract.toml)
- behavioral contract review:
  - [config_surface_and_launch_profile_contract.md](./config_surface_and_launch_profile_contract.md)
  - [runtime_ownership_reduction_matrix.md](./runtime_ownership_reduction_matrix.md)
  - [backend_free_workspace_slice.md](./backend_free_workspace_slice.md)
- validation:
  - `yzx_repo_validator validate-specs`
  - `yzx_repo_validator validate-config-surface-contract`

## Traceability

- Bead: `yazelix-4buc.6`
- Defended by: `yzx_repo_validator validate-specs`
- Defended by: `yzx_repo_validator validate-config-surface-contract`
