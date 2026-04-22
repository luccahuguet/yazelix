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
  - `nushell/scripts/core/yazelix.nu`
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
| Compatibility Nu registry | `nushell/scripts/core/yazelix.nu` re-exports internal families and proxies help | historical debt | No longer public-root authority; best next delete-first target |
| Shell/process-heavy command bodies | `yzx/launch.nu`, `yzx/enter.nu`, `yzx/desktop.nu`, `core/yzx_session.nu` | intentional | Likely Nu/POSIX survivors for now |
| UX/demo/info command bodies | `yzx/menu.nu`, `popup.nu`, `screen.nu`, `tutor.nu`, `whats_new.nu`, `edit.nu`, `import.nu`, `dev.nu` | mixed | Keep until a family-level deletion budget exists |

## 4. Survivor Reasons

- Rust `yzx.rs`: `canonical_owner`
- Rust `public_command_surface.rs`: `canonical_owner`
- Rust `command_metadata.rs`: `canonical_owner`
- Rust `yzx_control.rs`: `canonical_owner`
- Nu `yzx/launch.nu`, `yzx/enter.nu`, and `core/yzx_session.nu`:
  `irreducible_shell_boundary`
- Nu `yzx/desktop.nu`: `external_tool_adapter`
- Nu `yzx/dev.nu`: `external_tool_adapter`
- Nu `yzx/edit.nu` and `yzx/import.nu`: `external_tool_adapter`
- Nu `yzx/menu.nu`, `yzx/popup.nu`, `yzx/screen.nu`, `yzx/tutor.nu`,
  and `yzx/whats_new.nu`: `canonical_owner` for product UX until a narrower
  deletion lane proves otherwise
- Nu `core/yazelix.nu`: `historical_debt`

## 5. Delete-First Findings

### Delete Now

- No product command body should be deleted inside this audit.
- `core/yazelix.nu` is the clearest next deletion candidate, but it needs a
  separate bead because tests, sweeps, and legacy module invocations still use
  it as a source-able compatibility registry.

### Bridge Layer To Collapse

- `core/yazelix.nu` no longer owns the public root, metadata, externs, or the
  main command-family route plan.
- Test helpers and sweep harnesses still source `core/yazelix.nu` instead of
  using the Rust `yzx` binary or direct modules.
- `INTERNAL_NU_FAMILIES` is now the real split boundary. It should shrink only
  when a family deletion removes or demotes a Nu owner end to end.

### Full-Owner Migration

- No broad public CLI migration is honest today.
- The next valid migration is family-specific: it must delete a Nu command-body
  or parser owner rather than moving shell execution into Rust.

### Likely Survivors

- launch, enter, restart, and desktop launch remain shell/process-heavy.
- menu, popup, screen, tutor, whats_new, edit, import, and dev remain
  UX/external-tool-heavy.
- Rust root routing, metadata, and extern sync should stay Rust-owned.

### No-Go Deletions

- Broad Clap rewrite
  - stop condition: reopen only after a new family-level cut materially shrinks
    `INTERNAL_NU_FAMILIES` or deletes `core/yazelix.nu`
- Moving launch/enter/restart execution to Rust
  - stop condition: only valid if a new deterministic subcore deletes a whole
    Nu owner instead of wrapping terminal, Zellij, or shell handoff logic
- Preserving old command names as compatibility aliases
  - stop condition: only keep aliases when explicitly requested; AGENTS command
    policy rejects aliases by default

## 6. Quality Findings

- duplicate owners:
  - root help/version exists in Rust and in `core/yazelix.nu`, though the Nu
    version is now compatibility-only
  - tests still use both the public Rust binary path and direct
    `core/yazelix.nu` sourcing
- missing layer problems:
  - no focused contract item currently names the split between public Rust root
    routing and direct internal Nu family execution
  - no explicit parity harness says when `core/yazelix.nu` may be deleted
- extra layer problems:
  - `core/yazelix.nu` re-export registry remains after Rust took over public
    route planning
- DRY opportunities:
  - tests and sweeps should stop rebuilding the old source-able command tree
    and use the same public root that users run
- weak or orphan tests:
  - command existence and help-shape tests should remain under suspicion unless
    they defend route ownership or a user-visible behavior
- only-known executable-defense tests:
  - `public_command_surface.rs` route-planning tests
  - `yzx_core_command_metadata.rs`
  - public root routing checks in `test_yzx_core_commands.nu`
  - menu catalog parity tests
- spec gaps:
  - no indexed contract item for the Rust root plus internal Nu helper split

## 7. Deletion Classes And Follow-Up Beads

| Bead | Retained behavior | Deletion class | Candidate surviving owner | Verification that must still pass | Explicit stop condition |
| --- | --- | --- | --- | --- | --- |
| `yazelix-f7hz` | public root routing, help/version, metadata, externs, and all intentionally Nu-owned families | `bridge_collapse` | Rust `yzx.rs` plus direct Nu modules | Rust route tests; command metadata tests; default suite; sweep path if touched | stop if any supported runtime path still needs source-able re-export semantics |

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
  - `nushell/scripts/core/yazelix.nu`
  - `nushell/scripts/yzx/*.nu`
- `nu nushell/scripts/dev/validate_specs.nu`

## Traceability

- Bead: `yazelix-rdn7.5.2`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
- Informed by: `docs/specs/v16_rust_cli_rewrite_evaluation.md`
- Informed by: `docs/specs/rust_migration_matrix.md`
