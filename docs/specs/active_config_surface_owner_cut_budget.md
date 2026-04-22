# Active Config-Surface Owner Cut Budget

## Summary

This document defines the delete-first budget for collapsing the duplicate
active config-surface owner seam between
`nushell/scripts/utils/config_surfaces.nu` and
`rust_core/yazelix_core/src/active_config_surface.rs`.

The goal is to make Rust the single owner of active managed config-surface
resolution while preserving the live v15.4 behavior:

- canonical `user_configs/yazelix.toml` ownership
- fail-fast duplicate and legacy-root detection
- default bootstrap from the shipped template
- managed Taplo support synchronization
- Home Manager parity with the shipped default config

This bead is planning-only. It does not delete code. It defines what may be
deleted, what can survive only as a non-owning shell seam, and what would make
the owner cut dishonest.

## Scope

- `nushell/scripts/utils/config_surfaces.nu`
- every live Nu caller of `load_active_config_surface` or
  `reconcile_primary_config_surfaces`
- Rust callers already using `active_config_surface.rs`
- adjacent small helpers that may need relocation if `config_surfaces.nu`
  stops being an owner file

Out of scope:

- wider `common.nu` runtime/config/state-root duplication
- deletion of `yzx_core_bridge.nu`
- launch/startup shell-orchestration cuts beyond the active-surface boundary

## Current Caller Inventory

## Nu callers that still use active-surface ownership

| Caller | What it needs today | Real shell-local need | Budget judgment |
| --- | --- | --- | --- |
| `utils/config_parser.nu` | `config_file` and `default_config_path` for Rust normalize requests | none beyond consuming structured paths | should move to a Rust-owned active-surface query |
| `utils/config_state.nu` | `config_file`, `default_config_path`, and managed-main path for state hashing/recording | no; the state logic is already Rust-owned | should move to a Rust-owned request boundary |
| `core/launch_yazelix.nu` | active config path/default path for terminal materialization and Ghostty reroll helpers | shell orchestration is real, active-surface ownership is not | should consume Rust-owned active-surface facts |
| `utils/doctor_fix.nu` | active config path for repair commands plus default-copy helper | shell-side fix flow is real; active-surface ownership is not | split: Rust-owned path resolution, tiny file-copy helper may survive elsewhere |
| `dev/materialization_dev_helpers.nu` | active config path/default path for internal helper calls | no | should consume Rust-owned active-surface facts |
| `yzx/edit.nu` | canonical managed main-config path | yes, it launches an editor, but not an owner-level need | can use a narrower path query instead of full active-surface ownership |

## Nu callers that only use small path/file helpers

| Caller | Current helper use | Budget judgment |
| --- | --- | --- |
| `utils/zjstatus_widget.nu` | `get_main_user_config_path` | path join helper should move out of `config_surfaces.nu` |
| `utils/config_schema.nu` | `get_main_user_config_path`, `load_config_surface_from_main` | pure path/file helper; should move out of `config_surfaces.nu` |
| `maintainer/update_workflow.nu` | `copy_default_config_surfaces`, `load_config_surface_from_main`, `get_main_user_config_path` | maintainer file/template helpers can survive, but not in an active-surface owner file |

## Rust callers already on the canonical owner

| Caller | Current usage | Budget judgment |
| --- | --- | --- |
| `config_commands.rs` | `resolve_active_config_paths`, `primary_config_paths` | already correct |
| `doctor_commands.rs` | `resolve_active_config_paths` | already correct |
| `control_plane.rs` | `resolve_active_config_paths` | already correct |
| `runtime_materialization.rs` | `primary_config_paths` | already correct |
| `doctor_config_report.rs` | `primary_config_paths`, `validate_primary_config_surface`, `ensure_managed_taplo` | already correct |

## Surviving Owner Decision

The surviving owner for active managed config-surface rules should be Rust
`active_config_surface.rs`.

That means these rules must stop living in Nushell:

