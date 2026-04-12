# Backend Capability Contract

## Summary

Yazelix should define its backend contract in terms of concrete capabilities, not in terms of a specific tool such as `devenv`. A backend is the runtime/environment layer that makes the Yazelix workspace and control-plane entrypoints possible. The contract should be precise enough to evaluate current full Yazelix and a future Nix-only `Yazelix Core` candidate without moving the goalposts. Other later backend experiments should be able to reuse the same contract instead of shaping it.

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
- Delete-first rule for this contract:
  - keep only the capabilities Yazelix genuinely needs from the backend layer
  - do not keep a capability in the backend contract just because current `devenv`-based code happens to bundle it there
  - if a capability is really about launch diagnostics, product scope, or integration ownership, move it out of the backend contract
- The backend contract includes these hard-required capability buckets:
  - Config-input relationship
    - consume the canonical Yazelix config surfaces as backend inputs without owning their product semantics
    - current important inputs include:
      - `~/.config/yazelix/user_configs/yazelix.toml`
      - `~/.config/yazelix/user_configs/yazelix_packs.toml`
      - Home Manager when it renders the same effective intent into those canonical surfaces
    - use the rebuild-relevant subset of those inputs when deciding whether the backend needs refresh/rebuild work
    - do not redefine config meaning inside the backend contract; config ownership and schema parity still belong to the config-surface contract
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
  - Runtime-owned binary and asset resolution
    - allow Yazelix to resolve runtime-owned tools and assets through the active runtime contract rather than assuming host-global installations
    - examples in current code include `devenv`, `nu`, launch scripts, and bundled runtime assets
  - Generated state relationship
    - define how backend materialization relates to generated configs, cached launch state, and runtime project state under `~/.local/share/yazelix`
    - generated state is not the backend itself, but its validity depends on backend identity and input drift
- The backend contract includes these optional conveniences:
  - reusable activation acceleration
    - cached profiles, reusable environments, or other fast-path activation strategies are useful, but the exact optimization strategy is not the hard requirement
    - the hard requirement is that freshness and invalidation semantics are explicit enough for Yazelix to reason about them
  - task/process/service orchestration beyond what Yazelix currently requires for normal entrypoints
  - broader cross-platform package parity than the current product actually promises
  - convenience wrappers that improve ergonomics without changing the core contract
- The backend contract explicitly does not own:
  - fast user-facing dependency/preflight diagnostics such as “is Helix/Zellij/the configured terminal available right now?”
    - that belongs to the runtime dependency / launch preflight contract
  - the final product/support boundary for `Yazelix Core`
  - workspace UX semantics such as pane ownership, sidebar synchronization, or tab naming
  - the choice of one new backend implementation

### Delete-First Classification

The current code suggests this first-pass classification:

| Capability | Classification | Why |
| --- | --- | --- |
| Config-input relationship | Required | The backend has to consume config and pack inputs to decide materialization and refresh, even though it should not own config semantics. |
| Runtime materialization | Required | Yazelix cannot honestly launch or generate its managed runtime without a concrete tool/runtime surface. |
| Stable runtime identity | Required | Install/update, generated state, and stable entrypoints all depend on one clear active runtime. |
| `env` / `run` / launch re-entry entrypoints | Required | These are user-facing command surfaces today, not incidental implementation details. |
| Refresh / rebuild semantics | Required | Yazelix already has rebuild-relevant inputs and needs a coherent freshness model. |
| Runtime-owned binary and asset resolution | Required | Current behavior depends on runtime-owned `nu`, backend CLI resolution, launch scripts, and shipped assets. |
| Relationship to generated state | Required | Generated configs and launch-state validity already depend on runtime/backend identity. |
| Reusable activation acceleration | Optional convenience | Fast profile reuse matters, but the exact optimization mechanism should not be mistaken for the hard backend contract itself. |
| Services / long-lived process orchestration | Optional convenience | Useful in some backend ecosystems, but not part of the minimum Yazelix product contract today. |
| “Is the configured terminal/editor available right now?” checks | Out of scope | This is runtime dependency / launch preflight behavior, not backend capability definition. |
| Final `Yazelix Core` support boundary | Out of scope | This is a product decision, not a backend capability bucket. |
| Workspace UX semantics | Out of scope | These belong to workspace/session contracts, not the backend. |

### Current Code Evidence

