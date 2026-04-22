# Spec And Docs Contract Alignment Audit

## Summary

This audit reviews the documentation/spec layer after the first
contract-driven-development protocol pass.

The repo now has a working schema, validators, and a pilot, but the spec tree is
not yet canonical. Only a few live specs have indexed contract items. Several
new audit/contract files were not yet listed in `spec_inventory.md`. Some
historical docs still mention deleted Nushell owners, which is acceptable only
when they are clearly historical and not active planning.

## 1. Subsystem Snapshot

- subsystem name: docs, specs, inventories, and public protocol references
- purpose: distinguish live contracts, planning docs, historical notes,
  templates, and deletion candidates so implementation work starts from current
  ownership instead of stale transition prose
- user-visible entrypoints:
  - `docs/contract_driven_development.md`
  - `docs/spec_driven_workflow.md`
  - `docs/specs/spec_inventory.md`
  - `docs/subsystem_code_inventory.md`
  - README architecture/performance claims
- primary source paths:
  - `docs/specs/*.md`
  - `docs/*.md`
  - `README.md`
  - `nushell/scripts/dev/validate_specs.nu`
  - `nushell/scripts/dev/contract_traceability_helpers.nu`
- external references:
  - NASA traceability/verification-matrix guidance
  - Google Testing Blog test-size and E2E guidance
  - Rust testing documentation
  - cargo-nextest, cargo-mutants, and Rust Fuzz Book references carried in
    `docs/contract_driven_development.md`

## 2. Inventory Gaps Found

`spec_inventory.md` already separated live, planning, historical, and template
docs, but it was incomplete. The audit found these tracked specs missing from
the inventory before this update:

| Spec | Audit classification |
| --- | --- |
| `active_config_surface_owner_cut_budget.md` | Planning |
| `config_runtime_control_plane_canonicalization_audit.md` | Planning |
| `launch_startup_session_canonicalization_audit.md` | Planning |
| `runtime_env_config_state_shim_collapse_budget.md` | Planning |
| `setup_shellhook_welcome_terminal_canonicalization_audit.md` | Planning |
| `spec_inventory.md` | Live inventory |
| `terminal_launch_contract.md` | Live |
| `welcome_screen_style_contract.md` | Live |

This audit also adds these new planning inventories:

| Spec | Audit classification |
| --- | --- |
| `public_yzx_command_surface_canonicalization_audit.md` | Planning |
| `integration_glue_canonicalization_audit.md` | Planning |
| `maintainer_harness_canonicalization_audit.md` | Planning |
| `spec_docs_contract_alignment_audit.md` | Planning |
| `governed_test_traceability_inventory.md` | Planning |

## 3. Contract Item Coverage

Indexed contract items currently exist in only a few specs:

| Spec | Current item prefixes | Audit judgment |
| --- | --- | --- |
| `canonical_contract_item_schema.md` | `CCI-*` | schema is indexed enough for the protocol |
| `config_runtime_control_plane_contract_item_pilot.md` | `CRCP-*` | good pilot for mixed ownership |
| `welcome_screen_style_contract.md` | `FRONT-*` | good live front-door contract |
| `terminal_launch_contract.md` | `TLAUNCH-*` | good live terminal-launch contract |

Most other live specs still carry prose contracts plus `Defended by` lines. That
is acceptable as migration debt, but it blocks clean per-test contract mapping.

Priority live specs to convert next:

| Spec | Why it is high value |
| --- | --- |
| `runtime_root_contract.md` | many runtime/root tests still rely on broad file-level traceability |
| `runtime_dependency_preflight_contract.md` | startup/launch/doctor preflight behavior is central to deletion safety |
| `floating_tui_panes.md` | popup/menu/transient tests are default-lane behavior |
| `status_doctor_machine_readable_reports.md` | public status/doctor Rust cuts need exact JSON/report contracts |
| `startup_profile_scenarios.md` | profiling tests are high-signal and maintainer-critical |
| `rust_nushell_bridge_contract.md` | remaining bridge-collapse work needs item-level stop conditions |
| `pane_orchestrator_component.md` | plugin/live-session tests need contract IDs before wrapper deletion |
| `test_suite_governance.md` | validators currently defend policy, but per-test deletion needs item IDs |

## 4. Stale Deleted-Owner References

The audit found references to deleted or historical Nushell owners in docs. The
right action is not always deletion.

