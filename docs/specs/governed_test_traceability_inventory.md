# Governed Test Traceability Inventory

## Summary

This inventory maps the governed Nu and Rust test surface to the current
contract-driven-development protocol.

The suite is strong enough to protect current deletion work, and the first
contract-item migration batch has already drained the old default-suite file
quarantine. Most tests still carry a mix of `Defends`, `Regression`, and
`Invariant` markers rather than full per-test contract IDs, but the broad
file-level debt is now smaller and more explicit.

## Current Counts

Counts from the audit pass:

| Surface | Count |
| --- | ---: |
| governed Nu `test_*.nu` files under `nushell/scripts/dev` | `18` |
| governed Nu `def test_*` functions | `198` |
| Nu tests with nearby `# Contract:` markers | `16` |
| first-party Rust files containing `#[test]` outside `target/` | `38` |
| first-party Rust `#[test]` functions outside `target/` | `146` |
| Rust tests with nearby `// Contract:` markers | `5` |
| quarantined default-suite component files | `0` |

Validator status during the audit:

- `nu nushell/scripts/dev/validate_default_test_traceability.nu` passes cleanly
- `nu nushell/scripts/dev/validate_rust_test_traceability.nu` passes cleanly

## Nu Test Inventory

| File | Lane | Tests | Current mapping | Audit classification |
| --- | --- | ---: | --- | --- |
| `test_yzx_commands.nu` | default | runner | broad file-level specs plus component bundle | keep; default-suite aggregator, not per-test owner |
| `test_yzx_core_commands.nu` | default component | `37` | mixed `BRIDGE-*`, `ROOT-*`, and `SDR-*` markers plus regressions | keep; one weak command-discovery test was deleted and the public-root compatibility registry is gone |
| `test_yzx_doctor_commands.nu` | default component | `16` | mixed `SDR-*` markers plus doctor regressions | keep; public status/doctor report contract now indexed |
| `test_yzx_generated_configs.nu` | default component | `38` | top-level live spec refs plus `CRCP-*` on helper-resolution tests; many generated-config regressions | mixed: some mapped, many regression-only, keep likely 9/10 cases |
| `test_yzx_popup_commands.nu` | default component | `14` | `POP-*` item IDs plus regressions | keep; popup contract items now map the strongest deterministic popup checks |
| `test_yzx_screen_commands.nu` | default component | `6` | `FRONT-*` item IDs | mapped; good model for future conversions |
| `test_yzx_workspace_commands.nu` | default component | `29` | `WSS-*`, `NWS-*`, and `PWS-*` item IDs plus regressions | keep; launch/session/workspace contracts now have indexed items for the strongest current tests |
| `test_yzx_yazi_commands.nu` | default component | `7` | `WSS-*`, `SOE-*`, and live spec refs plus regressions | keep; integration and managed-editor boundary tests now have narrower item IDs |
| `test_helix_managed_config_contracts.nu` | maintainer/default-adjacent | `5` | policy file marker plus Helix/config regressions | keep; needs item IDs before deletion decisions |
| `test_shell_managed_config_contracts.nu` | maintainer/default-adjacent | `12` | policy file marker plus shell/runtime regressions | keep; only known defense for shell config and extern bridge |
| `test_yzx_helix_doctor_contracts.nu` | maintainer | `2` | policy file marker plus Helix doctor defends | keep; move to item IDs when doctor/Helix specs are indexed |
| `test_yzx_maintainer.nu` | maintainer | `26` | maintainer regressions and invariants | keep; release/update/profile/issue-sync defenses |
| `test_zellij_plugin_contracts.nu` | maintainer | `5` | policy file marker plus Zellij plugin regressions | keep; likely move some to plugin/materialization contract IDs |
| `test_config_sweep.nu` | sweep | runner | no governed `def test_*` functions | keep as sweep runner, not default test inventory |
| `test_historical_upgrade_notes_e2e.nu` | maintainer | runner | no governed `def test_*` functions | keep as E2E runner if still release-relevant |
| `test_stale_config_diagnostics_e2e.nu` | maintainer | runner | no governed `def test_*` functions | keep as E2E runner if still diagnostic-relevant |
| `test_upgrade_contract_e2e.nu` | maintainer | runner | no governed `def test_*` functions | keep as upgrade contract runner |
| `test_upgrade_summary_e2e.nu` | maintainer | runner | no governed `def test_*` functions | keep as whats-new/upgrade summary runner |