- `nushell/scripts/utils/runtime_env.nu` shows that Yazelix already expects runtime-owned environment resolution, not just host `PATH` lookup.
- `nushell/scripts/utils/environment_bootstrap.nu` shows that rebuild, refresh, and re-entry behavior are backend concerns.
- `nushell/scripts/utils/launch_state.nu` and `nushell/scripts/utils/config_state.nu` show that reusable launch/profile semantics are part of the current backend contract.
- `docs/specs/config_surface_and_launch_profile_contract.md` shows that the backend consumes canonical config surfaces and rebuild-relevant subsets without owning config semantics outright.
- `nushell/scripts/yzx/env.nu`, `nushell/scripts/yzx/run.nu`, `nushell/scripts/yzx/launch.nu`, and `nushell/scripts/yzx/refresh.nu` show that the backend currently owns activation and rebuild control-plane behavior.
- `docs/specs/flake_interface_contract.md`, `docs/specs/one_command_install_ux.md`, and `shells/posix/install_yazelix.sh.in` show that install/update identity is part of the backend story too.

### Evaluation Matrix Requirement

Any later backend evaluation should assess at least these columns against this contract:

- full Yazelix today
- possible Nix-only `Yazelix Core` backend

Optional later candidates, such as a Pixi experiment, should reuse the same matrix instead of changing the contract itself.

The point of this matrix is not to force parity. It is to keep the capability comparison stable.

### First-Pass Comparison Matrix

This matrix is intentionally tentative. It exists to make the backend contract reusable, not to pre-approve a new backend.

| Capability bucket | Full Yazelix today | Nix-only `Yazelix Core` candidate |
| --- | --- | --- |
| Config-input relationship | Native fit through canonical managed config surfaces plus rebuild-relevant input hashing. | Native fit if Core keeps the same config surface contract and only narrows backend behavior, not config ownership. |
| Runtime materialization | Native fit via the current `devenv`-backed runtime model. | Plausible fit if Core is a fixed, opinionated flake/runtime bundle rather than a flexible pack-driven environment. |
| Stable runtime identity | Native fit through the installed runtime pointer, packaged runtime tree, and stable `yzx` entrypoint. | Native fit if Core keeps a stable runtime pointer or direct package identity and does not reintroduce clone-path assumptions. |
| `yzx env` / `yzx run` / launch re-entry entrypoints | Native fit through the current environment bootstrap and re-entry flow. | Possible with adaptation. Core would likely keep `launch` and selected noninteractive entrypoints, but may narrow or redefine `env` semantics. |
| Refresh / rebuild semantics | Native fit through current rebuild-relevant input hashing and explicit refresh/re-entry flows. | Possible with adaptation. Core still needs freshness semantics, but likely with fewer moving parts and without `devenv`-profile rebuild logic. |
| Runtime-owned binary and asset resolution | Native fit. Current runtime owns `nu`, launch scripts, bundled assets, and prefers runtime-local tools. | Native fit if Core ships the default stack directly and keeps runtime-owned asset resolution explicit. |
| Relationship to generated state | Native fit through the current runtime/config/state split and launch-state model. | Native fit if Core preserves the same split-root ownership model. |
| Reusable activation acceleration | Native fit through cached launch-profile reuse. | Possible but optional. Core should not require a `devenv`-style profile cache if a simpler runtime model makes reuse cheaper or less necessary. |
| Services / long-lived process orchestration | Available today mainly because the current backend stack can support it. | Probably intentionally dropped or kept out of scope unless Core proves it needs them. |

### First-Pass Risk Notes

- Full Yazelix today:
  - strongest fit for flexible environment composition and current command surface
  - highest complexity because backend concerns and product concerns are still partly braided together
- Nix-only `Yazelix Core` candidate:
  - strongest fit when treated as a fixed default-stack product, not as a drop-in replacement for full pack-driven Yazelix
  - likely losses:
    - less flexible environment composition
    - narrower meaning for `yzx env`
    - fewer assumptions about arbitrary extra tools beyond the shipped Core bundle
  - likely gains:
    - simpler install/update story
    - fewer moving backend pieces
    - clearer boundary between shipped runtime and optional host responsibilities

## Non-goals

- choosing a concrete alternative backend now
- defining launch-time dependency checker behavior
- defining the final `Yazelix Core` product boundary
- rewriting the implementation to match the contract yet
- promising that every future backend candidate must preserve every current full-Yazelix convenience

## Acceptance Cases

1. When a later bead asks “can this candidate backend support Yazelix?”, the answer can be evaluated against explicit capability buckets instead of ad hoc intuition.
2. When a later bead asks how `yazelix.toml`, pack config, or Home Manager relate to the backend, the answer is clear: they are backend inputs for materialization and refresh, but their semantics are owned by the config-surface contract rather than the backend contract.
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
  - `nushell/scripts/utils/runtime_env.nu`
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
