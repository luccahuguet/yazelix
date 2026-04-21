# Pane Orchestrator Tab-Local Session State Seam

## Summary

Define one narrow, versioned, plugin-owned read seam for the **active tab's**
workspace/session truth inside Zellij.

The seam should let Nushell and later sidebar/Yazi consumers ask the pane
orchestrator for a typed snapshot instead of combining the maintainer-only
debug payload, the old sidebar-only read seam, and local derivation paths.

This is a **read-only session-truth slice**, not a broader Rust CLI rewrite and
not a second state store outside the plugin.

## Why

The plugin already owns most of the required truth:

- active tab position
- per-tab workspace root and bootstrap-vs-explicit source
- managed editor/sidebar pane identity
- focus context
- layout/sidebar-collapsed state
- sidebar Yazi identity keyed by the managed sidebar pane

But the current read surface is still fragmented:

- `maintainer_debug_editor_state` is a maintainer-only JSON payload, not a
  stable contract
- the old `get_active_sidebar_yazi_state` command was a second read seam just
  for sidebar Yazi
- Nushell still carries some non-authoritative derivation around workspace roots
  and tab-local targeting

`yazelix-0w1u.1` should promote one explicit typed read seam so future consumers
can stop depending on debug payload shape or ad-hoc re-derivation.

## Scope

- pane orchestrator Rust source under `rust_plugins/zellij_pane_orchestrator/`
- one new pipe command for active-tab session state, with a stable typed JSON
  response
- shared serde types in the orchestrator crate when that improves local
  correctness or test clarity
- Nushell transport/client helpers that should consume the new seam first
- docs that define the owner boundary and bootstrap policy

## Rust Dependency Gate

Crate-vs-in-house decision for this bead:

- production crates to keep using:
  - `serde` / `serde_json` for typed JSON payloads already used by the plugin
  - `zellij_tile` for plugin integration and pane/session data
- dev-only crates:
  - none expected beyond the current crate test stack
- build in-house:
  - active-tab snapshot assembly
  - workspace/focus/layout/sidebar field shaping
  - response/version semantics
- rejected additions by default:
  - new RPC frameworks
  - schema/version helper crates
  - general state-management crates
- packaging impact:
  - no new Nix/package impact is expected if the seam stays on the current
    `serde`/`serde_json` path

If implementation discovers a real need for a new crate, update this spec or the
bead notes before continuing.

## Behavior

### Command Name

Add a new pipe command:

- `get_active_tab_session_state`

Do **not** promote `maintainer_debug_editor_state` into the long-term stable
contract just because it is close to the needed payload. Keep debug commands
debug-shaped and introduce one explicit stable read seam instead.

### Success Shape

On success, the plugin returns JSON with this shape:

```json
{
  "schema_version": 1,
  "active_tab_position": 2,
  "workspace": {
    "root": "/home/lucca/pjs/yazelix",
    "source": "explicit"
  },
  "managed_panes": {
    "editor_pane_id": "terminal:7",
    "sidebar_pane_id": "terminal:8"
  },
  "focus_context": "editor",
  "layout": {
    "active_swap_layout_name": "default_sidebar",
    "sidebar_collapsed": false
  },
  "sidebar_yazi": {
    "yazi_id": "sidebar-123",
    "cwd": "/home/lucca/pjs/yazelix"
  }
}
```

### Field Contract

- `schema_version`
  - integer, starts at `1`
- `active_tab_position`
  - Zellij tab position for the active tab
- `workspace.root`
  - current tab workspace root
- `workspace.source`
  - `"bootstrap"` or `"explicit"`
- `managed_panes.editor_pane_id`
  - current tab managed editor pane id, or `null` when missing
- `managed_panes.sidebar_pane_id`
  - current tab managed sidebar pane id, or `null` when missing
- `focus_context`
  - `"editor"`, `"sidebar"`, or `"other"`
- `layout.active_swap_layout_name`
  - current active swap layout name, or `null`
- `layout.sidebar_collapsed`
  - `true` / `false` when layout state is known, or `null` when unknown
