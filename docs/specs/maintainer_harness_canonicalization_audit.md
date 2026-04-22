# Maintainer Harness Canonicalization Audit

## Summary

This audit separates product/runtime Nushell from maintainer, dev, validator,
benchmark, sweep, and packaging Nushell.

The maintainer surface is large, but deleting it does not directly shrink the
runtime product path. The delete-first goal here is narrower: remove stale
wrappers, keep harness logic out of product migration accounting, and harden the
few maintainer paths that still defend release-quality behavior.

## 1. Subsystem Snapshot

- subsystem name: maintainer, dev, validator, benchmark, and sweep harnesses
- purpose: run tests, sync issues, build plugins, update runtime pins, bump
  versions, validate specs/tests/packages, profile startup, and run config
  sweeps
- user-visible entrypoints:
  - `yzx dev test`
  - `yzx dev update`
  - `yzx dev bump`
  - `yzx dev sync_issues`
  - `yzx dev build_pane_orchestrator`
  - `yzx dev profile`
  - `yzx dev lint_nu`
- primary source paths:
  - `nushell/scripts/yzx/dev.nu`
  - `nushell/scripts/maintainer/*.nu`
  - `nushell/scripts/dev/*.nu`
  - `nushell/scripts/dev/sweep/*.nu`
  - `.github/workflows/*.yml`
  - `.pre-commit-config.yaml`
- external dependencies that matter:
  - `bd`
  - `gh`
  - `git`
  - Nix commands
  - `cargo`
  - `nu-lint`
  - terminal emulators for visual sweep paths

## 2. Current LOC And Owner Buckets

These are rough `wc -l` counts from the audit pass. They are sizing data, not a
deletion score.

| Bucket | Approx lines | Current role | Product deletion impact |
| --- | ---: | --- | --- |
| Product/runtime Nushell | about `11.8k` lines from the delete-first inventory | startup, launch, setup, public command bodies, integration glue | high |
| Dev/test Nushell | about `17.2k` lines under `nushell/scripts/dev` | tests, validators, sweeps, fixtures, demo helpers, package smoke checks | indirect |
| Maintainer Nushell | about `2.1k` lines under `nushell/scripts/maintainer` | issue sync, version bump, update workflow, readme sync, plugin build, test runner | indirect but release-critical |
| Sweep helpers | about `450` lines under `nushell/scripts/dev/sweep` plus `test_config_sweep.nu` | matrix and visual sweep orchestration | release-confidence lane |

The top maintainer/test files by size are:

| File | Approx lines | Audit role |
| --- | ---: | --- |
| `test_yzx_generated_configs.nu` | `2246` | default-suite component; real generated-config contract coverage |
| `test_yzx_core_commands.nu` | `2129` | default-suite component; public command and control-plane coverage |
| `test_yzx_workspace_commands.nu` | `1966` | default-suite component; launch/session/workspace coverage |
| `test_yzx_maintainer.nu` | `1880` | maintainer lane; release/update/profile regressions |
| `test_yzx_popup_commands.nu` | `1048` | default-suite component; transient/popup behavior |
| `test_yzx_doctor_commands.nu` | `919` | default-suite component; public doctor/status behavior |
| `test_shell_managed_config_contracts.nu` | `738` | maintainer/default-adjacent shell config behavior |
| `maintainer/update_workflow.nu` | `649` | release/update workflow owner |
| `validate_default_test_traceability.nu` | `520` | governance validator owner |
| `maintainer/test_runner.nu` | `413` | public `yzx dev test` harness owner |

## 3. Must-Not-Lose Behavior

| Behavior | Current contract or source | Current owner | Current verification | Candidate surviving owner |
| --- | --- | --- | --- | --- |
| Default suite runs explicit high-signal membership instead of globbing every `test_*.nu` | `docs/specs/test_suite_governance.md` | Nu `maintainer/test_runner.nu` plus `test_yzx_commands.nu` | `yzx dev test`; `validate_default_test_count_budget.nu`; `validate_default_test_traceability.nu` | same or smaller runner |
| Sweep and visual lanes remain explicit and separate from the default suite | `docs/specs/test_suite_governance.md` | Nu `test_config_sweep.nu` and `dev/sweep/*.nu` | sweep lane manual/targeted checks | same, but dispatch should be less dynamic |
| Version bump and release notes stay transactional and refuse dirty/invalid release states | `docs/specs/upgrade_notes_contract.md` | Nu `maintainer/version_bump.nu` | maintainer tests for bump and upgrade contracts | same |
| Update workflow refreshes runtime pins, runs canaries, and requires explicit activation mode for real updates | `docs/specs/runtime_distribution_capability_tiers.md`; maintainer tests | Nu `maintainer/update_workflow.nu` | maintainer update tests | same |
| Plugin build/sync keeps pane-orchestrator wasm rebuild requirements visible | AGENTS Rust plugin workflow | Nu `maintainer/plugin_build.nu` and legacy `dev/update_zellij_pane_orchestrator.nu` | maintainer tests and manual build command | `plugin_build.nu`; delete or demote wrapper |
| Contract/test/spec validators keep the ratchet cheap and deterministic | `docs/contract_driven_development.md` | Nu validators under `nushell/scripts/dev` | validator commands | same unless a Rust validator clearly deletes Nu logic |
| Profiling harness records the real startup boundaries and can compare cold/warm/desktop/launch scenarios | `docs/specs/startup_profile_scenarios.md` | Nu `yzx/dev.nu` plus `startup_profile.nu` | maintainer profile tests | same |

## 4. Canonical Owner Map

