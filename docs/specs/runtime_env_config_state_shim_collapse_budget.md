# Runtime Env And Config-State Shim Collapse Budget

## Summary

This document defines the delete-first budget for the remaining Nushell request
shims around runtime-env and config-state ownership:

- `nushell/scripts/utils/runtime_env.nu`
- `nushell/scripts/utils/config_state.nu`

The key question for `yazelix-ekfc.1` is whether they should move in one
implementation lane or split. The answer is split.

- `config_state.nu` no longer carries a real shell boundary. It is now mostly an
  argv/request shim over the Rust owner and is the clean first deletion lane
- `runtime_env.nu` still carries an irreducible shell/process boundary through
  `run_runtime_argv` and live callers that stage env before spawning processes

Keeping those two seams in one implementation task would blur the stop
condition. One side should fully collapse. The other side should shrink to an
explicit shell-exec seam, not pretend to be equally deletable right now.

## Scope

- current live callers of `get_runtime_env`, `run_runtime_argv`,
  `compute_config_state`, and `record_materialized_state`
- Rust owners in `runtime_env.rs`, `config_state.rs`, `control_plane.rs`,
  `yzx_control.rs`, and related helper routes
- verification that must survive the deletion cuts

Out of scope:

- the broader launch/startup audit under `yazelix-rdn7.5.3`
- replacing shell/process execution with Rust
- reviving launch-profile or ambient-host inference semantics

## Retained Contracts

The owner cut must preserve all of these:

| Behavior | Current contract or source | Current owner | Current verification | Candidate surviving owner |
| --- | --- | --- | --- | --- |
| Canonical runtime-env policy stays Rust-owned and does not drift back into Nu | `CRCP-002`; `docs/specs/launch_bootstrap_rust_migration.md` | Rust `runtime_env.rs`; Nu `runtime_env.nu` request bridge | `rust_core/yazelix_core/tests/yzx_core_runtime_env.rs`; `nu nushell/scripts/dev/test_helix_managed_config_contracts.nu`; `nu nushell/scripts/dev/test_yzx_popup_commands.nu`; `nu nushell/scripts/dev/test_yzx_core_commands.nu` | Rust `runtime_env.rs` / `control_plane.rs` plus a smaller Nu shell-exec seam only |
| Generated-state freshness stays Rust-owned and uses the canonical managed main-config path for record decisions | `docs/specs/rust_nushell_bridge_contract.md`; `docs/specs/config_runtime_control_plane_canonicalization_audit.md` | Rust `config_state.rs`; Nu `config_state.nu` argv shim | `rust_core/yazelix_core/src/config_state.rs`; `nu nushell/scripts/dev/validate_config_surface_contract.nu`; `nu nushell/scripts/dev/test_yzx_generated_configs.nu`; `nu nushell/scripts/dev/test_yzx_core_commands.nu` | Rust `config_state.rs` and `control_plane.rs` only |
| No ambient host inference is reintroduced while building runtime-env inputs or config-state inputs | `CRCP-002`; `docs/specs/cross_language_runtime_ownership.md` | mixed Nu/Rust today | same tests as above | Rust explicit request construction |
| Popup/editor/startup callers can still stage the canonical runtime env before they spawn a process | `docs/specs/launch_bootstrap_rust_migration.md` | Nu callers plus `runtime_env.nu::run_runtime_argv` | `nu nushell/scripts/dev/test_yzx_popup_commands.nu`; `nu nushell/scripts/dev/test_yzx_workspace_commands.nu`; `nu nushell/scripts/dev/test_helix_managed_config_contracts.nu` | Nu shell-exec boundary only |

## Current Caller Inventory

## `runtime_env.nu`

| Caller | What it needs today | Real shell/process boundary | Budget judgment |
| --- | --- | --- | --- |
| `core/start_yazelix.nu` | env record before `with-env` startup/setup and inner-script exec | yes | keep caller shell orchestration, move request construction out of Nu |
| `yzx/launch.nu` | env record before `with-env` launch handoff | yes | keep caller shell orchestration, move request construction out of Nu |
| `zellij_wrappers/yzx_popup_program.nu` | env record and argv execution for popup program launch | yes | keep `run_runtime_argv`-style seam, delete request construction |
| `integrations/zellij_runtime_wrappers.nu` | env record merged into current shell env | yes, but only for env application | keep caller-local env application, delete request construction |
| `utils/editor_launch_context.nu` | canonical editor env before spawning editor | yes | keep caller-local env use, delete request construction |
| tests in `test_helix_managed_config_contracts.nu` and `test_yzx_popup_commands.nu` | direct helper coverage | n/a | must survive under the narrowed seam |

## `config_state.nu`

