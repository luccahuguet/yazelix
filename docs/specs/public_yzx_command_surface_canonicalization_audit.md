# Public Yzx Command Surface Canonicalization Audit

## Summary

This audit reviews the public `yzx` command surface after the Rust root,
Rust-owned command metadata, generated extern lifecycle, and several public
family owner cuts landed.

The conclusion is still a Clap no-go. The public root is already Rust-owned and
small. The remaining value is deleting one more real Nu owner seam, not adding a
framework above intentionally Nu-owned command bodies.

## 1. Subsystem Snapshot

- subsystem name: public `yzx` command routing, metadata, and Clap readiness
- purpose: route `yzx` argv, render root help/version, own public metadata,
  sync generated Nushell externs, and dispatch intentionally Nu-owned command
  families through explicit internal modules
- user-visible entrypoints:
  - `yzx`
  - `yzx --help`
  - `yzx --version`
  - all public command families listed in
    `rust_core/yazelix_core/src/public_command_surface.rs`
- primary source paths:
  - `rust_core/yazelix_core/src/bin/yzx.rs`
  - `rust_core/yazelix_core/src/bin/yzx_control.rs`
  - `rust_core/yazelix_core/src/public_command_surface.rs`
  - `rust_core/yazelix_core/src/command_metadata.rs`
  - `nushell/scripts/core/yzx_session.nu`
  - `nushell/scripts/yzx/*.nu`
  - `nushell/scripts/dev/test_yzx_core_commands.nu`
  - `rust_core/yazelix_core/tests/yzx_core_command_metadata.rs`
- external dependencies that matter:
  - packaged `bin/yzx`
  - packaged `libexec/yzx_control`
  - packaged `libexec/yzx_core`
  - Nushell internal modules for intentionally shell/UX-owned families

## 2. Must-Not-Lose Behavior

| Behavior | Current contract or source | Current owner | Current verification | Candidate surviving owner |
| --- | --- | --- | --- | --- |
| Public root help, version flags, command metadata, menu visibility, and generated extern content come from one source of truth | `docs/specs/v16_rust_cli_rewrite_evaluation.md`; `docs/specs/rust_nushell_bridge_contract.md` | Rust `public_command_surface.rs` and `command_metadata.rs` | `rust_core/yazelix_core/tests/yzx_core_command_metadata.rs`; `rust_core/yazelix_core/src/public_command_surface.rs` tests; `test_yzx_menu_catalog_tracks_live_exported_command_surface` | same |
| Rust-owned public families stay on the Rust control path and do not bounce through a generic Nu root | `docs/specs/v16_rust_cli_rewrite_evaluation.md` | Rust `yzx.rs` and `yzx_control.rs` | `classifies_rust_owned_control_family_at_root`; `test_public_yzx_root_routes_rust_control_family_without_direct_nu_route_modules` | same |
| Intentionally Nu-owned families remain reachable through direct internal modules while they still own shell/process or UX behavior | `docs/specs/rust_migration_matrix.md`; this audit | Rust route planner plus individual Nu modules | `plans_grouped_internal_family_to_direct_module`; `keeps_help_alias_behavior_for_grouped_internal_families`; workspace/popup/screen/default suite tests | Rust route planner plus direct Nu modules |
| Alias and missing-subcommand behavior does not drift while the command surface is split | `docs/specs/v16_rust_cli_rewrite_evaluation.md` | Rust `public_command_surface.rs` | `preserves_alias_and_missing_subcommand_contracts`; public Nu command tests | same |
| Generated extern bridge lifecycle stays Rust-owned and does not reintroduce Nushell command-tree probing | `docs/specs/rust_nushell_bridge_contract.md` | Rust `command_metadata.rs` | `generated_extern_bridge_is_rust_owned_and_current`; shell-managed config tests | same |

## 3. Canonical Owner Map

| Concern | Current owner or split boundary | Split kind | Audit judgment |
| --- | --- | --- | --- |
| Root routing and root help/version | Rust `yzx.rs` plus `public_command_surface.rs` | intentional | Canonical owner |
| Public command metadata, menu categories, extern rendering | Rust `command_metadata.rs` over `public_command_surface.rs` | intentional | Canonical owner |
| Rust control families | Rust `yzx_control.rs` and command modules | intentional | Canonical owner |
| Internal Nu family table | Rust `INTERNAL_NU_FAMILIES` maps families to concrete modules | temporary bridge debt | Acceptable until each family gets its own owner decision |
| Compatibility Nu registry | deleted under `yazelix-f7hz` | landed debt removal | The fake public Nu root is gone; direct Nu family modules are now the only internal helper path |
| Shell/process-heavy command bodies | `yzx/launch.nu`, `yzx/enter.nu`, `yzx/desktop.nu`, `core/yzx_session.nu` | intentional | Likely Nu/POSIX survivors for now |
| UX/demo/info command bodies | `yzx/menu.nu`, `dev.nu` | mixed | `yzx edit`, `yzx import`, `yzx screen`, `yzx tutor`, and `yzx whats_new` are now Rust-owned. Keep the surviving Nu surfaces only while they remain honest process/UX boundaries |

