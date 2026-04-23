# Maintainer Harness Canonicalization Audit

## Summary

This audit separates product/runtime Nushell from maintainer, dev, validator,
benchmark, sweep, and packaging Nushell.

The maintainer surface is large, but deleting it does not directly shrink the
runtime product path. The delete-first goal here is narrower: remove stale
wrappers, keep harness logic out of product migration accounting, and harden the
few maintainer paths that still defend release-quality behavior.

The current state is already materially slimmer than the original audit pass:

- the governed Nu test omnibus files are deleted
- `yzx dev test` now dispatches fixed Rust `nextest` suites by inventory
- the remaining Nu test-related files are shell-heavy runners and validators,
  not canonical governed test owners

`yazelix-4ucy` landed after this audit:

- `dev/update_zellij_pane_orchestrator.nu` is deleted and its sync behavior now
  lives directly in `maintainer/plugin_build.nu`
- `dev/update_zjstatus.nu` is deleted and its vendoring logic now lives
  directly in `maintainer/update_workflow.nu`
- demo helpers remain, but they are documented as manual maintainer helpers
  rather than normal runtime entrypoints

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
| Dev/test Nushell | about `14.4k` lines under `nushell/scripts` after the governed test deletions | validators, sweep/E2E runners, fixtures, demo helpers, package smoke checks | indirect |
| Maintainer Nushell | about `2.1k` lines under `nushell/scripts/maintainer` | issue sync, version bump, update workflow, readme sync, plugin build, test runner | indirect but release-critical |
| Sweep helpers | about `450` lines under `nushell/scripts/dev/sweep` plus `config_sweep_runner.nu` | matrix and visual sweep orchestration | release-confidence lane |

The top maintainer/test files by size after the governed Nu-test and validator
cuts are:

| File | Approx lines | Audit role |
| --- | ---: | --- |
| `maintainer/update_workflow.nu` | `768` | release/update workflow owner |
| `yzx/dev.nu` | `410` | public maintainer/dev command router |
| `dev/config_normalize_test_helpers.nu` | `356` | temporary helper debt to delete after stronger Rust coverage absorbs it |
| `config_sweep_runner.nu` | `325` | non-visual and visual sweep runner |

## 3. Must-Not-Lose Behavior

| Behavior | Current contract or source | Current owner | Current verification | Candidate surviving owner |
| --- | --- | --- | --- | --- |
| Default suite runs explicit high-signal Rust suite membership instead of globbing every `test_*.nu` | `docs/specs/test_suite_governance.md` | Rust `yzx_repo_maintainer run-tests` plus Rust suite inventory | `yzx dev test`; `yzx_repo_validator validate-default-test-traceability` | Rust runner |
| Sweep and visual lanes remain explicit and separate from the default suite | `docs/specs/test_suite_governance.md` | Nu `config_sweep_runner.nu` and `dev/sweep/*.nu` | sweep lane manual/targeted checks | same, but dispatch should stay fixed and direct |
| Version bump and release notes stay transactional and refuse dirty/invalid release states | `docs/specs/upgrade_notes_contract.md` | Rust `repo_version_bump.rs` via `yzx_repo_maintainer version-bump` | maintainer tests for bump and upgrade contracts | Rust owner plus thin `yzx dev` handoff |
| Update workflow refreshes runtime pins, runs canaries, and requires explicit activation mode for real updates | `docs/specs/runtime_distribution_capability_tiers.md`; maintainer tests | Nu `maintainer/update_workflow.nu` | maintainer update tests | same |
| Plugin build/sync keeps pane-orchestrator wasm rebuild requirements visible | AGENTS Rust plugin workflow | Nu `maintainer/plugin_build.nu` | maintainer tests and manual build command | `plugin_build.nu` |
| Contract/test/spec validators keep the ratchet cheap and deterministic | `docs/contract_driven_development.md` | Rust `yzx_repo_validator` | validator commands | Rust owner |
| Profiling harness records the real startup boundaries and can compare cold/warm/desktop/launch scenarios | `docs/specs/startup_profile_scenarios.md` | Nu `yzx/dev.nu` plus `startup_profile.nu` | maintainer profile tests | same |

## 4. Canonical Owner Map

