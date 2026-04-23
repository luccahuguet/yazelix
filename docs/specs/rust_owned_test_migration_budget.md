# Rust-Owned Test Migration Budget

## Summary

This document records the delete-first budget for retiring governed Nushell
tests without losing the product contracts they defended.

That budget is now in a materially different state than the earlier planning
pass:

- governed Nu test surface under `nushell/scripts/dev/test_*.nu` is now `0`
- the strongest deterministic coverage from the former Nu omnibus files now
  lives in Rust-owned `nextest` suites
- the remaining `.nu` files in `nushell/scripts/dev/` are shell-heavy runners
  or validators, not governed tests

The rule remains strict:

- port only strong tests that defend real contracts, regressions, or invariants
- delete weak tests instead of porting them
- if a shell-heavy behavior still matters, do not preserve it in Nu by inertia;
  block it on the owning Rust migration or delete it

## Scope

In scope:

- former governed Nu test files under `nushell/scripts/dev`
- Rust-owned test coverage in `rust_core/yazelix_core`
- the shared Rust `nextest` harness support needed to replace strong Nu tests

Out of scope:

- sweep and E2E runner scripts that no longer define governed `def test_*`
  bodies
- deterministic validators, which are tracked in
  `docs/specs/maintainer_and_validator_nushell_budget.md`
- shell-heavy product behavior that still needs an owner cut before a strong
  Rust port is honest

## Current State

Measured on `2026-04-22`:

| Surface | Current size |
| --- | ---: |
| governed Nu `test_*.nu` files | `0` files |
| governed Nu `def test_*` functions | `0` |
| transitional shell-heavy runners | `881` LOC across `5` files |

The five surviving runner files are:

- `config_sweep_runner.nu`
- `historical_upgrade_notes_e2e_runner.nu`
- `stale_config_diagnostics_e2e_runner.nu`
- `upgrade_contract_e2e_runner.nu`
- `upgrade_summary_e2e_runner.nu`

Those runners are not governed tests and are budgeted separately under
`config_metadata/nushell_budget.toml`.

## Landed Replacement Coverage

The deleted Nu test ownership moved into these Rust-owned suites and support
layers:

- shared harness support in
  `rust_core/yazelix_core/tests/support/fixtures.rs`
- deterministic public control-plane coverage in
  `rust_core/yazelix_core/tests/yzx_control_runtime_surface.rs`
- deterministic workspace/session/doctor coverage in
  `rust_core/yazelix_core/tests/yzx_control_workspace_surface.rs`
- command-metadata and extern bridge coverage in
  `rust_core/yazelix_core/tests/yzx_core_command_metadata.rs`
- config normalization and helper-selection coverage in
  `rust_core/yazelix_core/tests/yzx_core_config_normalize.rs`
- Yazi materialization coverage in
  `rust_core/yazelix_core/src/yazi_materialization.rs`
- Zellij materialization coverage in
  `rust_core/yazelix_core/src/zellij_materialization.rs`

## Deleted Governed Nu Files

These governed Nu owners are intentionally gone:

- `test_yzx_commands.nu`
- `test_yzx_core_commands.nu`
- `test_yzx_generated_configs.nu`
- `test_yzx_workspace_commands.nu`
- `test_yzx_popup_commands.nu`
- `test_yzx_yazi_commands.nu`
- `test_yzx_doctor_commands.nu`
- `test_yzx_helix_doctor_contracts.nu`
- `test_shell_managed_config_contracts.nu`
- `test_helix_managed_config_contracts.nu`
- `test_zellij_plugin_contracts.nu`
- `test_yzx_maintainer.nu`
- `test_yzx_screen_commands.nu`

The delete-first point matters here: these files were not renamed into a
second-class governed lane. They were either replaced by stronger Rust-owned
coverage or deleted because their assertions were too weak to justify a port.

## Landed Migration Decisions

### `yazelix-rdn7.4.5.13`

The remaining strong public control-plane tests from
`test_yzx_core_commands.nu` moved to Rust-owned command tests around:

- `yzx config`
- `yzx update`
- `yzx run`
- `yzx status`

Weak route-listing and helper-discovery trivia was deleted.

### `yazelix-rdn7.4.5.7`

The remaining strong generated-config and materialization assertions moved to
Rust-owned coverage around:

- config normalization and helper selection
- Yazi materialization
- Zellij materialization

Wrapper-argv trivia and shell-owned launch details were not preserved in Nu.

### `yazelix-rdn7.4.5.9`

The strongest workspace/session/doctor assertions moved to Rust-owned coverage
around:

- `yzx cwd`
- `yzx reveal`
- typed doctor JSON output

Anything still fundamentally blocked on a surviving shell owner now waits for
that owner cut instead of surviving as a governed Nu test.

### `yazelix-rdn7.4.5.11`

The strong managed-config and extern-bridge contracts moved to Rust-owned
coverage around generated extern metadata and deterministic managed-config
behavior.

### `yazelix-rdn7.4.5.4`

After the strong Rust replacements landed, the redundant Nu files and helper
glue were deleted. There is no governed Nu steady-state survivor list.

## Strong-Only Migration Rules

1. Port only tests that defend explicit contracts, regressions, or invariants
2. Delete help-output trivia, command-discovery noise, and redundant fixture
   churn instead of porting it
3. If a strong behavior still needs a real shell/process boundary, block the
   Rust port on that owner cut rather than preserving a governed Nu test
4. New Rust-owned test coverage is `nextest`-first by default under
   `docs/specs/rust_test_hardening_tools_decision.md`

## Remaining Blocked Areas

Some behavior still needs future Rust-owned coverage, but it no longer gets to
survive as governed Nu tests:

- launch-time terminal and Ghostty behavior that still depends on the terminal
  shell floor
- detached launch probing and early terminal-death visibility
- remaining desktop/launcher and startup shell boundaries
- shell initializer and runtime-resolution flows that still need a cleaner
  owner cut before a strong Rust harness can defend them honestly

Those follow-ups should land directly into Rust once their owner cuts exist.

## Verification Gate

- `yzx_repo_validator validate-default-test-traceability`
- `yzx_repo_validator validate-rust-test-traceability`
- `nix develop -c cargo nextest run --profile ci --manifest-path rust_core/Cargo.toml -p yazelix_core`
- later plugin-owned Rust ports should use the same nextest-first policy

## Acceptance

1. Governed Nu tests are eliminated instead of grandfathered
2. The strongest former Nu test ownership is replaced by Rust-owned nextest
   suites
3. Weak tests are explicitly deleted instead of quietly preserved
4. Shell-heavy behavior that still matters is marked as blocked on owner cuts,
   not retained as a permanent Nu testing surface

## Traceability

- Bead: `yazelix-rdn7.4.5.1`
- Bead: `yazelix-rdn7.4.5.5`
- Bead: `yazelix-rdn7.4.5.4`
- Informed by: `docs/specs/governed_test_traceability_inventory.md`
- Informed by: `docs/specs/rust_nextest_harness_boundary.md`
- Informed by: `docs/specs/rust_test_hardening_tools_decision.md`
- Defended by: `yzx_repo_validator validate-specs`
