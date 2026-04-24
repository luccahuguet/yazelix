# Maintainer And Validator Nushell Budget

## Summary

This document is the live delete-first budget for maintainer, `yzx dev`, and
validator Nu after `yazelix-lj7z.4`.

The deterministic validator surface is already fully Rust-owned. The new
maintainer cut also deleted the old Nu owners for issue sync, issue-bead
contract validation, repo-checkout helper selection, and version bump policy.

What remains is still large, but it is more honestly shell- and workflow-heavy:
`2,008` LOC across `8` tracked files.

## Scope

In scope:

- `nushell/scripts/maintainer/*.nu`
- `nushell/scripts/yzx/dev.nu`
- non-governed shell helpers under `nushell/scripts/dev/`
- the Rust-owned maintainer and validator binaries that replaced deleted Nu
  policy owners

Out of scope:

- product/runtime launch and front-door owners
- the remaining shell-heavy sweep runner budget

## Current Measured Surface

Measured on `2026-04-24`:

| Family | Current LOC | Hard target LOC | Main follow-up |
| --- | ---: | ---: | --- |
| Maintainer and `yzx dev` shell orchestration | `2,008` | `900` | keep shrinking `yzx/dev.nu`, update flow, plugin build, and helper debt |
| Deterministic validators and contract linters | `0` | `0` | already Rust-owned |

## Landed Rust Owners

Rust now owns these deterministic maintainer/validator paths:

- version bump validation, changelog rotation, and README sync support
- GitHub issue type inference and lifecycle reconciliation
- canonical Beads comment reconciliation
- deterministic validator and package-smoke checks through
  `yzx_repo_validator`

Deleted Nu owners:

- `maintainer/issue_bead_contract.nu`
- `maintainer/issue_sync.nu`
- `maintainer/repo_checkout.nu`
- `maintainer/version_bump.nu`

## Retained Nu Floor

These paths are still allowed only because they remain external-tool or
workflow heavy:

- `maintainer/update_workflow.nu`
- `maintainer/plugin_build.nu`
- `yzx/dev.nu`
- `dev/update_yazi_plugins.nu`
- `dev/materialization_dev_helpers.nu`
- `dev/config_normalize_test_helpers.nu`
- demo helpers
- sweep helpers under `dev/sweep/`
- `dev/yzx_test_helpers.nu`

## Floor Rules

1. Keep only real shell/process/tool orchestration in Nu
2. Do not move deterministic routing or policy back into `yzx/dev.nu`
3. Route first-party Rust tests through `cargo nextest run` by default
4. Keep version-bump and issue-sync policy in Rust
5. Delete helper files once the only things they support are deleted or
   Rust-owned

## Next Honest Cuts

1. shrink `yzx/dev.nu` from a broad maintainer router toward a thin public
   argv handoff
2. collapse `update_workflow.nu` once retained Nix/git policy can move without
   adding a second orchestration layer
3. re-evaluate `plugin_build.nu` after the pane-orchestrator sync path narrows
4. keep deleting helper files that now exist only to support shell-heavy sweep
   or legacy maintainer flows

## Verification

- `yzx_repo_validator validate-specs`
- `yzx_repo_validator validate-nushell-budget`
- `cargo test -p yazelix_core --manifest-path rust_core/Cargo.toml`

## Traceability

- Bead: `yazelix-lj7z.2`
- Bead: `yazelix-lj7z.4`
- Defended by: `yzx_repo_validator validate-specs`
- Defended by: `yzx_repo_validator validate-nushell-budget`