Classification summary:

- mapped to indexed items:
  - `test_yzx_screen_commands.nu`
  - `test_yzx_popup_commands.nu`
  - selected `test_yzx_workspace_commands.nu`
  - selected `test_yzx_yazi_commands.nu`
  - selected `test_yzx_generated_configs.nu` helper-resolution tests
- mapped to live prose specs but not item IDs:
  - many generated-config, doctor/status, shell config, and Zellij plugin tests
- regression-only or invariant-only:
  - most default component tests and maintainer release/update/profile tests
- quarantine:
  - no default-suite component files remain quarantined after the first indexed
    contract-item batch
- likely delete/demote candidates:
  - remaining tests whose only value is command discovery or help existence
    after the relevant public command surface and session specs get item IDs
  - repeated generated-config assertions that duplicate stronger Rust
    materialization tests
  - expensive default-lane checks that can move to maintainer or sweep once
    replacement contract coverage exists

## Rust Test Inventory

| Rust bucket | Representative files | Tests | Current mapping | Audit classification |
| --- | --- | ---: | --- | --- |
| config/runtime/control-plane core | `config_normalize.rs`, `config_state.rs`, `runtime_contract.rs`, `control_plane.rs`, `yzx_core_config_normalize.rs`, `yzx_core_runtime_env.rs` | many | `CRCP-*` on runtime env and config normalize tests; many `Defends`/`Regression` markers | keep; high-value deterministic Rust-owned logic |
| public yzx routing/metadata | `public_command_surface.rs`, `command_metadata.rs`, `yzx_core_command_metadata.rs`, `yzx_control.rs` | `17` | mostly `Defends` and `Regression`; few item IDs | keep; needs command-surface item IDs |
| public Rust-owned command families | `config_commands.rs`, `doctor_commands.rs`, `keys_commands.rs`, `support_commands.rs`, `workspace_commands.rs`, `update_commands.rs` where present | many | behavior/regression markers | keep; map to future item IDs before deleting Nu tests |
| materialization and render plans | `runtime_materialization.rs`, `terminal_materialization.rs`, `ghostty_materialization.rs`, `helix_materialization.rs`, `yazi_materialization.rs`, `zellij_materialization.rs`, render-plan tests | many | strong behavior/regression markers | keep; candidates for targeted Rust hardening tools |
| report renderers | `doctor_config_report.rs`, `doctor_helix_report.rs`, `doctor_runtime_report.rs`, `status_report.rs`, report integration tests | many | behavior markers | keep while public report output is Rust-owned |
| pane orchestrator plugin | `active_tab_session_state.rs`, `sidebar_contract.rs`, `pane_contract.rs`, `transient_pane_contract.rs`, `horizontal_focus_contract.rs`, `workspace.rs` | `22` | behavior/regression/invariant markers | keep; only known fast defenses for live plugin policies |

Classification summary:

- mapped to indexed items:
  - `CRCP-001` and `CRCP-002` tests in `yzx_core_config_normalize.rs` and
    `yzx_core_runtime_env.rs`
- regression-only or invariant-only:
  - route-planning, generated extern, materialization, plugin-state, and report
    rendering tests
- likely 9/10 keepers:
  - runtime materialization lifecycle tests
  - public route classifier tests
  - yazi/zellij render-plan error tests
  - pane orchestrator active-tab/sidebar/transient/focus contract tests
- candidate demotions:
  - tests that preserve byte compatibility with historical Nu output should be
    revisited after the corresponding legacy owner is deleted and the new
    contract is indexed