- canonical `user_configs` path resolution
- duplicate user-config versus legacy-root rejection
- fallback bootstrap from `yazelix_default.toml`
- managed Taplo support synchronization
- active main-config path selection with `YAZELIX_CONFIG_OVERRIDE`

Nu may still own shell-local behavior after the cut, but only when it is not
re-deciding active config-surface semantics. Examples:

- launching an editor for a resolved path
- copying a template file into a managed path for maintainer workflows
- reading TOML from an already resolved explicit path

## Narrowest Honest Surviving Nu Seam

After the owner cut, any surviving Nu helpers should be limited to one of these
categories:

- path-only helpers
  - example: `get_main_user_config_path`
- file-copy helpers for Yazelix-owned managed paths
  - example: current `copy_default_config_surfaces`
- file-load helpers for already explicit paths
  - example: current `load_config_surface_from_main`

Those helpers must not:

- validate legacy-root versus canonical ownership
- decide whether to bootstrap the managed main config
- synchronize Taplo support
- choose the active main config surface

If those are the only survivors, `config_surfaces.nu` should be deleted and the
surviving non-owning helpers should move to a narrower file such as
`config_paths.nu` or `config_files.nu`.

## Deletion Budget

## Delete or demote from `config_surfaces.nu`

- `get_primary_config_paths`
- `ensure_current_primary_config_surface`
- `reconcile_primary_config_surfaces`
- `resolve_active_config_paths`
- `load_active_config_surface`

These are owner functions and should move entirely behind Rust.

## Survive only if relocated as non-owning helpers

- `get_main_user_config_path`
- `get_managed_taplo_support_path`
- `copy_default_config_surfaces`
- `load_config_surface_pair`
- `load_config_surface_from_main`

`normalize_config_surface_path` has no live repo callers and should be deleted
unless a real use appears during implementation.

## Candidate Rust interface for Nu callers

`yazelix-izwm.2` should use one explicit Rust-owned query surface for Nu
callers that still need active config-surface facts. The exact command shape is
still implementation work, but the budget is:

- the interface must return structured paths, not human prose
- the interface must preserve bootstrap, Taplo sync, and fail-fast validation
- Nu callers should not parse `yzx config` human output to recover those paths
- the interface may be an internal helper command or a narrow structured
  control-plane surface, but it must not create a second semantic owner

## Verification Gate

The owner cut is only honest if all of these still hold afterward:

- `nu nushell/scripts/dev/validate_config_surface_contract.nu`
- `nu nushell/scripts/dev/test_yzx_core_commands.nu`
  - especially the `yzx config --path` and `yzx config reset` cases
- `nu nushell/scripts/dev/test_yzx_generated_configs.nu`
  - especially managed config bootstrap and Taplo support cases
- Rust coverage exists for the surviving owner
  - current gap: `active_config_surface.rs` still lacks dedicated unit tests and
    `yazelix-izwm.2` should add them or explain why adjacent coverage is enough

## Stop Conditions

`yazelix-izwm.2` must stop and record a no-go if any of these turn out to be
true:

- the only way to remove `config_surfaces.nu` is to make Nu parse human-facing
  output from a public command
- the surviving Rust interface cannot preserve default bootstrap or Taplo sync
  without implicitly taking ownership of user-managed external config files
- a caller still needs mixed shell side effects and active-surface ownership in
  one inseparable step

If that happens, the fallback is not "keep the whole owner file." The fallback
is to record the smallest remaining non-owning shell seam explicitly.

## Acceptance

1. The surviving owner is explicit
2. Every live caller is classified as owner-free, shell-local, or blocked
3. The functions that must leave `config_surfaces.nu` are named directly
4. The functions that may survive only as non-owning helpers are named directly
5. The verification gate and explicit stop conditions are recorded before
   implementation starts

## Traceability

- Bead: `yazelix-izwm.1`
- Informed by: `docs/specs/config_runtime_control_plane_canonicalization_audit.md`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
- Defended by: manual review of the cited Nu and Rust caller inventory
