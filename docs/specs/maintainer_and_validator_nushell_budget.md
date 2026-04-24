# Maintainer And Validator Nushell Budget

## Summary

The maintainer Nushell floor is now `425` LOC in one file:
`nushell/scripts/yzx/dev.nu`.

This is no longer the old mixed maintainer/update/sweep surface. The Rust-owned
maintainer binary now owns:

- update workflow policy and vendored asset refresh
- pane-orchestrator build and sync
- sweep planning
- `nu-lint` execution
- deterministic validators, issue sync, and version bump policy

## Scope

In scope:

- `nushell/scripts/yzx/dev.nu`
- the Rust-owned `yzx_repo_maintainer` command surface it routes to

Out of scope:

- startup/profile product surfaces outside `yzx dev`
- runtime/front-door Nushell owners

## Current Measured Surface

Measured on `2026-04-24`:

| Surface | Current LOC | Hard target LOC | Status |
| --- | ---: | ---: | --- |
| Maintainer and `yzx dev` Nu floor | `425` | `425` | allowlisted floor |
| Deterministic validator Nu | `0` | `0` | fully Rust-owned |

## Behavior

Retained Nu ownership is now narrow:

- public `yzx dev` routing
- the startup-profile shell harness that still launches the real `yzx enter`,
  `yzx desktop launch`, and `yzx launch` paths

Everything else in the maintainer surface is out of Nushell now.

## Non-goals

- moving the startup-profile shell harness to Rust unless that deletes the real
  shell owner end-to-end
- recreating maintainer policy in helper `.nu` files after it moved to Rust

## Acceptance Cases

1. `yzx dev update` routes to the Rust maintainer owner without any retained
   `update_workflow.nu` helper
2. `yzx dev lint_nu` routes to the Rust maintainer owner without a retained
   Nu `nu-lint` wrapper
3. The maintainer Nu floor is only `yzx/dev.nu`

## Verification

- `yzx_repo_validator validate-specs`
- `yzx_repo_validator validate-nushell-budget`
- `yzx_repo_validator validate-nushell-syntax`

## Traceability

- Bead: `yazelix-pw9j.4`
- Bead: `yazelix-pw9j.4.1`
- Bead: `yazelix-pw9j.4.2`
- Bead: `yazelix-pw9j.4.3`
- Defended by: `yzx_repo_validator validate-specs`
- Defended by: `yzx_repo_validator validate-nushell-budget`
- Defended by: `yzx_repo_validator validate-nushell-syntax`

## Open Questions

- If the startup-profile harness ever gains a fully packaged Rust owner that
  still launches the real shell paths, ratchet the floor again from the new
  measured tree