## Only-Known Executable Defenses

Do not delete these until replacement coverage or explicit contract retirement
exists:

- runtime-root and shell startup bootstrap behavior in
  `test_yzx_workspace_commands.nu`
- terminal launch command shape and `terminal.config_mode=user` behavior in
  `test_yzx_generated_configs.nu`
- detached launch probe timing and early-death visibility in
  `test_yzx_maintainer.nu`
- public status/doctor JSON and diagnostic behavior in
  `test_yzx_core_commands.nu` and `test_yzx_doctor_commands.nu`
- Home Manager takeover, desktop entry, update, and wrapper-owner behavior in
  `test_yzx_core_commands.nu`
- shell-managed config, generated extern bridge, and host-shell non-takeover
  behavior in `test_shell_managed_config_contracts.nu`
- pane-orchestrator live state behavior in Rust plugin tests
- Yazi/sidebar/editor integration behavior in
  `test_yzx_yazi_commands.nu` and `test_yzx_workspace_commands.nu`
- runtime materialization, generated-config, and render-plan Rust tests

## Weak Or Orphan Findings

The first cleanup pass already deleted one weak default-lane command-discovery
test from `test_yzx_core_commands.nu`. The remaining risky areas for
`yazelix-rdn7.4.2` are:

- default-lane command-discovery tests after `core/yazelix.nu` compatibility
  registry deletion is planned
- generated-config tests that duplicate Rust render-plan assertions after the
  Rust materialization contracts get item IDs
- broad policy-only file-level traceability in default components
- historical byte-compatibility tests that may become implementation trivia once
  the retained contract is rewritten around Rust ownership

## Required Contract IDs Before Test Deletion

Minimum item-ID batches before aggressive pruning:

| Area | Needed contract items | Why |
| --- | --- | --- |
| public command routing | Rust root, internal Nu helper boundary, metadata/extern lifecycle | prevents deleting the last route parity tests |
| runtime root and startup preflight | runtime root, startup/launch preflight, detached-launch probe | protects launch/session deletion lanes |
| status and doctor reports | JSON envelope, human summary, fix/no-fix split | protects Rust report-owner cuts |
| workspace/integration glue | plugin-owned sidebar identity, retarget response, Yazi sync, editor cwd | protects wrapper deletion work |
| generated config/materialization | Yazi/Zellij/terminal/Helix render-plan outputs and stale/repair states | enables deduping Nu and Rust tests |
| test governance | default-suite admission, quarantine, lane minimums | lets low-value tests be removed without weakening process guarantees |

## Follow-Up

- `yazelix-rdn7.4.2` should use this inventory to delete, demote, or quarantine
  weak and orphan tests without mass purging the only executable defenses.
- `yazelix-rdn7.4.3` now records the keep/reject decision for
  `cargo-mutants`, `cargo-fuzz`, and `cargo-nextest`; follow implementation
  work through `yazelix-fkgs` if the nextest pilot is worth doing.
- `yazelix-0qxa` landed the next workspace/session contract-item batch for
  workspace/session, shell-opened editor, and persistent/non-persistent window
  semantics. Future pruning should keep using those item IDs instead of
  reintroducing broad file-level traceability.
- `yazelix-rdn7.4.5.1` now records the next serious Nu-to-Rust migration budget
  in `rust_owned_test_migration_budget.md`.

## Verification

- `nu nushell/scripts/dev/validate_default_test_traceability.nu`
- `nu nushell/scripts/dev/validate_rust_test_traceability.nu`
- manual count and marker review with `rg` across `nushell/scripts/dev`,
  `rust_core`, and `rust_plugins` excluding `target/`

## Traceability

- Bead: `yazelix-rdn7.4.1`
- Defended by: `nu nushell/scripts/dev/validate_default_test_traceability.nu`
- Defended by: `nu nushell/scripts/dev/validate_rust_test_traceability.nu`