- `sidebar_yazi`
  - `null` when no current-tab sidebar Yazi state is registered
  - otherwise `{ yazi_id, cwd }`

The seam should be assembled from the plugin's existing tab-local state:

- `workspace_state_by_tab`
- `managed_panes_by_tab`
- `focus_context_by_tab`
- `active_swap_layout_name_by_tab`
- current layout variant / `is_sidebar_closed()`
- `get_active_sidebar_yazi_state_snapshot(active_tab_position)`

### Failure Shape

Keep the current transport style for v1:

- `permissions_denied`
- `not_ready`
- `missing`
- `invalid_payload` only if the command later accepts a payload and validation fails

That avoids forcing a larger Nushell transport rewrite in the same slice.

### Bootstrap Policy

This seam must preserve the plugin's **actual** bootstrap policy:

- new tabs bootstrap from the plugin's `initial_cwd`
- they do **not** bootstrap from a fresh filesystem probe or from `HOME` by default

The docs should say this explicitly so the acceptance criteria match the real
implementation.

## First Consumers

Composer 2 should prepare to switch these read paths first:

- `nushell/scripts/integrations/zellij.nu`
  - `read_current_tab_workspace_root`
  - `get_current_tab_workspace_root_including_bootstrap`
  - any helper that currently parses `maintainer_debug_editor_state`
- sidebar/Yazi-facing integrations that need active-tab identity rather than a
  session-global cache view
- diagnostics that currently read `maintainer_debug_editor_state` for tab-local
  truth

`retarget_workspace` remains the mutation seam for workspace changes in this
bead. This slice is only about the read contract.

## Non-goals

- cross-tab snapshot enumeration
- a new Rust CLI or `yzx_core` command family
- moving path resolution, `zoxide`, repo-root inference, or Yazi `emit-to`
  execution into Rust
- a full pane manifest export
- replacing `retarget_workspace`
- deleting the sidebar Yazi cache in the same slice

## Acceptance Cases

1. The pane orchestrator exposes one explicit typed command for active-tab
   session truth instead of making consumers rely on
   `maintainer_debug_editor_state`.
2. The typed payload includes workspace root/source, managed editor/sidebar pane
   identity, focus context, and sidebar visibility/open-state information.
3. Sidebar Yazi identity comes from current-tab plugin state validation, not
   from a session-global cache scan.
4. The documented bootstrap policy matches the real plugin behavior
   (`initial_cwd`).
5. Composer 2 can identify the exact Rust fields/functions and the first Nushell
   consumers to change without redoing architecture discovery.

## Verification

- spec validation:
  - `nu nushell/scripts/dev/validate_specs.nu`
- Rust/plugin verification after implementation:
  - `cargo test --manifest-path rust_plugins/zellij_pane_orchestrator/Cargo.toml --lib`
  - `yzx dev build_pane_orchestrator --sync`
- focused Nushell verification after implementation:
  - `nu -c 'source nushell/scripts/dev/test_yzx_workspace_commands.nu; [(test_run_pane_orchestrator_command_raw_targets_session_plugin_without_plugin_configuration) (test_retarget_workspace_for_path_returns_plugin_owned_sidebar_state_and_editor_status)]'`

## Traceability

- Bead: `yazelix-0w1u.1`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
- Defended by: `nu -c 'source nushell/scripts/dev/test_yzx_workspace_commands.nu; [(test_run_pane_orchestrator_command_raw_targets_session_plugin_without_plugin_configuration) (test_retarget_workspace_for_path_returns_plugin_owned_sidebar_state_and_editor_status)]'`

## Open Questions

- Resolved 2026-04-20: the remaining debug surface should survive only as the
  explicitly maintainer-only command `maintainer_debug_editor_state`, not as a
  production-facing helper name.
- Should the first stable seam include `permissions_granted`, or should
  permission/readiness stay encoded only in the non-JSON transport tokens?
- Resolved 2026-04-20: `get_active_sidebar_yazi_state` should not survive as a
  public compatibility read seam; later consumers should converge on the typed
  session snapshot.
