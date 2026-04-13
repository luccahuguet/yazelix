# Workspace Session Contract

This document defines the current contract between Yazelix's Nushell layer and the Zellij pane orchestrator plugin.

The goal is not to describe every implementation detail. The goal is to make clear:

- what state exists
- which side owns it
- which commands are allowed to mutate it
- which bugs should be treated as contract breaks rather than one-off regressions

## Why This Contract Exists

Many Yazelix UX bugs are really session-boundary bugs:

- a tab opens in one directory but new panes use another
- the sidebar in one tab affects another tab
- tab naming, Yazi cwd, and editor cwd drift apart
- a command changes the focused pane cwd but not the workspace root, or vice versa

Those bugs happen when the workspace model is implicit. This contract makes the workspace model explicit.

## Actors

There are three relevant actors:

1. Nushell command and integration layer
2. Zellij pane orchestrator plugin
3. Sidebar Yazi state cache

### 1. Nushell Layer

This layer owns user intent and path resolution.

Examples:

- resolve `yzx cwd foo` through `zoxide` or the filesystem
- decide whether a target path should become a directory or repo-root workspace
- decide when editor and sidebar cwd should be synchronized
- generate runtime config before launching the session

The Nushell layer should not guess or duplicate per-tab managed-pane state when the plugin already owns it.

### 2. Pane Orchestrator Plugin

This layer owns authoritative per-tab workspace/session state inside Zellij.

It tracks:

- active tab position
- per-tab workspace root
- whether the workspace root came from bootstrap or explicit user action
- managed editor pane identity
- managed sidebar pane identity
- focus context
- current layout/sidebar-collapsed state

The plugin is the source of truth for managed-pane identity and tab-local workspace state.

### 3. Sidebar Yazi State Cache

The sidebar Yazi process writes its own cache files under Yazelix state.

That cache currently exists because the plugin does not own Yazi instance identity or cwd directly.

The cache is keyed by:

- session
- sidebar pane id

This cache should be treated as an integration cache, not as the main workspace source of truth.

## Ownership Rules

### Nushell Owns

- resolving user input into an explicit target directory
- repo-root derivation for workspace targeting
- when to call workspace-mutating plugin commands
- synchronization of external tools after workspace changes
- generated config and pre-launch setup

Concretely, Nushell owns logic like:

- `get_workspace_root`
- `resolve_tab_cwd_target`
- `sync_active_sidebar_yazi_to_directory`
- `sync_managed_editor_cwd`

### Plugin Owns

- tab-local workspace root once set
- whether that root is `bootstrap` or `explicit`
- tab naming derived from explicit workspace changes
- managed pane discovery by title (`editor`, `sidebar`)
- focus transitions between managed panes
- layout-family and sidebar-open/closed operations
- opening workspace terminals from the stored workspace root

Concretely, the plugin owns state surfaced by `debug_editor_state`.

### Yazi Cache Owns

- sidebar Yazi instance id
- last observed sidebar cwd/path for the active sidebar pane

This is enough to target the current sidebar instance, but it should not become the owner of the workspace model itself.

## Current State Model

### Workspace Root

Workspace root is stored per tab in the plugin.

It has two sources:

- `bootstrap`
- `explicit`

Current behavior:

- the plugin initializes new tabs from a bootstrap root, currently `HOME`
- explicit workspace commands replace that per-tab root
- callers that care about the bootstrap-vs-explicit distinction should inspect the plugin state directly rather than rely on a filtered helper export

Implication:

- bootstrap state is a startup convenience, not a strong user intent signal
- explicit state is the real workspace contract

### Managed Panes

Managed panes are discovered by stable pane titles:

- `editor`
- `sidebar`

The plugin is authoritative for which pane currently counts as the managed editor or managed sidebar in the active tab.

If managed pane discovery fails, that is a contract failure and should surface clearly.

### Focus Context

The plugin tracks whether focus is currently in:

- editor
- sidebar
- other

This is what powers actions like managed focus toggling. Nushell should not reimplement this with pane scanning heuristics.

## Command Contract

### `retarget_workspace`

Input:

- explicit workspace root from Nushell
- whether the currently focused terminal pane should also receive `cd`
- optional managed editor kind to sync against the active tab

Plugin responsibilities:

- store the per-tab workspace root as `explicit`
- rename the tab from that root
- optionally `cd` the currently focused pane
- optionally `cd` the managed editor pane if it exists and the editor kind is supported
- return the active tab's current sidebar Yazi identity so Nushell can emit sidebar adapter commands without re-deriving target pane truth

Plugin does not:

- resolve `zoxide`
- infer repo roots
- execute `ya emit-to` itself

Nushell still owns path resolution and the actual Yazi adapter commands, but it should consume the plugin's retarget response instead of recomputing active editor/sidebar targeting through separate state lookups.

### `open_file`

Input:

- editor kind
- file path
- working directory

Contract:

- Nushell decides the working directory and file target
- the plugin routes to the managed editor pane if present
- the plugin should not invent its own project-root logic here

### `set_managed_editor_cwd`

Input:

- editor kind
- explicit directory

Contract:

- Nushell decides when this sync is needed
- the plugin applies it to the managed editor pane

### `open_workspace_terminal`

Contract:

- the plugin opens a terminal using the stored workspace root for the active tab
- if no explicit per-tab state exists yet, bootstrap state is used

## Invariants

These are the important invariants the system should preserve.

### 1. Workspace State Is Tab-Local

Changing one tab's workspace root must not retarget another tab.

### 2. Managed Pane Identity Is Plugin-Owned

Nushell can request managed editor/sidebar actions, but it should not become the authority on which pane is "the editor" or "the sidebar".

### 3. Explicit Workspace Changes Are Stronger Than Bootstrap

Once a tab has an explicit workspace root, that is the authoritative root for:

- tab naming
- new managed panes
- workspace terminal actions
- editor/sidebar sync requests

### 4. Sidebar Targeting Must Use Current-Tab Identity

Sidebar synchronization must target the active tab's sidebar pane, not a session-global "most recent" cache entry.

### 5. External Sync Is A Follow-On Step

Changing the tab workspace root and synchronizing Yazi/editor cwd are related but separate steps.

The workspace root is the core state mutation.
Editor/Yazi sync is adapter behavior layered on top.

## Known Boundary Gaps

These are honest gaps in the current design.

### Bootstrap Policy Is Still Product Policy

The plugin currently seeds new tabs from `HOME` bootstrap state.
That is coherent, but it is still a product decision, not just an implementation detail.

### Sidebar Cache Is Telemetry, Not Truth

The Yazi sidebar plugin may still write cache files under `~/.local/share/yazelix/state/yazi/sidebar`, but active-tab sidebar targeting now comes from pane-orchestrator state keyed by the managed sidebar pane id.

That cache is for debugging or external inspection only. Active-tab reveal, refresh, and sidebar sync must not depend on scanning it for correctness.

### Workspace Root Naming Is Based On Explicit Updates

Tab naming is aligned with explicit workspace changes, but bootstrap behavior remains special-cased at startup.

## Practical Rule For Future Changes

When adding or changing a workspace feature, answer these questions first:

1. Is this changing workspace root, pane identity, focus context, or external-tool sync?
2. Which subsystem should own that state?
3. Is the change a plugin state mutation, a Nushell intent-resolution change, or an integration-layer sync?
4. Which invariant from this contract is being relied on?

If the answer is unclear, the feature is probably crossing the boundary incorrectly.
