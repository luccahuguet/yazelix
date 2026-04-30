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

The pane orchestrator should expose one explicit typed read seam so future
consumers can stop depending on debug payload shape or ad-hoc re-derivation.

## Scope

- pane orchestrator Rust source under `rust_plugins/zellij_pane_orchestrator/`
- one new pipe command for active-tab session state, with a stable typed JSON
  response
- shared serde types in the orchestrator crate when that improves local
  correctness or test clarity
- Nushell transport/client helpers that should consume the new seam first
- docs that define the owner boundary and bootstrap policy
- AI activity and token-budget extension slots that carry facts only, leaving
  provider-specific UI formatting to status-bar consumers

## Rust Dependency Gate

Crate-vs-in-house decision for this contract:

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

If implementation discovers a real need for a new crate, update this contract or
the implementation notes before continuing.

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
  },
  "extensions": {
    "ai_pane_activity": [
      {
        "tab_position": 2,
        "provider": "codex",
        "pane_id": "terminal:9",
        "activity": "thinking",
        "state": "thinking"
      }
    ],
    "ai_token_budget": []
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
- `extensions.ai_pane_activity`
  - empty means unknown, not idle
  - facts are tab-local and may represent `unknown`, `inactive`, `active`,
    `thinking`, or `stale`
  - `activity` remains a schema-v1 compatibility token; consumers should prefer
    the normalized `state`
- `extensions.ai_token_budget`
  - empty means unknown
  - future provider adapters may publish `{ tab_position, provider,
    remaining_tokens, total_tokens }`
  - the pane orchestrator owns only the fact transport, not provider-specific
    token accounting

The seam should be assembled from the plugin's existing tab-local state:

- `workspace_state_by_tab`
- `managed_panes_by_tab`
- `focus_context_by_tab`
- `active_swap_layout_name_by_tab`
- current layout variant / `is_sidebar_closed()`
- `get_active_sidebar_yazi_state_snapshot(active_tab_position)`
- `ai_pane_activity_by_tab`

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
contract. This slice is only about the read contract.

## Non-goals

- cross-tab snapshot enumeration
- a new Rust CLI or `yzx_core` command family
- moving path resolution, `zoxide`, repo-root inference, or Yazi `emit-to`
  execution into Rust
- a full pane manifest export
- provider SDK integration or provider-specific token-budget adapters
- bar colors, labels, or other provider-specific presentation rules inside the
  pane orchestrator
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
6. AI pane activity facts remain tab-local and can represent inactive,
   active/thinking, stale, and unknown states.
7. The pane orchestrator writes active-tab status facts to a launch-scoped
   status-bar cache, and zjstatus dynamic widgets read only that cache instead
   of opening pane-orchestrator pipes from the bar.
8. Agent-usage facts are produced by throttled cache writers with provider
   command timeouts. New windows may seed their first paint from recent sibling
   session cache facts, but zjstatus usage widgets must never run usage
   providers directly. The grouped provider widgets (`claude_usage` and
   `opencode_usage`) render configured period lists as one compact segment so
   the provider name is not repeated for day/month facts. The `codex_usage`
   widget reads a shared cross-window cache and renders only the 5-hour and
   weekly token/quota windows.

## Verification

- contract validation:
  - `yzx_repo_validator validate-contracts`
- Rust/plugin verification after implementation:
  - `cargo test --manifest-path rust_plugins/zellij_pane_orchestrator/Cargo.toml --lib`
  - `yzx dev build_pane_orchestrator --sync`
- Rust/core verification after implementation:
  - `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache`
  - `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core agent_usage`
- focused Nushell verification after implementation:
  - `nu -c 'source nushell/scripts/dev/test_yzx_workspace_commands.nu; [(test_run_pane_orchestrator_command_raw_targets_session_plugin_without_plugin_configuration) (test_retarget_workspace_for_path_returns_plugin_owned_sidebar_state_and_editor_status)]'`

## Traceability
- Defended by: `yzx_repo_validator validate-contracts`
- Defended by: `cargo test --manifest-path rust_plugins/zellij_pane_orchestrator/Cargo.toml --lib ai_activity_extension_represents_tab_local_state_taxonomy`
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_bus_ai_activity_widget_formats_highest_priority_fact`
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache_round_trip_renders_cached_workspace_fact`
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache_agent_usage_refresh_writes_precomputed_summary`
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache_write_seeds_agent_usage_from_recent_sibling_session_cache`
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache_codex_usage_renders_5h_week_display_modes`
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache_codex_usage_refresh_writes_shared_combined_cache`
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache_grouped_claude_usage_renders_configured_periods_compactly`
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
