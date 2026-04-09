# Cross-Language Runtime Ownership

## Summary

Yazelix should keep hot-path correctness concentrated in as few language/runtime owners as possible.

Current recommendation:

- Nushell owns CLI UX, config semantics, runtime activation, and generated-config orchestration.
- Rust Zellij plugins own live workspace/session state.
- Lua Yazi plugins stay thin in-Yazi adapters.
- POSIX shell stays limited to installer/launcher glue.
- Zellij CLI and KDL stay transport/static-shape layers, not state owners.

The point is not to rewrite everything. It is to stop letting one user-visible behavior depend equally on POSIX shell, Nushell, Rust, Lua, and Zellij CLI state at once.

## Why

Yazelix is hardest to debug when correctness crosses too many runtimes:

- a workspace bug spans Nushell, the pane orchestrator, and the Yazi cache
- a launch/runtime bug spans install glue, backend activation, and recorded profile state
- a focus or tab-root bug crosses static KDL, runtime plugin state, and post-hoc shell sync

That stack can work, but it is expensive. The right delete-first move is to decide which layer should own the truth and keep the others thin.

## Scope

- define language/runtime owners for the main hot paths
- identify which layers are durable owners versus adapters
- identify at least one high-value seam that should collapse further
- support later Rust migration planning without forcing it now

## Ownership Map

| Layer | Should own | Should not own |
| --- | --- | --- |
| POSIX shell | installer entrypoints, stable launcher glue, narrow host integration scripts | workspace state, config semantics, launch-profile freshness, tab/workspace routing |
| Nushell | `yzx` CLI UX, config parsing/migration, backend activation semantics, generated config orchestration, explicit integration glue | authoritative live tab state, long-lived pane identity, in-Yazi UI state |
| Rust pane orchestrator | authoritative per-tab workspace root, managed pane identity, focus/layout/sidebar state, tab-local workspace mutations | high-level config semantics, import flows, backend refresh policy |
| Lua Yazi plugins | in-Yazi keymaps/status UI, small adapter events, local cache writes when needed | workspace source of truth, tab identity, launch/runtime policy |
| Zellij CLI/KDL | command transport and static layout/config shape | durable workspace truth, business logic, config ownership |

## Hot-Path Classification

### Runtime Activation And Refresh

Primary owner: Nushell

- `devenv_backend.nu`
- `launch_state.nu`
- `config_state.nu`
- `yzx env`
- `yzx run`
- `yzx refresh`
- `yzx launch`
- `start_yazelix`

Reason:

- these paths are about config intent, rebuild freshness, launch-profile reuse, and process activation
- they are not plugin-state problems

### Live Workspace And Session State

Primary owner: Rust pane orchestrator

- current tab workspace root
- managed editor/sidebar identity
- focus context
- layout/sidebar open or collapsed state
- tab-local mutations such as editor/sidebar focus toggle

Reason:

- this is session-local truth inside Zellij
- Nushell can request mutations, but it should not be the durable owner

### In-Yazi Behavior

Primary owner: Lua, but only as an adapter

- Yazi keymap behavior
- Yazi plugin setup/init flows
- small local status/sidebar UI behavior

Reason:

- this logic belongs inside Yazi when it is purely in-Yazi UX
- once the behavior becomes tab/workspace truth, it should move back out to the plugin or Nushell owner

### Static Layout And Launch Transport

Primary owner: Zellij KDL/CLI plus thin Nushell generators

- generated layouts
- `zellij run` / `zellij action`
- declared plugin wiring

Reason:

- these are transport and declarative shape surfaces
- they should not grow into state owners

## High-Value Collapse To Keep Pursuing

The best remaining cross-language collapse is:

- keep authoritative workspace/session truth in the Rust pane orchestrator
- keep Yazi Lua as an adapter
- keep Nushell out of re-deriving pane identity or focus truth when the plugin already knows it

That means:

- workspace root and pane identity should keep moving toward plugin-owned truth
- sidebar cache files should stay integration cache, not become a second workspace model
- future Rust migration should prioritize session-state slices rather than generic CLI rewrites

## What Should Stay In Nushell

Not everything should migrate:

- config semantics
- migration planning/apply rules
- backend activation policy
- explicit user-facing command UX
- install/update/distribution glue

Those are product/control-plane concerns, and Nushell is already the right owner for them in main.

## Non-goals

- rewriting all glue into Rust now
- removing Lua or Zellij CLI usage entirely
- forcing a second backend implementation
- using ownership mapping as an excuse to duplicate config or runtime semantics

## Acceptance Cases

1. There is an explicit language/runtime map for CLI UX, backend activation, live workspace/session state, in-Yazi behavior, and static layout/transport.
2. The map identifies the pane orchestrator as the long-lived owner for workspace/session truth.
3. The map identifies Lua Yazi code as an adapter layer, not a workspace truth owner.
4. Later Rust migration planning can use this map instead of guessing where the first high-value slices are.

## Verification

- manual review against [architecture_map.md](../architecture_map.md)
- manual review against [workspace_session_contract.md](../workspace_session_contract.md)
- manual review against [yzx_command_surface_backend_coupling.md](./yzx_command_surface_backend_coupling.md)
- spec validation: `nu nushell/scripts/dev/validate_specs.nu`

## Traceability

- Bead: `yazelix-ewjz.1`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
