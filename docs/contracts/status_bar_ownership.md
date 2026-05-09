# Status Bar Ownership

## Summary

Yazelix status-bar ownership is split across zjstatus, the standalone `yazelix_bar` child repo, the integrated Yazelix runtime adapter, and the pane orchestrator.

The supported boundary is cache-first: generated zjstatus command widgets read small window-local cache files through `yzx_control zellij status-cache-widget`. They must not call the pane orchestrator directly on every bar paint.

## Ownership Matrix

| Surface | Owner | Status |
| --- | --- | --- |
| zjstatus plugin runtime, layout keys, style tags, command widget intervals, and placeholder expansion | upstream zjstatus plus Yazelix generated KDL | Keep native |
| generic `mode`, `tabs`, `session`, `datetime`, brand, tab-label, compact/full bar, and command-placeholder rendering | `yazelix_bar` child crate | Keep child |
| standalone preset generation and package-local `zjstatus.wasm` path substitution | `yazelix_bar` child repo | Keep child |
| widget tray token validation and generic dynamic command placeholders such as `{command_workspace}` | `yazelix_bar` child crate | Keep child |
| workspace, cursor, Claude, Codex, OpenCode Go, CPU, RAM, and version command definitions for the integrated template | `yazelix_bar` child crate rendered from Yazelix-supplied paths | Keep child |
| cursor status widget text and glyph display from cursor facts | `yazelix_bar` child crate | Extract from adapter |
| cursor cache path discovery and first-paint hydration from Yazelix session state | Yazelix core status adapter | Keep adapter |
| status-bus schema decode and inspect-session rendering | Yazelix core plus pane-orchestrator producer | Keep adapter |
| window-local `status_bar_cache.json` writes, heartbeat merges, and cache path discovery | Yazelix core | Keep adapter |
| provider usage display models, summary formatting, quota/tokens modes, and cached-fact widget rendering | `yazelix_bar` child crate | Extract from adapter |
| provider usage cache refreshes, shared-cache locking, tokenusage/OpenCode probing, and launch-scoped status-cache hydration | Yazelix core `zellij_commands::status::agent_usage` module | Keep adapter |
| live sidebar/editor/workspace facts | pane orchestrator | Keep producer |
| direct `status-bus-workspace` zjstatus command | none | Deleted |

## Contract Items

#### SBO-001
- Type: boundary
- Status: live
- Owner: `yazelix_bar` child repo
- Statement: Generic bar rendering, tab label rendering, widget-tray token validation, compact/full bar policy, simple fact widgets, and standalone preset generation belong to `yazelix_bar`. The main repo should consume that renderer rather than maintain a parallel generic bar renderer
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core zellij_materialization`

#### SBO-002
- Type: behavior
- Status: live
- Owner: Yazelix core status adapter
- Statement: Integrated dynamic widgets render from window-local cached facts through `yzx_control zellij status-cache-widget`, not by invoking pane-orchestrator pipes directly from every zjstatus command
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache`

#### SBO-003
- Type: boundary
- Status: live
- Owner: pane orchestrator
- Statement: The pane orchestrator owns live tab, pane, sidebar, and workspace facts. The status adapter may decode its versioned status-bus payload, but it must not become a second live workspace-state owner
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_bus`

#### SBO-004
- Type: boundary
- Status: live
- Owner: Yazelix Zellij command surface
- Statement: The old direct `status-bus-workspace` command is not part of the supported status-bar path. Generated zjstatus templates must keep using cache-widget commands for dynamic widgets
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core renders_cached_zjstatus_widget_commands_with_runtime_helper_paths`

#### SBO-005
- Type: boundary
- Status: live
- Owner: Yazelix generated Zellij materialization
- Statement: Integrated zjstatus command definitions are rendered from the typed Yazelix command adapter, not hand-owned by the KDL fragment. The fragment owns zjstatus layout shape and placeholders; the adapter owns runtime helper paths, widget command names, intervals, formats, and render modes
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core renders_cached_zjstatus_widget_commands_with_runtime_helper_paths`

#### SBO-006
- Type: boundary
- Status: live
- Owner: `yazelix_bar` child repo plus Yazelix core status adapter
- Statement: Cursor widget display rendering belongs to `yazelix_bar` when supplied with explicit cursor facts compatible with `yazelix-cursors`. Yazelix core owns only launch-scoped cache path discovery, environment-derived first-paint hydration, and session integration. `yazelix-cursors` remains the owner of cursor schemes, assets, and non-Zellij cursor distribution
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache`

#### SBO-007
- Type: boundary
- Status: live
- Owner: `yazelix_bar` child repo plus Yazelix core status adapter
- Statement: Claude, Codex, and OpenCode Go widget display rendering belongs to `yazelix_bar` when supplied with cached usage facts. Yazelix core owns provider invocation, SQLite/database probing, shared cache paths, locking, freshness/backoff, session config hydration, and `yzx_control zellij status-cache-widget` transport
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache`

#### SBO-008
- Type: boundary
- Status: live
- Owner: Yazelix core status adapter
- Statement: Workspace display remains Yazelix-owned until a separate contract defines a generic standalone fallback. It depends on pane-orchestrator live tab/pane facts and has no standalone non-Yazelix source of truth in SP8
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_bus`

## Deletion And Extraction Plan

Delete-first order:

1. Delete direct pane-orchestrator-per-paint widget commands that the generated templates no longer use
2. Delete or demote weak tests that defend old command names instead of current cache behavior
3. Move standalone display rendering to `yazelix_bar`: simple fact widgets, cursor fact rendering, and cached Claude/Codex/OpenCode Go usage rendering
4. Keep provider usage refreshers in focused Yazelix-core modules before considering any public package
5. Keep pane-orchestrator facts, workspace rendering, cache writes, and runtime path discovery in the integrated runtime unless a future contract defines a reusable status bus

Do not move provider usage polling, pane-orchestrator payloads, cache file paths, Home Manager apply semantics, or Yazelix session facts into `yazelix_bar` or `yazelix-cursors`.

## Verification

- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core zellij_materialization`
- `yzx_repo_validator validate-contracts`

## Traceability

- Defended by: `yzx_repo_validator validate-contracts`