| Caller | What it needs today | Real shell/process boundary | Budget judgment |
| --- | --- | --- | --- |
| `utils/environment_bootstrap.nu` | structured config state with `needs_refresh` and normalized config | no | switch to a Rust-owned from-env or control-plane request path |
| `core/launch_yazelix.nu` | structured config state during launch profiling/materialization decisions | no | switch to a Rust-owned from-env or control-plane request path |
| `validate_config_surface_contract.nu` | compute and record state for invariant checks | no | switch validator to Rust-owned path |
| `test_yzx_core_commands.nu` | direct compute/record helper coverage | no | update tests to the surviving owner |
| `test_yzx_generated_configs.nu` | direct record helper coverage for symlinked managed config | no | update tests to the surviving owner |

## Owner Decision

Implementation should split:

## First implementation lane

- collapse `config_state.nu` completely
- move request construction for `config-state.compute` and
  `config-state.record` into Rust
- let Nu callers consume either:
  - a `--from-env` helper path, or
  - one narrower Rust control-plane surface

Why first:

- it deletes a whole Nushell owner file
- it does not depend on keeping a live shell-exec seam
- it preserves the current generated-state contract with the least ambiguity

## Second implementation lane

- collapse only the request-building half of `runtime_env.nu`
- keep one explicit Nu shell-exec seam for process spawning and `with-env`
  application
- delete or move the file only if the surviving shell-exec helper can stand on
  its own without smuggling request ownership back in

Why second:

- `runtime_env.compute` already took the typed decision layer
- the remaining Nu code is partly real shell/process orchestration, not just a
  fake bridge
- the runtime-env stop condition is different from config-state and should stay
  explicit

## Deletion Budget

## `config_state.nu`

Functions that should disappear from Nu ownership:

- `compute_config_state`
- `record_materialized_state`

The surviving owner should construct these Rust inputs itself:

- runtime dir
- active managed config/default/contract paths
- state path
- canonical managed main-config path
- config/runtime hashes supplied back into the record call

Candidate implementation shapes:

- add `--from-env` to `config-state.compute`
- add `--from-env` to `config-state.record`
- or add one narrower Rust-owned control-plane entrypoint that resolves these
  inputs explicitly

Result goal:

- delete `nushell/scripts/utils/config_state.nu`

## `runtime_env.nu`

Functions that should stop being owner logic:

- `build_runtime_env_request`
- most of `get_runtime_env` once Rust can resolve explicit inputs directly

Function that may survive:

- `run_runtime_argv`

Candidate implementation shapes:

- add `--from-env` to `runtime-env.compute`
- or add a Rust control-plane helper that loads normalized config and builds the
  request without Nu reconstructing it

Result goal:

- `runtime_env.nu` becomes an exec-only seam, or is deleted after the exec seam
  is moved to a narrower file such as `runtime_exec.nu`

## Verification Gate

Config-state cut must still pass:

- `rust_core/yazelix_core/src/config_state.rs`
- `nu nushell/scripts/dev/validate_config_surface_contract.nu`
- `nu -c 'source nushell/scripts/dev/test_yzx_generated_configs.nu; [(test_record_materialized_state_accepts_symlinked_managed_main_config)]'`
- `nu -c 'source nushell/scripts/dev/test_yzx_core_commands.nu; let st = (compute_config_state); record_materialized_state $st'` equivalent surviving-path coverage

Runtime-env cut must still pass:

- `rust_core/yazelix_core/tests/yzx_core_runtime_env.rs`
- `nu nushell/scripts/dev/test_helix_managed_config_contracts.nu`
- `nu nushell/scripts/dev/test_yzx_popup_commands.nu`
- `nu nushell/scripts/dev/test_yzx_workspace_commands.nu`
- `nu nushell/scripts/dev/test_yzx_core_commands.nu`

Shared validation:

- `nu nushell/scripts/dev/validate_syntax.nu`
- `nu nushell/scripts/dev/validate_specs.nu`

## Stop Conditions

## Config-state stop condition

Stop if the surviving Rust path cannot preserve the canonical managed-surface
recording gate without reintroducing a Nu-side path/argv owner.

The fallback is not to keep `config_state.nu` indefinitely. The fallback is to
record the smallest remaining caller-local adapter explicitly.

## Runtime-env stop condition

Stop if the only way to delete `runtime_env.nu` is to hide real shell/process
orchestration inside Rust or to reintroduce ambient host inference.

The acceptable fallback is:

- keep one explicit Nu exec/application seam
- delete only the request-building part

## Bead Decision

This planning bead chooses a split implementation lane.

Bead changes required:

- revise `yazelix-ekfc.2` to own the config-state owner cut
- create a new sibling bead for the runtime-env request-construction cut and
  explicit exec-seam outcome

## Acceptance

1. The retained contracts for runtime-env and generated-state are named
2. Every live caller is classified by whether it still needs a shell/process
   boundary
3. The implementation order is explicit and justified
4. The verification gate and stop conditions are explicit before code deletion
5. The bead structure matches the split decision instead of leaving a stale
   combined implementation task

## Traceability

- Bead: `yazelix-ekfc.1`
- Informed by: `docs/specs/config_runtime_control_plane_canonicalization_audit.md`
- Informed by: `docs/specs/launch_bootstrap_rust_migration.md`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
- Defended by: manual review of the cited callers and Rust owners
