# Status Bar Ownership

## Summary

Yazelix status-bar ownership is split across zjstatus, the standalone `yazelix_zellij_bar` child repo, the integrated Yazelix runtime adapter, and the pane orchestrator.

The supported boundary is runnable-standalone-first for every non-workspace widget. `yazelix_zellij_bar` owns renderers, stdout widget commands, cache schemas, cache locking/backoff, provider probing, CPU/RAM commands, integrated zjstatus replacement rendering, and explicit path/env handling for widgets that do not require Yazelix session state. Yazelix owns session-specific path selection, generated layout integration, and workspace facts.

## Ownership Matrix

| Surface | Owner | Status |
| --- | --- | --- |
| zjstatus plugin runtime, layout keys, style tags, command widget intervals, and placeholder expansion | upstream zjstatus plus Yazelix generated KDL | Keep native |
| generic `mode`, `tabs`, `session`, `datetime`, brand, tab-label, compact/full bar, and command-placeholder rendering | `yazelix_zellij_bar` child package command surface | Keep child |
| standalone preset/template packaging and package-local `zjstatus.wasm` path substitution | `yazelix_zellij_bar` child repo | Keep child |
| widget tray token validation and generic dynamic command placeholders such as `{command_workspace}` | `yazelix_zellij_bar` child package command surface | Keep child |
| workspace, cursor, Claude, Codex, OpenCode Go, CPU, RAM, and version command definitions for the integrated template | `yazelix_zellij_bar_widget render-yazelix-runtime` rendered from Yazelix-supplied paths | Keep child |
| cursor status widget text, glyph display, env reading, `yzc current` fallback, and standalone stdout command | `yazelix_zellij_bar` child repo plus `yazelix-ghostty-cursors` facts API | Move child |
| cursor cache path discovery and first-paint hydration from Yazelix session state | Yazelix core status adapter | Keep adapter |
| status-bus schema decode and inspect-session rendering | Yazelix core plus pane-orchestrator producer | Keep adapter |
| window-local `status_bar_cache.json` writes, heartbeat merges, and cache path discovery | Yazelix core | Keep adapter |
| provider usage display models, summary formatting, quota/tokens modes, cached-fact widget rendering, cache schemas, locking, freshness/backoff, provider probing, and standalone stdout commands | `yazelix_zellij_bar` child repo | Move child |
| launch-scoped provider cache path selection and session widget settings | Yazelix core status adapter | Keep adapter |
| CPU/RAM command widgets | `yazelix_zellij_bar` child repo | Move child |
| live sidebar/editor/workspace facts | pane orchestrator | Keep producer |
| direct `status-bus-workspace` zjstatus command | none | Deleted |

## Contract Items

#### SBO-001
- Type: boundary
- Status: live
- Owner: `yazelix_zellij_bar` child repo
- Statement: Generic bar rendering, tab label rendering, widget-tray token validation, compact/full bar policy, simple fact widgets, runnable non-workspace widget commands, and standalone preset/template packaging belong to `yazelix_zellij_bar`. The main repo consumes the child package command surface as data-driven placeholder replacements rather than linking the child crate or maintaining parallel widget implementations
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
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core substitutes_child_zjstatus_command_definitions`

#### SBO-005
- Type: boundary
- Status: live
- Owner: `yazelix_zellij_bar` child repo plus Yazelix generated Zellij materialization
- Statement: Integrated zjstatus command definitions are rendered by `yazelix_zellij_bar_widget render-yazelix-runtime` from a typed Yazelix request. The fragment owns zjstatus layout shape and placeholders; the child command owns widget command names, intervals, formats, render modes, widget tray output, custom text, and tab labels. The main adapter supplies runtime paths and applies the returned replacement map
- Verification: automated `cargo test` in `luccahuguet/yazelix-zellij-bar` and `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core zellij_materialization`

#### SBO-006
- Type: boundary
- Status: live
- Owner: `yazelix_zellij_bar` child repo plus Yazelix core status adapter
- Statement: Cursor widget implementation belongs to `yazelix_zellij_bar` when supplied with cursor facts compatible with `yazelix-ghostty-cursors`. This includes display rendering, env reading, automatic `yzc current --format env` fallback, and a standalone stdout command. Yazelix core owns only launch-scoped environment-derived first-paint hydration and session integration. `yazelix-ghostty-cursors` remains the owner of cursor schemes, assets, and non-Zellij cursor distribution
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache`

#### SBO-007
- Type: boundary
- Status: live
- Owner: `yazelix_zellij_bar` child repo plus Yazelix core status adapter
- Statement: Claude, Codex, and OpenCode Go widget implementation belongs to `yazelix_zellij_bar`: display rendering, standalone stdout commands, cache schemas, cache locking, freshness/backoff, provider probing, and explicit cache/database path handling. Yazelix core may choose session-specific paths and widget settings, but must not own the provider widget implementation
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache`

#### SBO-008
- Type: boundary
- Status: live
- Owner: Yazelix core status adapter
- Statement: Workspace display remains Yazelix-owned until a separate contract defines a generic standalone fallback. It depends on pane-orchestrator live tab/pane facts and has no standalone non-Yazelix source of truth in SP9
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_bus`

#### SBO-009
- Type: boundary
- Status: live
- Owner: `yazelix_zellij_bar` child repo
- Statement: A non-workspace widget is not standalone unless a non-Yazelix user can run it through a `yazelix_zellij_bar_widget` command or equivalent child-owned API without `yzx`, `yzx_control`, `~/.config/yazelix`, `~/.local/share/yazelix`, pane-orchestrator state, or Yazelix launch-scoped cache paths
- Verification: automated `cargo test` in `luccahuguet/yazelix-zellij-bar`

## Deletion And Extraction Plan

Delete-first order:

1. Delete direct pane-orchestrator-per-paint widget commands that the generated templates no longer use
2. Delete or demote weak tests that defend old command names instead of current cache behavior
3. Move runnable non-workspace widget commands to `yazelix_zellij_bar`: CPU, RAM, cursor, Claude, Codex, and OpenCode Go
4. Delete provider usage refreshers, cache schemas, lock/backoff implementation, and CPU/RAM scripts from Yazelix once the child command surface exists
5. Keep pane-orchestrator facts, workspace rendering, generated layout integration, session-specific path selection, and runtime path discovery in the integrated runtime unless a future contract defines a reusable status bus

Do not move pane-orchestrator payloads, Home Manager apply semantics, or Yazelix session facts into `yazelix_zellij_bar` or `yazelix-ghostty-cursors`. Do move non-workspace widget polling/probing/cache behavior when it can be parameterized by explicit paths/env and run outside Yazelix.

## Verification

- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core zellij_materialization`
- `yzx_repo_validator validate-contracts`

## Traceability

- Defended by: `yzx_repo_validator validate-contracts`