| Concern | Current owner or split boundary | Split kind | Audit judgment |
| --- | --- | --- | --- |
| Public `yzx dev` command surface | Nu `yzx/dev.nu` plus Rust metadata | intentional | Keep Nu command bodies; Rust already owns metadata |
| Test runner selection and logging | Nu `maintainer/test_runner.nu` | intentional with bridge debt | Keep but simplify sweep dispatch |
| Default suite test logic | Nu test files and Rust tests | intentional | Governed by test inventory, not this harness audit |
| Contract/spec/test validators | Nu validator scripts | intentional | Good Nu fit unless a Rust port deletes a whole validator |
| Issue/GitHub sync | Nu `maintainer/issue_sync.nu` and `issue_bead_contract.nu` | external_tool_adapter | Keep; `bd` and `gh` are shell tools |
| Version bump/update workflow | Nu maintainer modules | external_tool_adapter | Keep; Nix/git-heavy |
| Plugin wasm build/sync | Nu `plugin_build.nu` plus legacy wrapper | mixed | Keep canonical module; delete/demote thin wrapper |
| Demo recording/font helpers | Nu dev scripts | manual | Should not be counted as product runtime contract |

## 5. Delete-First Findings

### Delete Now

- No maintainer script should be deleted in this audit.
- Obvious candidates need a wrapper/demote bead because some are still the only
  documented command for manual workflows.

### Bridge Layer To Collapse

- `maintainer/test_runner.nu` uses dynamic `nu -c` module strings for sweep and
  visual dispatch. That is a shell-boundary smell and should be replaced with a
  direct script/module boundary if possible.
- The sweep dispatch path uses runtime-root assumptions that deserve a focused
  check. The audit did not find a local import for `get_yazelix_runtime_dir` in
  `maintainer/test_runner.nu`, so the bead should verify whether sweep-only and
  visual modes are currently healthy before editing.

### Full-Owner Migration

- No broad Rust maintainer rewrite is justified now.
- A Rust port would be valid only for a deterministic validator or parser that
  deletes a full Nu validator owner. It should not target `gh`, `bd`, `git`,
  Nix, or terminal-window orchestration just because the files are large.

### Likely Survivors

- `issue_sync.nu`, `issue_bead_contract.nu`, `version_bump.nu`,
  `update_workflow.nu`, `repo_checkout.nu`, `test_runner.nu`, and
  `plugin_build.nu`
- validators that scan source files and docs cheaply
- startup/profile harnesses that execute the real startup paths
- sweep helpers that launch matrix cases

### No-Go Deletions

- Deleting update/release automation
  - stop condition: these scripts are release-critical and have high-signal
    maintainer regressions
- Deleting issue sync
  - stop condition: GitHub/Beads contract still requires a local owner
- Moving sweep visual execution to Rust
  - stop condition: the hard part is external terminal/process orchestration

## 6. Quality Findings

- duplicate owners:
  - `dev/update_zellij_pane_orchestrator.nu` overlaps with
    `maintainer/plugin_build.nu`
  - `dev/update_zjstatus.nu` is called from update workflow and should be
    reviewed as either canonical vendoring owner or thin wrapper
  - package validators share helper logic through `nixpkgs_package_smoke.nu`,
    which is good; keep that as a shared maintainer helper
- missing layer problems:
  - no single maintainer-harness contract says which scripts are shipped
    runtime commands versus manual-only repo helpers
  - no cheap dry-run check currently proves sweep dispatch without opening
    visual windows
- extra layer problems:
  - demo recording scripts are listed in `nushell/scripts/README.md` as if they
    were normal runtime script surface
  - some wrapper scripts survive only as one-command forwarders
- DRY opportunities:
  - direct runner invocation for sweep lanes
  - fold thin update/plugin wrappers into canonical maintainer modules
- weak or orphan tests:
  - this audit does not adjudicate per-test quality; that belongs to
    `yazelix-rdn7.4`
- only-known executable-defense tests:
  - release bump dirty/tag/metadata tests
  - update activation and runtime pin sync tests
  - issue sync planning tests
  - startup profile harness tests
- spec gaps:
  - no indexed contract for maintainer harness ownership versus product runtime
    ownership

## 7. Deletion Classes And Follow-Up Beads

| Bead | Retained behavior | Deletion class | Candidate surviving owner | Verification that must still pass | Explicit stop condition |
| --- | --- | --- | --- | --- | --- |
| `yazelix-fg51` | default, lint-only, sweep, visual, all, profile, and new-window `yzx dev test` behavior | `bridge_collapse` | one explicit Nu test runner with direct dispatch | default suite; targeted sweep dispatch; syntax validation | keep external Nu execution if isolation is needed, but avoid interpolated command program strings |
| `yazelix-4ucy` | wasm build/sync, zjstatus vendoring, demo helpers if kept, nixpkgs smoke validation | `delete_now` / `no_go_record` | canonical maintainer modules or manual docs | maintainer tests and specific tool checks | do not delete the only documented release/package workflow entrypoint |

## Verification

- manual review of:
  - `nushell/scripts/yzx/dev.nu`
  - `nushell/scripts/maintainer/*.nu`
  - `nushell/scripts/dev/*.nu`
  - `nushell/scripts/dev/sweep/*.nu`
- `nu nushell/scripts/dev/validate_specs.nu`

## Traceability

- Bead: `yazelix-rdn7.5.6`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
- Informed by: `docs/specs/test_suite_governance.md`
- Informed by: `docs/specs/startup_profile_scenarios.md`
- Informed by: `docs/specs/upgrade_notes_contract.md`
