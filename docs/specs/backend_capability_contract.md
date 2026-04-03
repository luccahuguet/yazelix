# Backend Capability Contract

## Summary

Yazelix should define its backend contract in terms of concrete capabilities, not in terms of a specific tool such as `devenv`. A backend is the runtime/environment layer that makes the Yazelix workspace and control-plane entrypoints possible. The contract should be precise enough to evaluate current full Yazelix, a future Pixi experiment, and a future Nix-only `Yazelix Core` candidate without moving the goalposts.

## Why

Yazelix currently talks about “the backend” as if it only means “which tool opens the shell.” The code already shows a broader contract:

- install and update own a stable runtime identity
- `yzx env`, `yzx run`, `yzx refresh`, and `yzx launch` rely on backend-controlled activation and rebuild behavior
- fast launch relies on cached profile reuse and invalidation rules
- runtime-owned binaries and assets are resolved through the active runtime, not only through generic `PATH` lookup

Without a written contract:

- later backend experiments will keep redefining the criteria
- launch preflight checks and backend capability checks will blur together
- `Yazelix Core` planning will stay fuzzy
- refactors will keep mixing backend concerns with workspace UX concerns

## Scope

- define the minimum backend capabilities Yazelix depends on today
- classify backend capabilities as required, optional convenience, or out of scope
- define which current runtime/install/profile semantics belong to the backend
- define what later backend evaluations must compare against
- distinguish backend capability requirements from launch-time preflight/dependency checks

## Behavior

- A Yazelix backend is the layer that materializes and activates the runtime/tool environment needed by Yazelix entrypoints.
- The backend contract includes these required capability buckets:
  - Runtime materialization
    - provide the shipped/default Yazelix runtime tool stack in a reproducible way
    - include the runtime-owned tools that Yazelix invokes directly outside any nested shell session when those tools are part of the product contract
  - Stable runtime identity
    - provide a stable notion of “the active Yazelix runtime” for install, update, restart, and generated-state ownership
    - allow stable user-facing entrypoints such as installed `yzx` to resolve the current runtime without guessing
  - Activation entrypoints
    - support the behaviors behind `yzx env`, `yzx run`, launch re-entry, and noninteractive runtime command execution
    - allow Yazelix to enter or invoke the managed environment from both fresh shells and already-running user sessions
  - Refresh and rebuild semantics
    - support explicit rebuild/refresh behavior when rebuild-relevant inputs change
    - give Yazelix a coherent way to request a fresh environment after input drift
  - Reusable activation semantics
    - support either cached profiles, reusable environments, or another explicit activation model that Yazelix can reason about
    - make invalidation rules concrete enough that Yazelix can decide whether a cached launch path is still valid
  - Runtime-owned binary and asset resolution
    - allow Yazelix to resolve runtime-owned tools and assets through the active runtime contract rather than assuming host-global installations
    - examples in current code include `devenv`, `nu`, launch scripts, and bundled runtime assets
  - Generated state relationship
    - define how backend materialization relates to generated configs, cached launch state, and runtime project state under `~/.local/share/yazelix`
    - generated state is not the backend itself, but its validity depends on backend identity and input drift
- The backend contract includes these optional conveniences:
  - task/process/service orchestration beyond what Yazelix currently requires for normal entrypoints
  - broader cross-platform package parity than the current product actually promises
  - convenience wrappers that improve ergonomics without changing the core contract
- The backend contract explicitly does not own:
  - fast user-facing dependency/preflight diagnostics such as “is Helix/Zellij/the configured terminal available right now?”
    - that belongs to the runtime dependency / launch preflight contract
  - the final product/support boundary for `Yazelix Core`
  - workspace UX semantics such as pane ownership, sidebar synchronization, or tab naming
  - the choice of one new backend implementation

### Current Code Evidence

- `nushell/scripts/utils/devenv_cli.nu` shows that Yazelix already expects preferred runtime-owned backend CLI resolution, not just host `PATH` lookup.
- `nushell/scripts/utils/environment_bootstrap.nu` shows that rebuild, refresh, and re-entry behavior are backend concerns.
- `nushell/scripts/utils/launch_state.nu` and `nushell/scripts/utils/config_state.nu` show that reusable launch/profile semantics are part of the current backend contract.
- `nushell/scripts/yzx/env.nu`, `nushell/scripts/yzx/run.nu`, `nushell/scripts/yzx/launch.nu`, and `nushell/scripts/yzx/refresh.nu` show that the backend currently owns activation and rebuild control-plane behavior.
- `docs/specs/flake_interface_contract.md`, `docs/specs/one_command_install_ux.md`, and `shells/posix/install_yazelix.sh.in` show that install/update identity is part of the backend story too.

### Evaluation Matrix Requirement

Any later backend evaluation should assess at least these columns against this contract:

- full Yazelix today
- possible Nix-only `Yazelix Core` backend
- possible Pixi-backed experiment

The point of this matrix is not to force parity. It is to keep the capability comparison stable.

## Non-goals

- choosing Pixi, Nix-only Core, or any other alternative backend now
- defining launch-time dependency checker behavior
- defining the final `Yazelix Core` product boundary
- rewriting the implementation to match the contract yet
- promising that every future backend candidate must preserve every current full-Yazelix convenience

## Acceptance Cases

1. When a later bead asks “can this candidate backend support Yazelix?”, the answer can be evaluated against explicit capability buckets instead of ad hoc intuition.
2. When a later bead asks whether cached launch/reuse behavior is part of the backend contract, the answer is clearly yes, with invalidation semantics treated as part of the backend surface.
3. When a later bead asks whether “is Ghostty installed?” belongs in the backend contract, the answer is clearly no, because that is part of launch-time dependency/preflight behavior.
4. When a future `Yazelix Core` discussion happens, it can reuse this contract without collapsing product-boundary questions back into backend-capability questions.
5. When later refactors separate backend concerns from workspace UX, they can point at concrete contract buckets rather than reverse-engineering requirements from `devenv`-specific code paths.

## Verification

- manual review against:
  - [architecture_map.md](../architecture_map.md)
  - [runtime_root_contract.md](./runtime_root_contract.md)
  - [config_surface_and_launch_profile_contract.md](./config_surface_and_launch_profile_contract.md)
  - [flake_interface_contract.md](./flake_interface_contract.md)
  - [one_command_install_ux.md](./one_command_install_ux.md)
- manual review of the backend-coupled code paths:
  - `nushell/scripts/utils/devenv_cli.nu`
  - `nushell/scripts/utils/environment_bootstrap.nu`
  - `nushell/scripts/utils/launch_state.nu`
  - `nushell/scripts/utils/config_state.nu`
  - `nushell/scripts/yzx/env.nu`
  - `nushell/scripts/yzx/run.nu`
  - `nushell/scripts/yzx/launch.nu`
  - `nushell/scripts/yzx/refresh.nu`
- CI/spec check: `nu nushell/scripts/dev/validate_specs.nu`

## Traceability

- Bead: `yazelix-6lkw`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`

## Open Questions

- Should the current reusable launch-profile model remain a hard backend requirement, or is it better described as one acceptable implementation strategy for the “reusable activation semantics” bucket?
- Which current `devenv`-specific conveniences should later be downgraded from “required” to “optional” once the backend boundary is cleaner?
- For a future Nix-only `Yazelix Core` candidate, which parts of the default tool stack should be considered required runtime materialization versus explicit host-managed dependencies?