| Reference family | Current locations | Audit judgment |
| --- | --- | --- |
| `yzx/env.nu`, `yzx/run.nu` | `launch_bootstrap_rust_migration.md` | valid historical transition context; must not become active planning |
| `launch_state.nu` | `backend_capability_contract.md`, `launch_bootstrap_rust_migration.md`, `docs/streamlining_audit_2026_04.md` | historical only; current planning should use launch/startup audits |
| `config_migrations.nu`, `config_migration_transactions.nu` | upgrade notes, streamlining audit, deleted-owner inventory notes | upgrade/history references are valid; planning docs should not target them |
| `runtime_contract_checker.nu`, `generated_runtime_state.nu`, `terminal_configs.nu`, `helix_config_merger.nu`, `nushell_externs.nu` | migration matrix and code inventory | valid when explicitly described as deleted/landed |

The stop condition for cleanup is important: do not delete historical upgrade
records merely because they mention removed files. Rewrite or archive only when
the text looks like live planning or a current contract.

## 5. Must-Not-Lose Behavior

| Behavior | Current contract or source | Current owner | Current verification | Candidate surviving owner |
| --- | --- | --- | --- | --- |
| Maintainers can tell live contracts from planning and history | `spec_inventory.md`; `contract_driven_development.md` | Docs plus `validate_specs.nu` | `nu nushell/scripts/dev/validate_specs.nu` | same |
| New specs carry traceability to Beads and verification | `docs/spec_driven_workflow.md`; `validate_specs.nu` | docs and validator | `validate_specs.nu` | same |
| Contract items have stable IDs, owner, status, statement, and verification | `canonical_contract_item_schema.md` | docs and validator | `validate_specs.nu` | same |
| Tests can migrate from broad spec references to item IDs without deleting real behavior | `test_suite_governance.md`; quarantine file | docs and validators | `validate_default_test_traceability.nu`; `validate_rust_test_traceability.nu` | same |
| Historical docs can remain useful without driving current implementation | `spec_inventory.md` | docs inventory | manual review plus `validate_specs.nu` | same |

## 6. Delete-First Findings

### Delete Now

- No spec deletion is safe inside this audit.
- `spec_inventory.md` needed immediate inventory additions for files already
  present in `docs/specs/`.

### Bridge Layer To Collapse

- Broad file-level `Defended by` references should shrink toward contract-item
  IDs in live specs.
- Planning specs should stop acting as implicit live contracts.

### Spec Rewrite Or Retire

- Live prose specs should be converted in ranked batches, starting with specs
  that defend default-suite tests and bridge-collapse work.
- Historical docs with deleted-owner references should be hard-archived or
  rewritten only when they are ambiguous.

### Likely Survivors

- Historical migration docs that preserve decision history
- Upgrade-note guarded-change records
- Audit docs that feed `yazelix-rdn7.6`
- Template and protocol docs

### No-Go Deletions

- deleting upgrade history because it references deleted files
  - stop condition: keep when the reference documents real release history
- indexing every prose line
  - stop condition: only normative behavior, boundary, invariant, failure mode,
    ownership, and non-goal statements should get IDs

## 7. Follow-Up Beads

| Bead | Retained behavior | Deletion class | Candidate surviving owner | Verification that must still pass | Explicit stop condition |
| --- | --- | --- | --- | --- | --- |
| `yazelix-7upo` | live specs continue to defend real behavior while tests gain concrete item IDs | `spec_rewrite_or_retire` | live specs plus validators | `validate_specs`; default and Rust traceability validators | do not promote planning/history docs to live contracts by accident |
| `yazelix-l533` | useful history remains, but current planning stops citing deleted Nu owners as active targets | `spec_rewrite_or_retire` | spec inventory plus archived/historical docs | `validate_specs`; manual doc review | keep upgrade/history references when they are clearly historical |

## Verification

- manual review of `docs/specs/*.md`
- manual stale-reference search across `docs/`, `README.md`, `nushell/scripts`,
  and `rust_core`
- `nu nushell/scripts/dev/validate_specs.nu`

## Traceability

- Bead: `yazelix-rdn7.5.7`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
- Informed by: `docs/contract_driven_development.md`
- Informed by: `docs/specs/spec_inventory.md`
- Informed by: `docs/specs/canonical_contract_item_schema.md`
