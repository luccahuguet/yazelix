# Cross-Language Runtime Ownership

## Summary

Yazelix should keep hot-path correctness concentrated in as few language and
runtime owners as possible.

Current recommendation:

- Rust `yzx_core` owns typed config, state, preflight, runtime-env, and
  structured report evaluation
- Rust `yzx_control` owns the public control-plane leaf parsing and execution
  for `yzx env`, `yzx run`, and `yzx update*`
- Nushell owns the remaining public CLI UX, process orchestration, generated
  file families that still live there, and final human rendering
- Rust pane orchestrator code owns live workspace and session truth inside
  Zellij
- Lua Yazi plugins stay thin in-Yazi adapters
- POSIX shell stays limited to stable launcher and host bootstrap glue

The next delete-first move is not a broad CLI rewrite. The next delete-first
move is to collapse the remaining Nu bridge owners and then delete one real
generator and materialization owner family end-to-end.

## Why

Yazelix is hardest to debug when one user-visible behavior depends equally on
POSIX shell, Nushell, Rust core, Rust plugin state, Lua, and Zellij transport.

The current branch is better than the old Classic-era stack, but mixed ownership
still exists in two important places:

- bridge-layer Nu files still surround Rust helper owners
- generated runtime materialization is still split between Rust planning and Nu
  orchestration or writing

The delete-first answer is to make those boundaries smaller, not to add one more
wrapper runtime.

## Scope

- define the current language and runtime owners for the main hot paths
- distinguish durable owners from adapters
- identify the highest-value remaining cross-language collapse
- support later delete-first Rust planning without forcing a broad public-CLI
  rewrite first

## Ownership Map

| Layer | Should own | Should not own |
| --- | --- | --- |
| POSIX shell | stable launcher entrypoints, narrow host bootstrap, runtime-root discovery, shell-specific wrappers | config semantics, runtime classification, workspace truth, long-lived generated-state policy |
| Rust `yzx_core` | typed config normalization, config-state hashing and recording, runtime-env computation, runtime preflight evaluation, materialization planning, structured status and doctor data, structured install ownership evaluation, structured render plans | public CLI UX, shell and process orchestration, final human prose, authoritative live workspace state |
| Rust `yzx_control` | public control-plane leaf parsing and execution for `yzx env`, `yzx run`, and `yzx update*` | becoming a second general public command registry while Nushell still owns the rest of help and completion |
| Nushell | remaining public `yzx` CLI UX, command help, startup profile schema, shell and terminal orchestration, generated file families that still live in Nu, final human rendering and integration glue | typed runtime truth already owned by `yzx_core`, authoritative live tab state already owned by the pane orchestrator |
| Rust pane orchestrator | authoritative per-tab workspace root, managed pane identity, focus and layout state, tab-local sidebar state, tab-local mutations | high-level config semantics, runtime/update policy, install/distribution ownership |
| Lua Yazi plugins | in-Yazi keymaps and status UI, small adapter events, local cache writes when needed | workspace source of truth, runtime policy, tab identity |
| Zellij CLI and KDL | command transport and static layout or config shape | durable workspace truth, generated-runtime business logic, config ownership |

## Hot-Path Classification

### Runtime Activation And Refresh

Current owner split:

- Rust `yzx_core` owns typed config normalization, config-state computation,
  runtime-env planning, runtime preflight, and structured runtime findings
- Rust `yzx_control` owns the already migrated `env`, `run`, and `update*`
  public control-plane leaves
- Nushell and POSIX shell still own launch, startup, terminal dispatch, startup
  profiles, and the remaining generated-state orchestration

This path is no longer "Nushell owns runtime activation." It is a mixed owner
path with a clear next delete target: the surviving Nu bridge and materialization
owners.

### Generated Runtime Materialization

Primary current owner: mixed

- Rust already owns materialization planning, repair evaluation, and Yazi or
  Zellij render plans
- Nushell still owns the main orchestration and writer families around
  `generated_runtime_state.nu`, the Yazi generation family, the Zellij
  generation family, and the terminal or Helix or initializer families

This is the highest-value remaining cross-language collapse because it still
holds large real product ownership in Nushell.

### Live Workspace And Session State

Primary owner: Rust pane orchestrator

- current tab workspace root
- managed editor and sidebar identity
- focus context
- layout and sidebar-collapsed state
- tab-local workspace mutations

Nushell can request mutations or consume state, but it should not re-derive that
truth once the plugin already owns it.

### In-Yazi Behavior

Primary owner: Lua, but only as an adapter

- Yazi keymap behavior
- Yazi plugin setup and init flows
- small local status and sidebar UI behavior

Once behavior becomes tab or workspace truth, it should move back out to the
plugin or runtime owner instead of becoming a second Lua state model.

### Static Layout And Launch Transport

Primary owner: Zellij KDL and CLI plus thin surviving generators

- generated layouts
- `zellij run` and `zellij action`
- declared plugin wiring

These are transport and declarative-shape surfaces. They should not grow into
durable business-logic owners.

## High-Value Collapse To Keep Pursuing

The best remaining cross-language collapse is now:

1. collapse the Nu bridge layer around `yzx_core` and `yzx_control`
2. move one generated runtime materialization family to full Rust ownership
3. defer any broad public-CLI rewrite until it deletes the public command
   registry and extern ownership too

That means:

- `config_parser.nu` and the per-command bridge files should stop surviving as a
  second ownership layer
- `generated_runtime_state.nu` should either stay as an honest Nu owner or lose
  ownership end-to-end
- the Yazi, Zellij, and terminal generation families should be judged by
  deletion budget, not by helper count

## What Should Stay In Nushell

Not everything should migrate:

- remaining public CLI UX and help for intentionally Nu-owned command families
- startup profile schema and process orchestration
- shell and terminal host integration
- final human-facing remediation text and interactive UX
- explicit integration glue around external tools when the hard part is host
  behavior rather than typed domain modeling

Those are still better Nushell fits than the current bridge and materialization
owners.

## Non-goals

- rewriting all glue into Rust now
- removing Lua or Zellij CLI usage entirely
- using ownership mapping as an excuse to duplicate config or runtime semantics
- treating a broad Rust CLI rewrite as the default next step

## Acceptance Cases

1. There is an explicit owner map for Rust core, Rust control-plane leaves,
   remaining Nushell UX and orchestration, live session state, Lua adapters, and
   POSIX glue
2. The map reflects the Rust helper slices that already landed instead of
   describing them as future work
3. The map identifies bridge collapse and full-owner materialization migration
   as the next high-value cuts
4. Later Rust planning can use this map instead of reopening the owner question
   from scratch

## Verification

- manual review against [architecture_map.md](../architecture_map.md)
- manual review against [workspace_session_contract.md](../workspace_session_contract.md)
- manual review against [rust_migration_matrix.md](./rust_migration_matrix.md)
- spec validation: `nu nushell/scripts/dev/validate_specs.nu`

## Traceability

- Bead: `yazelix-ewjz.1`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
