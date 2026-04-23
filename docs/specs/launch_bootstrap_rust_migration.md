# Launch Bootstrap Rust Migration

> Status: Historical transition note
> `runtime-env.compute` already landed, and the remaining v15.x launch and
> bootstrap Rust follow-up was explicitly stopped on `2026-04-19`
> Keep this file as migration history only
> Names such as `core/yazelix.nu`, `yzx/env.nu`, and `yzx/run.nu` that appear
> below are historical references from the transition window, not current live
> owners
> Current delete-first planning should start from
> [rust_migration_matrix.md](./rust_migration_matrix.md)

## Summary

Current v15 Yazelix launch and bootstrap work should move toward Rust only through narrow typed helper slices behind Nushell, not through a broad CLI or terminal-orchestration rewrite.

For `yazelix-kt5.4`, the first live migration target is canonical runtime-environment planning. That decision layer currently lives in `runtime_env.nu` and is shared by `yzx env`, `yzx run`, popup/helper launches, and the startup path. Rust should take over that deterministic planning work first while Nushell and POSIX shell keep command UX, process spawning, terminal retries, startup profiles, and the final Zellij handoff.

## Why

The older `devenv_backend`, `launch_state`, launch-profile reuse, and public `yzx refresh` story is no longer the live product. Planning launch/bootstrap Rust work against those historical seams would freeze deleted behavior back into the codebase.

The current branch already has a better seam:

- POSIX shell owns stable runtime-root bootstrap and base process environment
- Nushell owns public `yzx` UX, startup profiling, and host/process orchestration
- `yzx_core` already owns typed config-state, runtime-contract, and materialization decisions

The remaining launch/bootstrap logic is only worth moving to Rust where it is deterministic, shared, and testable. Canonical runtime-environment planning meets that bar. Terminal command construction and detached launch execution do not.

## Scope

- define the current live launch/bootstrap ownership map for v15
- define which current modules stay authoritative, become shims, or are explicit non-goals
- pick the first Rust-owned decision layer under `yazelix-kt5.4`
- define parity and regression coverage for `yzx env`, `yzx run`, popup/helper launches, generated-state refresh, and launch/startup paths
- define the v15.x versus v16 boundary for this subsystem

## Behavior

### Current Live Owners

| Area | Current authoritative owner | v15.x decision |
| --- | --- | --- |
| Stable launcher entry and runtime-root bootstrap | `shells/posix/yzx_cli.sh`, `shells/posix/runtime_env.sh` | Keep POSIX-owned |
| Public command surface | `nushell/scripts/core/yazelix.nu`, `nushell/scripts/yzx/*.nu` | Keep Nushell-owned |
| Config parsing and state freshness | `environment_bootstrap.nu`, `config_state.nu`, `yzx_core config.normalize`, `config-state.compute` | Keep current mixed ownership |
| Runtime dependency and launch preflight classification | direct preflight calls in `start_yazelix.nu`, `launch_yazelix.nu`, and `yzx/launch.nu` over `yzx_core startup-launch-preflight.evaluate` and `runtime-contract.evaluate` | Keep current mixed ownership |
| Generated runtime materialization and repair | Rust: `runtime-materialization.plan`, `runtime-materialization.materialize`, `runtime-materialization.repair`, including `--from-env` request construction; caller-local Nu progress and error rendering | Keep Rust-owned |
| Canonical per-session runtime environment | `nushell/scripts/utils/runtime_env.nu` | First `kt5.4` Rust migration slice |
| Terminal launch string construction and detached execution | `launch_yazelix.nu`, `terminal_launcher.nu`, `shells/posix/start_yazelix.sh` | Keep Nushell/POSIX-owned in v15.x |
| In-terminal startup and Zellij handoff | `start_yazelix.nu`, `start_yazelix_inner.nu` | Keep Nushell-owned in v15.x |

Historical files and contracts must stay out of scope for this work:

- `nushell/scripts/utils/devenv_backend.nu`
- `nushell/scripts/utils/launch_state.nu`
- launch-profile reuse and `launch_state.json`
- public `yzx refresh`

### First Rust Slice

The first `kt5.4` implementation slice should add a private `yzx_core` helper command:

- `runtime-env.compute`

Recommended code location:

- `rust_core/yazelix_core/src/runtime_env.rs`

Recommended ownership boundary:

- Rust computes the canonical runtime environment as structured data from explicit inputs
- Nushell applies that environment with `with-env` / `load-env` and keeps process execution
- POSIX shell keeps the outer bootstrap environment from `runtime_env.sh`

The request should be explicit and machine-readable. It should include the minimum data needed to compute the current canonical runtime env without ambient inference, including:

- runtime root
- home and XDG-derived roots needed for Yazelix-managed config paths
- current `PATH` entries after wrapper bootstrap
- normalized config fields that affect the runtime env such as editor command, Helix runtime override, and sidebar/layout choice

The response should return the computed env record and any stable metadata needed by wrappers, such as editor kind or the filtered path-entry list, without spawning processes or printing prose.

### Module Fate Under This Plan

`runtime_env.nu` is the first module that should shrink materially under `kt5.4`.

The following decision helpers should move out of Nushell and become Rust-owned:

- `normalize_path_entries`
- `runtime_owned_path_entries`
- `strip_runtime_owned_path_entries`
- `is_helix_editor_command`
- `is_neovim_editor_command`
- `resolve_editor_command`
- `resolve_helix_runtime`

After the first slice lands, `runtime_env.nu` should remain only as a thin bridge and execution helper:

- `get_runtime_env` becomes a bridge wrapper around `runtime-env.compute`
- `activate_runtime_env` stays as `load-env`
- `run_runtime_argv` stays as structured argv execution inside the computed env

The following modules stay authoritative in v15.x and must not be ported as part of this slice:

- `shells/posix/runtime_env.sh`
- `nushell/scripts/utils/environment_bootstrap.nu`
- `nushell/scripts/core/launch_yazelix.nu`
- `nushell/scripts/utils/terminal_launcher.nu`
- `nushell/scripts/core/start_yazelix.nu`
- `nushell/scripts/core/start_yazelix_inner.nu`

### Later `kt5.4` Work

Only after the runtime-env slice has landed and deleted real Nushell decision logic should later work consider another launch/bootstrap Rust slice.

The next possible candidate is structured launch-request planning, not terminal execution. That means data-only decisions such as:

- normalized working-directory classification
- terminal candidate selection metadata
- request-shape validation shared by launch/start wrappers

That later slice is optional in v15.x. It should happen only if it deletes a real Nushell decision seam without moving shell-string assembly or detached terminal execution into Rust.

### Parity And Regression Strategy

`kt5.4` must preserve the current v15 behavior surface while changing inner ownership.

`yzx env` and `yzx run` parity:

- keep wrapper-side argv behavior unchanged
- verify canonical env output through `nushell/scripts/dev/test_yzx_core_commands.nu`
- add Rust helper integration tests for `runtime-env.compute`

Popup/helper and nested-editor parity:

- keep `VISUAL == EDITOR` in the canonical runtime env
- keep popup/helper launches on the same shared env contract
- verify via `nushell/scripts/dev/test_yzx_popup_commands.nu`
- verify Helix wrapper and curated PATH behavior via `nushell/scripts/dev/test_helix_managed_config_contracts.nu`

Generated-state refresh parity:

- do not reintroduce public refresh semantics
- keep `needs_refresh`, materialization planning, and repair behavior owned by the existing config-state and materialization path
- verify with `nushell/scripts/dev/test_yzx_generated_configs.nu` and the current doctor/generated-state checks that defend the live v15 contract

Launch/startup parity:

- keep `launch_yazelix.nu`, `start_yazelix.nu`, and `start_yazelix_inner.nu` as the startup-profile and process-orchestration owners
- keep current terminal fallback, verbose output, and detached-launch behavior
- verify through `nushell/scripts/dev/test_yzx_workspace_commands.nu`

### v15.x Versus v16 Boundary

Safe v15.x scope:

- private `yzx_core` helper commands behind Nushell
- deletion of deterministic decision logic from `runtime_env.nu`
- optional later data-only launch planning if it clearly shrinks surviving Nushell logic

Likely v16-or-later scope:

