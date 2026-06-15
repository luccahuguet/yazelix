# Workspace Session Contract

This document defines the current contract between Yazelix's control layer and the Zellij pane orchestrator plugin.

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

1. Yazelix control and integration layer
2. Zellij pane orchestrator plugin
3. Sidebar Yazi adapter plugin

### 1. Yazelix Control Layer

This layer owns user intent and path resolution.

Examples:

- resolve `yzx cwd foo` through `zoxide` or the filesystem
- decide whether a target path should become a directory or repo-root workspace
- decide when editor and sidebar cwd should be synchronized
- generate runtime config before launching the session

The control layer should not guess or duplicate per-tab managed-pane state when the plugin already owns it.

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

### 3. Sidebar Yazi Adapter Plugin

The sidebar Yazi process reports its live pane id, Yazi instance id, and cwd to the pane orchestrator.

The adapter does not own durable workspace state. It publishes observations, and the pane orchestrator validates them against the current tab's managed sidebar pane before exposing them to `yzx reveal`, sidebar refresh, inspect, and status-bus consumers.

## Contract Items

#### WSS-001
- Type: ownership
- Status: live
- Owner: pane orchestrator active-tab session seam
- Statement: `get_active_tab_session_state` is the stable read seam for
  active-tab workspace/session truth. Yazelix callers must not reconstruct managed
  editor/sidebar identity or tab-local workspace state from separate cache scans
  or pane heuristics
- Verification: automated
  `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core yzx_control_reveal_uses_session_snapshot_and_focuses_sidebar`; automated
  `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core workspace_session::tests`

#### WSS-002
- Type: ownership
- Status: live
- Owner: pane orchestrator workspace/session state plus sidebar Yazi adapter events
- Statement: The pane orchestrator owns tab-local workspace root and managed
  pane identity. Sidebar Yazi identity and cwd come from adapter events stored
  in the pane orchestrator, not from filesystem cache scans or recency guesses
- Verification: automated
  `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core yzx_control_zellij_open_editor_passes_current_yazi_state_to_retarget`; automated
  `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core workspace_session::tests`

#### WSS-003
- Type: boundary
- Status: live
- Owner: `retarget_workspace` plus caller-local follow-on sync
- Statement: `retarget_workspace` is the single live workspace-mutation seam.
  The control layer resolves the path and follow-on Yazi/editor adapter actions, while
  the plugin stores the explicit workspace root, owns tab naming, and returns
  current-tab sidebar identity in the mutation response
- Verification: automated
  `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core yzx_control_cwd_retargets_workspace_and_syncs_sidebar`; automated
  `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core workspace_session::tests`

#### WSS-004
- Type: behavior
- Status: live
- Owner: session snapshot and retarget response consumers
- Statement: Sidebar reveal, refresh, and post-retarget sync flows target the
  current tab's sidebar identity from the session snapshot or retarget response,
  not from a session-global "most recent" cache guess
- Verification: automated
  `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core yzx_control_reveal_uses_session_snapshot_and_focuses_sidebar`; automated
  `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core yzx_control_cwd_retargets_workspace_and_syncs_sidebar`

#### WSS-005
- Type: boundary
- Status: live
- Owner: Rust-owned integration/transient/startup fact helpers plus caller-local
  Nu orchestration
- Statement: Front-door and integration callers consume explicit retained facts
  for sidebar enablement, managed editor kind, Yazi commands, popup geometry,
  and startup/session toggles instead of reparsing the full config in each
  command or wrapper path
- Verification: automated
  `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core session_facts::tests`; automated
  `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core workspace_commands::tests`; automated
  `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_control_workspace_surface`

#### WSS-006
- Type: invariant
- Status: live
- Owner: `yazelix-zellij-config-pack`
- Statement: Built-in Zellij layout family behavior is rendered by the consumed
  config-pack child crate. Main Yazelix must not keep parallel built-in layout
  templates or metadata; generated workspace-state checks derive expected
  layout file names from the child pack.
- Verification: automated
  `yzx_repo_validator validate-workspace-session-contract`; automated
  `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core zellij_materialization::tests::startup_layouts_keep_initial_tab_distinct_from_home_scoped_new_tabs`

#### WSS-007
- Type: behavior
- Status: live
- Owner: `yzx doctor` workspace asset drift checks
- Statement: Missing or stale generated workspace assets, including generated
  Zellij config, generated layouts, and generated plugin wasm artifacts, must
  surface as doctor findings with the generated-state repair action when the
  issue is repairable
- Verification: automated
  `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core workspace_asset_contract::tests`

#### WSS-008
- Type: boundary
- Status: live
- Owner: maintainer session inspection surface
- Statement: Maintainers can inspect the active tab session snapshot through
  `yzx dev inspect_session` without ad hoc plugin pipes, and that output must
  include workspace root/source, focus context, layout state, managed panes, and
  sidebar Yazi identity
