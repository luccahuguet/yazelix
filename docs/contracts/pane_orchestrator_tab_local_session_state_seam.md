# Pane Orchestrator Tab-Local Session State Seam

## Summary

Define one narrow, versioned, plugin-owned read seam for the **active tab's**
workspace/session truth inside Zellij.

The seam should let Yazelix control code and later sidebar/Yazi consumers ask the pane
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
- Yazelix control code should not carry non-authoritative derivation around
  workspace roots and tab-local targeting

The pane orchestrator should expose one explicit typed read seam so future
consumers can stop depending on debug payload shape or ad-hoc re-derivation.

## Scope

- pane orchestrator Rust source in the external `yazelix-zellij-pane-orchestrator` project
- versioned pipe commands for active-tab session state and all-tab activity
  state, with stable typed JSON responses
- shared serde types in the orchestrator crate when that improves local
  correctness or test clarity
- Yazelix control transport/client helpers that should consume the new seam first
- docs that define the owner boundary and bootstrap policy
- AI activity extension facts and their tab-name decoration product surface;
  this contract exposes all-tab activity facts for a future bar-owned renderer,
  while native tab-name decoration remains the current fallback bridge

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
- `get_all_tab_activity_state`

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
    ]
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

### Activity Tab Decoration

The pane orchestrator owns the live activity facts and terminal-title activity
signals that feed visible tab-name decoration.

`register_ai_pane_activity` accepts tab-local activity observations. When at
least one current-tab fact is `stale`, the orchestrator prefixes the Zellij tab
name with `[!] `. Otherwise, when at least one fact is `active` or `thinking`,
the orchestrator prefixes the tab name with `[...] `. Alert takes priority over
busy, and busy takes priority over no marker. When the tab has no alert or busy
facts, the orchestrator restores the recorded base tab name and clears only the
marker it added.

The orchestrator can also treat a live spinner-prefixed terminal title, such as
Codex's `⠋ project` title, as a busy signal. The tab still uses the stable
`[...] ` marker instead of mirroring every spinner frame into the tab name.
This lets the tab carry the same activity state a pane frame already shows
without renaming the tab for every animation frame.

Terminal-title activity is remembered by producing pane. When a pane was
observed busy through a spinner-prefixed title and then stops using the activity
shape while that pane is not focused, the orchestrator records the pane as
`stale` and the tab renders `[!] `. The marker clears only when the producing
pane is focused again or when that pane disappears. If the activity title clears
while the producing pane is already focused, the activity is considered
acknowledged and the orchestrator restores the recorded base tab name.

This uses Zellij's native tab name as the bridge into the status bar. The
`yazelix_zellij_bar` child repo still owns tab label formatting and only renders
the marker when the selected tab label mode includes `{name}`. Compact tab mode
intentionally hides tab names, so it also hides this activity marker.

The native tab-name bridge is a fallback, not the long-term rendering owner.
For bar-owned rendering, `get_all_tab_activity_state` returns a separate
versioned JSON payload:

```json
{
  "schema_version": 1,
  "tabs": [
    {
      "tab_id": 30,
      "tab_position": 2,
      "base_name": "agent",
      "activity_state": "alert",
      "activity": [
        {
          "tab_position": 2,
          "provider": "terminal-title",
          "pane_id": "terminal:12",
          "activity": "stale",
          "state": "stale"
        }
      ]
    }
  ]
}
```

`activity_state` is reduced to `"alert"`, `"busy"`, or `"idle"` with the same
priority as native tab-name decoration. `base_name` is the clean tab name the
bar should render; when the fallback bridge has already mutated a native tab
name, the orchestrator uses its recorded base name instead of exposing the
decorated display name as source truth. The all-tab payload carries facts and
state only. Presentation strings such as `[!]`, `[...]`, colors, and tab-label
spacing belong to `yazelix_zellij_bar`.

The markers are deliberately ASCII. Terminal-title activity is an input signal,
not tab-label text, because high-frequency terminal-title animation must not
become high-frequency tab renaming. Native tab-name writes are coalesced and
rate-limited; internal activity state may update frequently, but the Zellij
rename side effect must only run for reduced visible state changes.

The seam should be assembled from the plugin's existing tab-local state:

- `workspace_state_by_tab`
- `managed_panes_by_tab`
- `focus_context_by_tab`
- `active_swap_layout_name_by_tab`
- current layout variant / `is_sidebar_closed()`
- `get_active_sidebar_yazi_state_snapshot(active_tab_position)`
- `ai_pane_activity_by_tab`

### Sidebar Yazi Identity Decision

Yazelix must not derive the managed sidebar Yazi client id deterministically
from Zellij pane, tab, or session identity.

Yazi's `--client-id` accepts a numeric `u64` and documents that value as a
globally unique client id. In the current Yazi DDS server, a repeated id replaces
the previous client entry for that id. A Yazelix-side hash of
`ZELLIJ_SESSION_NAME`, `ZELLIJ_TAB_POSITION`, and `ZELLIJ_PANE_ID` would
therefore be a best-effort collision-avoidance scheme, not a safe ownership
contract.

Deterministic ids also do not remove the sidebar registration seam. The pane
orchestrator needs the active Yazi `cwd`, and that value is live Yazi state that
changes on Yazi `cd` and tab events. The current registration payload includes
both the Yazi id and the current cwd, and the orchestrator accepts it only when
the reported pane id is the current tab's managed sidebar pane. Reconciliation
then drops sidebar Yazi state when the associated managed sidebar pane is no
longer live.