| Concern | Current owner or split boundary | Split kind | Audit judgment |
| --- | --- | --- | --- |
| Public `yzx dev` command surface | Nu `yzx/dev.nu` plus Rust metadata | intentional | Keep only thin shell/public routing in Nu; move deterministic policy down |
| Test runner selection and logging | Rust `yzx_repo_maintainer run-tests` | completed owner cut | Keep Nu only as public `yzx dev test` handoff |
| Default suite test logic | Rust tests plus Nu shell runner | intentional | Governed tests live in Rust; Nu should not own test logic again |
| Contract/spec/test validators | Nu validator scripts | intentional | Good Nu fit unless a Rust port deletes a whole validator |
| Issue/GitHub sync | Rust `repo_issue_sync.rs` via `yzx_repo_maintainer sync-issues` plus thin shell handoff | mixed | Keep Rust as the deterministic owner; Nu may retain only public argv or fixed tool execution |
| Version bump/update workflow | Rust `repo_version_bump.rs` for bump policy plus Nu `maintainer/update_workflow.nu` for the retained update flow | mixed | Keep version policy in Rust and keep only real Nix/git orchestration in Nu |
| Plugin wasm build/sync | Nu `plugin_build.nu` plus legacy wrapper | mixed | Keep canonical module; delete/demote thin wrapper |
| Demo recording/font helpers | Nu dev scripts | manual | Should not be counted as product runtime contract |

## 5. Delete-First Findings

### Delete Now

- The thin update/plugin wrapper files named by this audit were later deleted
  under `yazelix-4ucy`.
- Demo recording helpers remain by design, but only as manual maintainer tools.

### Bridge Layer To Collapse

- `maintainer/test_runner.nu` was deleted under `yazelix-lj7z.3`; the Rust
  maintainer runner now owns default-vs-sweep selection, profiling, and log
  shaping
- `yzx/dev.nu` still centralizes a broad set of maintainer and profiling entry
  surfaces that should keep shrinking toward thin shell routing
- the temporary shell-heavy runner scripts should not become a permanent second
  test layer just because the governed Nu suite is gone

### Full-Owner Migration

- No broad Rust maintainer rewrite is justified now.
- A Rust port would be valid only for a deterministic validator or parser that
  deletes a full Nu validator owner. It should not target `gh`, `bd`, `git`,
  Nix, or terminal-window orchestration just because the files are large.

### Likely Survivors

- `update_workflow.nu`, `plugin_build.nu`, `yzx/dev.nu`, and sweep helpers
- Rust validators and maintainer runners that scan source files and docs cheaply
- startup/profile harnesses that execute the real startup paths
- sweep helpers that launch matrix cases

### No-Go Deletions

- Deleting update/release automation
  - stop condition: these scripts are release-critical and have high-signal
    maintainer regressions
- Deleting issue sync policy from Rust
  - stop condition: there is no honest reason to move deterministic GitHub/Beads reconciliation back into Nu
- Moving sweep visual execution to Rust
  - stop condition: the hard part is external terminal/process orchestration

## 6. Quality Findings

- duplicate owners:
  - resolved under `yazelix-4ucy`: pane-orchestrator wasm sync now lives only
    in `maintainer/plugin_build.nu`
  - resolved under `yazelix-4ucy`: vendored `zjstatus.wasm` refresh now lives
    only in `maintainer/update_workflow.nu`
  - resolved under `yazelix-rdn7.4.5.4`: governed Nu test ownership no longer
    duplicates Rust-owned deterministic command/materialization/workspace tests
  - resolved under `yazelix-lj7z.2`: package validators now share Rust
    helper logic inside `yzx_repo_validator`
- missing layer problems:
  - no single maintainer-harness contract says which scripts are shipped
    runtime commands versus manual-only repo helpers
  - no cheap dry-run check currently proves sweep dispatch without opening
    visual windows
- extra layer problems:
  - resolved under `yazelix-4ucy`: demo recording scripts are now documented as
    manual maintainer helpers instead of normal runtime script surface
- DRY opportunities:
  - keep direct runner invocation for sweep lanes
  - fold any remaining thin `yzx dev` forwarding policy into canonical
    maintainer modules
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
| `yazelix-fg51` | default, lint-only, sweep, visual, all, profile, and new-window `yzx dev test` behavior | `bridge_collapse` | one explicit Nu test runner with fixed inventory dispatch | default suite; targeted sweep dispatch; syntax validation | keep external Nu execution if isolation is needed, but avoid broad routing policy drift |
| `yazelix-4ucy` | wasm build/sync, zjstatus vendoring, demo helpers if kept, nixpkgs smoke validation | landed | canonical maintainer modules plus manual docs | maintainer tests and specific tool checks | keep the canonical maintainer entrypoints documented |

## Verification

- manual review of:
  - `nushell/scripts/yzx/dev.nu`
  - `nushell/scripts/maintainer/*.nu`
  - `nushell/scripts/dev/*.nu`
  - `nushell/scripts/dev/sweep/*.nu`
- `yzx_repo_validator validate-specs`

## Traceability

- Bead: `yazelix-rdn7.5.6`
- Defended by: `yzx_repo_validator validate-specs`
- Informed by: `docs/specs/test_suite_governance.md`
- Informed by: `docs/specs/startup_profile_scenarios.md`
- Informed by: `docs/specs/upgrade_notes_contract.md`
