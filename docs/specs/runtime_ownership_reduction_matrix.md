# Runtime Ownership Reduction Matrix

> Status: Historical pre-v15-trim planning note.
> This matrix explores alternative shapes that still preserved pack sidecars, installer-owned runtime identity, or launch-profile reuse.
> Do not treat it as the current branch contract. See [v15_trimmed_runtime_contract.md](./v15_trimmed_runtime_contract.md).

## Summary

Yazelix should distinguish clearly between two different kinds of reduction work:

1. deleting distribution ownership
2. deleting backend ownership

Those are related, but they are not the same cut.

The important consequence is:

- deleting installer-owned runtime and launcher management does **not** require deleting `yazelix.toml` or `yazelix_packs.toml`
- deleting backend/environment ownership is the stronger reduction, and that is the point where packs and other environment-shaping semantics become questionable

## Why

Architecture discussions around a “smaller Yazelix” keep colliding because the repo currently bundles multiple responsibilities under the broad word “runtime”:

- dynamic user intent
- shipped runtime code
- backend/environment materialization
- distribution/install ownership

The current contracts already separate those concerns:

- [Config Surface And Launch Profile Contract](./config_surface_and_launch_profile_contract.md)
- [Backend Capability Contract](./backend_capability_contract.md)
- [Runtime Distribution Capability Tiers](./runtime_distribution_capability_tiers.md)
- [Backend-Free Workspace Slice](./backend_free_workspace_slice.md)

But when discussing streamlining, it is still too easy to jump from “delete installer-owned `runtime/current`” to “then Yazelix cannot listen to `yazelix.toml` anymore,” which is not true.

## Scope

- define the difference between distribution ownership and backend ownership
- define the main reduction shapes Yazelix can talk about coherently
- state what happens to `yazelix.toml` and `yazelix_packs.toml` in each shape
- state which command families survive, narrow, or disappear in each shape

## Behavior

### Ownership Axes

Yazelix currently spans these distinct concerns:

1. Dynamic user intent
   - `~/.config/yazelix/user_configs/yazelix.toml`
   - `~/.config/yazelix/user_configs/yazelix_packs.toml`
2. Deterministic shipped runtime
   - runtime scripts
   - templates
   - bundled assets
   - Rust plugins
3. Backend/environment ownership
   - environment materialization
   - rebuild/refresh semantics
   - runtime tool availability
   - launch-profile reuse
4. Distribution/install ownership
   - installed `runtime/current`
   - stable `yzx` launcher ownership
   - installer-owned repair/update flows

Deleting one axis does not automatically delete the others.

### Reduction Matrix

| Shape | Distribution ownership | Backend ownership | `yazelix.toml` | `yazelix_packs.toml` | Likely command-surface effect |
| --- | --- | --- | --- | --- | --- |
| Full Yazelix \(current model\) | Keep | Keep | Keep as canonical user intent | Keep as canonical pack intent | Current integrated surface remains valid |
| Package-runtime-only Yazelix | Delete | Keep | Keep | Usually keep | Drop installer/update/repair ownership, but keep environment materialization and config-driven runtime behavior |
| Backend-free workspace-only slice | Delete | Delete | Keep only the workspace/config subset that still has meaning without backend provisioning | Narrow heavily, move out of Yazelix, or drop | Keep workspace/session/config UX leaves; backend-bound control-plane commands become invalid or out of scope |

### Package-Runtime-Only Reduction

This is the smaller reduction.

What gets deleted:

- installer-owned `~/.local/share/yazelix/runtime/current` as a product promise
- stable-launcher ownership/repair as a product promise
- a generic in-app runtime-update surface
- most install-artifact doctoring
- the need for separate user-facing runtime/distribution stories such as:
  - installer-managed
  - Home Manager-managed
  - package runtime
  - runtime-root-only

What remains:

- `yazelix.toml` remains canonical dynamic user intent
- `yazelix_packs.toml` can remain canonical pack intent
- backend materialization still exists
- refresh/rebuild behavior still exists
- generated configs and launch-profile caching can still exist

Why the config files survive:

- `yazelix.toml` and `yazelix_packs.toml` are owned by the config-surface contract, not by installer ownership
- package/runtime installation and dynamic user intent are different layers
- a package-provided Yazelix runtime can still read user intent and materialize a user-specific environment from it

This means the following families can still make sense after deleting distribution ownership:

- `yzx launch`
- `yzx enter`
- `yzx env`
- `yzx run`
- internal generated-state repair helpers
- config/edit/import flows
- workspace/session commands

The main change is that Nix, Home Manager, or another package provider owns install/update transitions instead of Yazelix itself.

### Backend-Free Workspace-Only Reduction

This is the stronger reduction.

What gets deleted in addition to the package-runtime-only cut:

- environment materialization as a Yazelix-owned responsibility
- refresh/rebuild ownership
- launch-profile caching as a first-class Yazelix behavior
- the promise that Yazelix provisions the tool/runtime environment needed by backend-bound commands

What survives cleanly:

- workspace/session commands such as `yzx cwd`, `yzx reveal`, `yzx popup`, and similar leaf actions
- lightweight config-target resolution such as `yzx edit`
- selected discoverability or informational commands

What becomes questionable or likely out of scope:

- `yzx env`
- `yzx run`
- internal generated-state repair helpers
- `yzx launch` if it still depends on backend activation semantics
- `yzx packs` as a first-class product surface

Why packs become unstable here:

- packs currently shape the provisioned environment and installed tool graph
- once Yazelix stops owning backend/environment materialization, pack intent no longer has a clear first-class execution owner inside Yazelix
- at that point, pack semantics likely need to move upward into Nix/Home Manager, shrink into a narrower declarative hint layer, or disappear

`yazelix.toml` does not necessarily disappear in this stronger mode, but it narrows:

- workspace UX settings can survive
- editor/sidebar/session behavior can survive
- backend-shaping settings lose meaning unless some new owner is defined for them

### Practical Reading Rule

When discussing streamlining:

- “delete install ownership” means Yazelix stops managing installed runtime identity and update/repair artifacts
- “delete backend ownership” means Yazelix stops materializing and activating the environment itself

Only the second statement puts packs and backend-driven config meaning at risk.

## Non-goals

- committing Yazelix to ship a separate Core edition now
- choosing a final future packaging story now
- deleting pack support in the current product
- redefining the config schema in this spec alone

## Acceptance Cases

1. A maintainer can explain the difference between deleting installer/distribution ownership and deleting backend ownership without collapsing them into one idea.
2. A maintainer can answer whether package-runtime-only Yazelix still listens to `yazelix.toml` and `yazelix_packs.toml` clearly: yes.
3. A maintainer can answer when pack semantics become questionable clearly: when backend/environment ownership is also removed.
4. Later streamlining or Core-boundary discussion can point at a stable matrix instead of re-arguing the same distinction.

## Verification

- manual review against:
  - [architecture_map.md](../architecture_map.md)
  - [config_surface_and_launch_profile_contract.md](./config_surface_and_launch_profile_contract.md)
  - [backend_capability_contract.md](./backend_capability_contract.md)
  - [runtime_distribution_capability_tiers.md](./runtime_distribution_capability_tiers.md)
  - [backend_free_workspace_slice.md](./backend_free_workspace_slice.md)
  - [yazelix_core_boundary.md](./yazelix_core_boundary.md)
- spec validation:
  - `yzx_repo_validator validate-specs`

## Traceability

- Bead: `yazelix-sxhc`
- Defended by: `yzx_repo_validator validate-specs`