## 4. Survivor Reasons

- Rust `yzx.rs`: `canonical_owner`
- Rust `public_command_surface.rs`: `canonical_owner`
- Rust `command_metadata.rs`: `canonical_owner`
- Rust `yzx_control.rs`: `canonical_owner`
- Nu `yzx/launch.nu`, `yzx/enter.nu`, and `core/yzx_session.nu`:
  `irreducible_shell_boundary`
- Nu `yzx/desktop.nu`: `external_tool_adapter`
- Nu `yzx/dev.nu`: `external_tool_adapter`
- Nu `yzx/menu.nu`: `external_tool_adapter` for the surviving product UX
  shell/process seams
- Rust `edit_commands.rs` and `import_commands.rs`: `canonical_owner` for
  `yzx edit` and `yzx import`
- Rust `front_door_commands.rs` and `yzx_control.rs`: `canonical_owner` for
  `yzx screen`, `yzx tutor`, and `yzx whats_new`

## 5. Delete-First Findings

### Delete Now

- `yazelix-f7hz` landed the obvious delete-now cut:
  `nushell/scripts/core/yazelix.nu` is gone.
- No additional product command body should be deleted inside this audit.

### Bridge Layer To Collapse

- `INTERNAL_NU_FAMILIES` is now the real split boundary.
- The public root no longer has a generic fallback Nu registry; future public
  deletion work now has to remove a concrete surviving Nu family owner end to
  end.

### Full-Owner Migration

- No broad public CLI migration is honest today.
- The next valid migration is family-specific: it must delete a Nu command-body
  or parser owner rather than moving shell execution into Rust.

### Likely Survivors

- launch, enter, restart, and desktop launch remain shell/process-heavy.
- menu, popup, and dev remain
  UX/external-tool-heavy.
- Rust root routing, metadata, and extern sync should stay Rust-owned.

### No-Go Deletions

- Broad Clap rewrite
  - stop condition: reopen only after a new family-level cut materially shrinks
    `INTERNAL_NU_FAMILIES`
- Moving launch/enter/restart execution to Rust
  - stop condition: only valid if a new deterministic subcore deletes a whole
    Nu owner instead of wrapping terminal, Zellij, or shell handoff logic
- Preserving old command names as compatibility aliases
  - stop condition: only keep aliases when explicitly requested; AGENTS command
    policy rejects aliases by default

## 6. Quality Findings

- duplicate owners:
  - no remaining public-root registry duplicate survives after `yazelix-f7hz`
  - some tests still mix direct helper/module invocation with public Rust entry
    paths, but they no longer source a generic Nu root
- missing layer problems:
  - the owner split is now named by `BRIDGE-001`, but more family-specific
    public command item IDs would still make later deletions safer
- extra layer problems:
  - no generic re-export registry remains, which is the correct end state for
    the landed public-root cut
- DRY opportunities:
  - remaining tests should prefer the same public root or direct concrete
    modules that the product now uses, without reintroducing a shared fake root
- weak or orphan tests:
  - command existence and help-shape tests should remain under suspicion unless
    they defend route ownership or a user-visible behavior
- only-known executable-defense tests:
  - `public_command_surface.rs` route-planning tests
  - `yzx_core_command_metadata.rs`
  - public root routing checks in `test_yzx_core_commands.nu`
  - menu catalog parity tests
- spec gaps:
  - there is still no family-specific indexed contract item set for the
    surviving direct Nu helper modules under the Rust root

## 7. Deletion Classes And Follow-Up Beads

| Bead | Retained behavior | Deletion class | Candidate surviving owner | Verification that must still pass | Explicit stop condition |
| --- | --- | --- | --- | --- | --- |
| `yazelix-f7hz` | public root routing, help/version, metadata, externs, and all intentionally Nu-owned families | `bridge_collapse` | Rust `yzx.rs` plus direct Nu modules | Rust route tests; command metadata tests; default suite; sweep path if touched | landed |

Follow-up intentionally not created:

- a Clap implementation bead
  - the audit keeps the current no-go because adding Clap now would not delete
    enough owner surface to justify the dependency and rewrite cost.

## Verification

- manual review of:
  - `rust_core/yazelix_core/src/bin/yzx.rs`
  - `rust_core/yazelix_core/src/bin/yzx_control.rs`
  - `rust_core/yazelix_core/src/public_command_surface.rs`
  - `rust_core/yazelix_core/src/command_metadata.rs`
  - `nushell/scripts/yzx/*.nu`
- `yzx_repo_validator validate-specs`

## Traceability

- Bead: `yazelix-rdn7.5.2`
- Defended by: `yzx_repo_validator validate-specs`
- Informed by: `docs/specs/v16_rust_cli_rewrite_evaluation.md`
- Informed by: `docs/specs/rust_migration_matrix.md`