- any broad rewrite of `yzx launch`, `start_yazelix`, or terminal-launch orchestration
- moving detached terminal execution or shell-boundary logic into Rust
- replacing Nushell as the public `yzx` owner

This means `yazelix-kt5.4` is worth doing now only as a narrow helper-backed insertion plan. A full launch/bootstrap rewrite should remain deferred unless the smaller slices prove enough value to justify it.

### 2026-04-19 Audit Outcome

Follow-up audit `yazelix-fjty` reviewed the live launch/startup owners after `runtime-env.compute` landed:

- `nushell/scripts/core/launch_yazelix.nu`
- `nushell/scripts/utils/terminal_launcher.nu`
- `nushell/scripts/core/start_yazelix.nu`
- `nushell/scripts/core/start_yazelix_inner.nu`
- `nushell/scripts/yzx/launch.nu`
- `nushell/scripts/yzx/env.nu`
- `nushell/scripts/yzx/run.nu`
- `shells/posix/start_yazelix.sh`
- `shells/posix/runtime_env.sh`

Conclusion for v15.x:

- working-directory and runtime-script validation is already owned by `runtime-contract.evaluate`
- canonical runtime env planning is already owned by `runtime-env.compute`
- the surviving launch/startup code is now mostly terminal config generation, host command detection, shell quoting, detached terminal execution, startup-profile wiring, generated-state refresh, and Zellij handoff

Those remaining owners are too shell-bound and orchestration-heavy to justify another v15.x Rust helper. The default decision after this audit is to stop the launch/bootstrap Rust rewrite here for v15.x.

Reopen this track only if a later bug or contract change exposes one more shared deterministic decision layer that is both:

- clearly smaller than the surrounding execution path
- able to delete real Nushell logic instead of adding another bridge around shell-string assembly or host process control

## Non-goals

- reintroducing `devenv_backend`, `launch_state`, launch-profile reuse, or public `yzx refresh`
- starting a clap/public-CLI rewrite
- moving terminal command construction or detached launch execution into Rust as part of the first slice
- moving generated text renderers into Rust
- changing startup profile schema or step ownership

## Acceptance Cases

1. A maintainer can map the live v15 launch/bootstrap path without relying on deleted `devenv` or launch-profile seams
2. The first `kt5.4` Rust slice is explicit: `runtime-env.compute` owns canonical runtime-environment planning and `runtime_env.nu` becomes a bridge
3. The plan states exactly which current modules stay Nushell/POSIX-owned in v15.x and which local helpers are expected to disappear
4. The parity strategy explicitly covers `yzx env`, `yzx run`, popup/helper launches, generated-state refresh/repair, and launch/startup paths
5. The plan states why a broad launch/bootstrap rewrite is deferred to v16-or-later unless narrower slices prove themselves first

## Verification

- `yzx_repo_validator validate-specs`
- manual review against `docs/specs/v15_trimmed_runtime_contract.md`
- manual review against `docs/specs/rust_migration_matrix.md`
- manual review against `docs/specs/rust_nushell_bridge_contract.md`
- manual review against `docs/specs/runtime_dependency_preflight_contract.md`
- future implementation verification:
  - `cargo test --manifest-path rust_core/yazelix_core/Cargo.toml`
  - `nu nushell/scripts/dev/test_yzx_core_commands.nu`
  - `nu nushell/scripts/dev/test_yzx_popup_commands.nu`
  - `nu nushell/scripts/dev/test_helix_managed_config_contracts.nu`
  - `nu nushell/scripts/dev/test_yzx_generated_configs.nu`
  - `nu nushell/scripts/dev/test_yzx_workspace_commands.nu`

## Traceability

- Bead: `yazelix-kt5.4`
- Defended by: `yzx_repo_validator validate-specs`

## Open Questions

- Should `runtime-env.compute` accept one JSON request payload like `runtime-contract.evaluate`, or should it stay on explicit flags because the input record is still small
- Resolved 2026-04-19: no later launch-request planning slice is justified for v15.x with the current code shape; revisit only if a new deterministic shared seam appears
- Should the first execution bead for this plan absorb the remaining work from `yazelix-0dra`, or should the bug and the Rust migration stay separate tracks that share tests