- Verification: automated
  `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core zellij_commands::tests::session_inspection_lines_include_workspace_layout_and_sidebar_identity`; automated
  `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core public_command_surface::tests::routes_grouped_rust_family_to_control_plane`

## Ownership Rules

### Control Layer Owns

- resolving user input into an explicit target directory
- repo-root derivation for workspace targeting
- when to call workspace-mutating plugin commands
- synchronization of external tools after workspace changes
- generated config and pre-launch setup

Concretely, the Rust control surface and remaining shell integration own logic like:

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

Concretely, the plugin owns state surfaced by the stable
`get_active_tab_session_state` seam.

`maintainer_debug_editor_state` remains a maintainer-only inspection payload,
not the primary long-term contract for active-tab session truth.

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

- the plugin initializes new tabs from a bootstrap root, currently the plugin's
  `initial_cwd`
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

This is what powers actions like managed focus toggling. Yazelix callers should not reimplement this with pane scanning heuristics.

## Command Contract

### `retarget_workspace`

Input:

- explicit workspace root from the control layer
- whether the currently focused terminal pane should also receive `cd`
- optional managed editor kind to sync against the active tab

Plugin responsibilities:

- store the per-tab workspace root as `explicit`
- rename the tab from that root
- optionally `cd` the currently focused pane
- optionally `cd` the managed editor pane if it exists and the editor kind is supported
- return the active tab's current sidebar Yazi identity so the control layer can emit sidebar adapter commands without re-deriving target pane truth

Plugin does not:

- resolve `zoxide`
- infer repo roots
- execute `ya emit-to` itself

The control layer still owns path resolution and the actual Yazi adapter commands, but it should consume the plugin's retarget response instead of recomputing active editor/sidebar targeting through separate state lookups.

### `open_file`

Input:

- editor kind
- file paths, with `file_path` accepted as a single-file compatibility field
- working directory

Contract:

- the Rust control plane decides the working directory and file targets
- Yazi-to-editor file opens use the resolved editor working directory as the workspace retarget root, so files nested inside a Git repository keep the repository root instead of the file parent
- the plugin routes to the managed editor pane if present
- the plugin should not invent its own project-root logic here

### `set_managed_editor_cwd`

Input:

- editor kind
- explicit directory

Contract:

- the control layer decides when this sync is needed
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

The control layer can request managed editor/sidebar actions, but it should not become the authority on which pane is "the editor" or "the sidebar".

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

The plugin currently seeds new tabs from its `initial_cwd` bootstrap state.
That is coherent, but it is still a product decision, not just an implementation detail.

### Stable Typed Read Surface Now Exists

The pane orchestrator now exposes `get_active_tab_session_state` as the stable,
versioned read seam for active-tab session truth.

That seam carries:

- active tab position
- workspace root plus `bootstrap` vs `explicit` source
- managed editor/sidebar pane identity
- focus context
- layout/sidebar-collapsed state
- validated current-tab sidebar Yazi identity

Yazelix control code and later sidebar/Yazi consumers should prefer this seam over
`maintainer_debug_editor_state` when they need contract-level tab-local truth.

### Sidebar Yazi State Is Plugin Memory, Not Filesystem Truth

The Yazi sidebar plugin does not write a separate sidebar identity cache. Active-tab sidebar targeting comes from pane-orchestrator state keyed by the managed sidebar pane id.

If no current-tab sidebar Yazi state has been registered yet, user-facing commands must fail clearly or retry later rather than falling back to a stale file.

### Workspace Root Naming Is Based On Explicit Updates

Tab naming is aligned with explicit workspace changes, but bootstrap behavior remains special-cased at startup.

## Practical Rule For Future Changes

When adding or changing a workspace feature, answer these questions first:

1. Is this changing workspace root, pane identity, focus context, or external-tool sync?
2. Which subsystem should own that state?
3. Is the change a plugin state mutation, a control-layer intent-resolution change, or an integration-layer sync?
4. Which invariant from this contract is being relied on?

If the answer is unclear, the feature is probably crossing the boundary incorrectly.

## Verification

- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_control_workspace_surface`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core session_facts::tests`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core workspace_commands::tests`
- `cargo test --manifest-path ../yazelix-zellij-pane-orchestrator/Cargo.toml --lib`
- `yzx_repo_validator validate-workspace-session-contract`
- `yzx_repo_validator validate-contracts`

## Traceability
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_control_workspace_surface`
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core session_facts::tests`
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core workspace_commands::tests`
- Defended by: `cargo test --manifest-path ../yazelix-zellij-pane-orchestrator/Cargo.toml --lib`
- Defended by: `yzx_repo_validator validate-workspace-session-contract`

## Open Questions

- Should Yazelix eventually expose a narrower public contract for adopting an
  existing pane as the managed editor or managed sidebar?
- Resolved 2026-04-26: the sidebar identity cache does not survive. The Yazi
  adapter publishes live state into the pane orchestrator, and consumers read
  that plugin-owned state.
