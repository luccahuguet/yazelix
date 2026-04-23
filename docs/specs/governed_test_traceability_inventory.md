# Governed Test Traceability Inventory

## Summary

This inventory maps the current governed test surface to the contract-driven
testing protocol after the delete-first Rust migration cut.

The important state change is simple:

- governed Nu test ownership is now gone
- first-party governed tests now live in Rust
- the remaining Nu files in `nushell/scripts/dev/` are non-governed runners or
  validators, not defended `def test_*` suites

## Current Counts

Counts from the post-cut audit on `2026-04-22`:

| Surface | Count |
| --- | ---: |
| governed Nu `test_*.nu` files under `nushell/scripts/dev` | `0` |
| governed Nu `def test_*` functions | `0` |
| Nu tests with nearby `# Contract:` markers | `0` |
| first-party Rust files containing `#[test]` outside `target/` | `45` |
| first-party Rust `#[test]` functions outside `target/` | `173` |
| Rust tests with nearby `// Contract:` markers | `13` |
| quarantined default-suite component files | `0` |

Validator status during the audit:

- `yzx_repo_validator validate-default-test-traceability` should pass
  with an empty governed Nu inventory
- `yzx_repo_validator validate-rust-test-traceability` remains the live
  governed-test ratchet

## Nu Surface Status

There is no governed Nu test inventory left to classify.

The surviving `.nu` files under `nushell/scripts/dev/` now fall into only two
classes:

1. non-governed shell-heavy runners
   - `config_sweep_runner.nu`
   - `historical_upgrade_notes_e2e_runner.nu`
   - `stale_config_diagnostics_e2e_runner.nu`
   - `upgrade_contract_e2e_runner.nu`
   - `upgrade_summary_e2e_runner.nu`
2. deterministic validators that still need Rust owner cuts

Those files still count toward the Nushell budget, but they are no longer part
of the governed test surface.

## Rust Test Inventory

| Rust bucket | Representative files | Current mapping | Audit classification |
| --- | --- | --- | --- |
| config/runtime/control-plane core | `yzx_core_config_normalize.rs`, `yzx_core_runtime_env.rs`, `yzx_core_owned_facts.rs`, `yzx_control_runtime_surface.rs` | `CRCP-*` items plus `Defends` and `Regression` markers | canonical governed owner for deterministic config/control-plane behavior |
| workspace/session/doctor control plane | `workspace_commands.rs`, `yzx_control_workspace_surface.rs` | `WSS-*`, `SOE-*`, `Defends`, and `Regression` markers | canonical governed owner for typed workspace/session truth and doctor JSON shape |
| command metadata and extern bridge | `yzx_core_command_metadata.rs`, `public_command_surface.rs`, `command_metadata.rs` | behavior/regression markers plus a smaller indexed item set | canonical governed owner for public route metadata and extern lifecycle |
| materialization and render plans | `yazi_materialization.rs`, `zellij_materialization.rs`, `runtime_materialization.rs`, `terminal_materialization.rs`, `ghostty_materialization.rs`, `helix_materialization.rs` | strong behavior/regression markers | canonical governed owner for deterministic generated-config and render-plan behavior |
| report renderers | `doctor_commands.rs`, `status_report.rs`, `doctor_config_report.rs`, `doctor_runtime_report.rs` | behavior/regression markers | canonical governed owner for machine-readable report shaping |
| pane orchestrator plugin | `active_tab_session_state.rs`, `sidebar_contract.rs`, `pane_contract.rs`, `transient_pane_contract.rs`, `workspace.rs` | behavior/regression/invariant markers | canonical governed owner for live plugin state and pane/session contracts |

## Landed Rust Replacements

The latest migration cut specifically added or expanded these Rust-owned
replacements:

- `yzx_control_runtime_surface.rs`
- `yzx_control_workspace_surface.rs`
- `yzx_core_command_metadata.rs`
- `yzx_core_config_normalize.rs`
- `yazi_materialization.rs`
- `zellij_materialization.rs`

These files now hold the strongest surviving coverage that was previously split
across `test_yzx_core_commands.nu`, `test_yzx_generated_configs.nu`,
`test_yzx_workspace_commands.nu`, `test_yzx_doctor_commands.nu`,
`test_yzx_yazi_commands.nu`, `test_shell_managed_config_contracts.nu`, and
adjacent Nu files.

## Only-Known Executable Defenses

Do not delete or weaken these without replacement coverage or explicit contract
retirement:

- Rust config normalization and helper-selection coverage
- Rust status/doctor JSON envelope coverage
- Rust workspace/session/sidebar truth coverage
- Rust extern bridge and public metadata coverage
- Rust Yazi/Zellij materialization and render-plan coverage
- Rust pane-orchestrator plugin contract coverage

## Weak-Test Policy After The Cut

The cut changes the policy boundary:

- weak Nu tests are deleted, not renamed into a shadow lane
- strong behavior that still matters must land as Rust-owned coverage
- shell-heavy runners are not evidence that a governed Nu test survivor list is
  acceptable

Future pruning should keep strengthening Rust-owned coverage instead of
rebuilding omnibus files.

## Follow-Up

- `yazelix-rdn7.4.5.4` closed the current cleanup wave by deleting the
  redundant Nu tests after Rust replacements landed
- `yazelix-rdn7.4.7` records the runner policy that first-party Rust tests are
  `nextest`-first by default
- future owner cuts should add new Rust coverage directly, not reintroduce Nu
  `test_*.nu` files

## Verification

- `yzx_repo_validator validate-default-test-traceability`
- `yzx_repo_validator validate-rust-test-traceability`
- manual count and marker review with `rg` across `nushell/scripts/dev`,
  `rust_core`, and `rust_plugins` excluding `target/`

## Traceability

- Bead: `yazelix-rdn7.4.1`
- Bead: `yazelix-rdn7.4.5.4`
- Defended by: `yzx_repo_validator validate-default-test-traceability`
- Defended by: `yzx_repo_validator validate-rust-test-traceability`