The supported identity model is:

1. Let Yazi allocate its own globally unique `YAZI_ID`.
2. Have the bundled sidebar-state plugin publish `{ pane_id, yazi_id, cwd }`.
3. Store that state only after validating the pane id against the live managed
   sidebar pane for one tab.
4. Keep cwd updates on Yazi navigation events.

If startup registration remains flaky, improve the register path with bounded
retry, acknowledgement, or fresher event triggers. Do not replace it with a
parallel deterministic-id derivation layer.

### Failure Shape

Keep the current transport style for v1:

- `permissions_denied`
- `not_ready`
- `missing`
- `invalid_payload` only if the command later accepts a payload and validation fails

That avoids forcing a larger control-transport rewrite in the same slice.

### Bootstrap Policy

This seam must preserve the plugin's **actual** bootstrap policy:

- new tabs bootstrap from the plugin's `initial_cwd`
- they do **not** bootstrap from a fresh filesystem probe or from `HOME` by default

The docs should say this explicitly so the acceptance criteria match the real
implementation.

## Current Consumers

Current read paths include:

- `rust_core/yazelix_core/src/zellij_commands/status.rs`
  - active-tab status-bus and inspect-session reads
- `rust_core/yazelix_core/src/zellij_commands/workspace.rs`
  - active-tab workspace root and sidebar-state probes
- `rust_core/yazelix_core/src/workspace_commands.rs`
  - workspace retargeting through `retarget_workspace`
- `rust_core/yazelix_core/src/workspace_commands/yazi_sidebar.rs`
  - sidebar refresh and reveal state
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
- provider SDK integration or provider-specific quota adapters
- bar colors, labels, or other provider-specific presentation rules inside the
  pane orchestrator
- provider-specific activity detection beyond accepting normalized activity
  facts through the plugin pipe API or recognizing existing terminal-title
  activity signals
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
5. Maintainers can identify the exact Rust fields/functions and current control
   consumers without redoing architecture discovery.
6. AI pane activity facts remain tab-local and can represent inactive,
   active/thinking, stale, and unknown states.
7. Active/thinking AI pane activity facts, live spinner-prefixed terminal
   titles, and completed off-focus terminal-title activity decorate full tab
   labels through stable tab-name markers without adding a separate bar widget,
   changing compact tab mode, or renaming the tab for every terminal-title
   animation frame. Completed terminal-title activity clears only when the
   producing pane is focused again or disappears.
8. The pane orchestrator writes active-tab status facts to a launch-scoped
   status-bar cache, and zjstatus dynamic widgets read only that cache instead
   of opening pane-orchestrator pipes from the bar.
9. The `cursor` widget reads the launch-scoped cursor fact from that cache and
   renders a compact cursor glyph plus the resolved preset name.
10. Agent-usage facts are produced by throttled cache writers with provider
   command timeouts, but zjstatus usage widgets must never run usage providers
   directly. The `claude_usage` and `codex_usage` widgets read shared
   cross-window caches. `codex_usage` renders 5-hour and weekly reset-window
   timing plus quota percentages by default, with token totals available through
   `codex_usage_display = "both"`. The `opencode_go_usage` widget reads a shared
   cross-window cache and renders its configured 5-hour, weekly, and monthly
   token/quota windows.
11. Shared agent-usage cache and lock filenames are scoped by provider cache
    schema version, so runtime versions with different cache contracts do not
    read from or write to the same files.

## Verification

- contract validation:
  - `yzx_repo_validator validate-contracts`
- Rust/plugin verification after implementation:
  - `cargo test --manifest-path ../yazelix-zellij-pane-orchestrator/Cargo.toml --lib`
  - `nix build ../yazelix-zellij-pane-orchestrator#yazelix_zellij_pane_orchestrator --no-link`
  - `nix build .#runtime --override-input yazelixZellijPaneOrchestrator ../yazelix-zellij-pane-orchestrator --no-link`
- Rust/core verification after implementation:
  - `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache`
  - `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core agent_usage`
- focused Yazelix control verification after implementation:
  - `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_control_workspace_surface`

## Traceability
- Defended by: `yzx_repo_validator validate-contracts`
- Defended by: `cargo test --manifest-path ../yazelix-zellij-pane-orchestrator/Cargo.toml --lib ai_activity_extension_represents_tab_local_state_taxonomy`
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache_round_trip_renders_cached_workspace_fact`
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache_claude_usage_refresh_writes_shared_combined_cache`
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache_codex_usage_renders_5h_week_display_modes`
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache_codex_usage_refresh_writes_shared_combined_cache`
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache_claude_usage_renders_5h_week_display_modes`
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_control_workspace_surface`

## Open Questions

- Resolved 2026-04-20: the remaining debug surface should survive only as the
  explicitly maintainer-only command `maintainer_debug_editor_state`, not as a
  production-facing helper name.
- Should the first stable seam include `permissions_granted`, or should
  permission/readiness stay encoded only in the non-JSON transport tokens?
- Resolved 2026-04-20: `get_active_sidebar_yazi_state` should not survive as a
  public compatibility read seam; later consumers should converge on the typed
  session snapshot.
