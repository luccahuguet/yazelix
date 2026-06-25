# Status Bar Ownership

## Summary

Yazelix status-bar ownership is split across zjstatus, the standalone `yazelix_zellij_bar` child repo, the integrated Yazelix runtime adapter, and the pane orchestrator.

The supported boundary is runnable-standalone-first for every non-workspace widget. `yazelix_zellij_bar` owns renderers, stdout widget commands, cache schemas, cache locking/backoff, provider probing, CPU/RAM commands, standalone KDL presets, integrated runtime KDL template rendering, and explicit path/env handling for widgets that do not require Yazelix session state. Yazelix owns session-specific path selection, generated layout integration, and workspace facts.

## Ownership Matrix

| Surface | Owner | Status |
| --- | --- | --- |
| zjstatus plugin runtime, layout keys, style tags, command widget intervals, and placeholder expansion | upstream zjstatus plus Yazelix generated KDL | Keep native |
| generic `mode`, `tabs`, `session`, `datetime`, brand, tab-label, compact/full bar, and command-placeholder rendering | `yazelix_zellij_bar` child package command surface | Keep child |
| standalone preset/template packaging and package-local `zjstatus.wasm` path substitution | `yazelix_zellij_bar` child repo | Keep child |
| widget tray token validation and generic dynamic placeholders such as `{pipe_workspace}` and bar-owned command placeholders | `yazelix_zellij_bar` child package command surface | Keep child |
| integrated plugin block, including workspace pipe format, Claude, Codex, OpenCode Go, CPU, RAM, and version command definitions | `yazelix_zellij_bar_widget render-yazelix-runtime` rendered from the child runtime KDL template plus Yazelix-supplied typed config | Keep child |
| status-bus schema decode and inspect-session rendering | Yazelix core plus pane-orchestrator producer | Keep adapter |
| window-local `status_bar_cache.json` writes, heartbeat merges, and cache path discovery | Yazelix core | Keep adapter |
| provider usage display models, summary formatting, quota/tokens modes, cached-fact widget rendering, cache schemas, locking, freshness/backoff, provider probing, and standalone stdout commands | `yazelix_zellij_bar` child repo | Move child |
| launch-scoped provider cache path selection and session widget settings | Yazelix core status adapter | Keep adapter |
| CPU/RAM command widgets | `yazelix_zellij_bar` child repo | Move child |
| live sidebar/editor/workspace facts | pane orchestrator | Keep producer |
| active-tab workspace pipe message and label content | pane orchestrator | Keep producer |
| all-tab activity facts | pane orchestrator snapshot written through the window-local status-bar cache; `get_all_tab_activity_state` remains the direct diagnostic/read seam | Keep producer |
| activity tab-label presentation | native Zellij tab-name mutation as the current bridge; `yazelix_zellij_bar` owns pure label rendering and diagnostic cache rendering | Keep bridge |
| terminal-bell tab presentation | upstream zjstatus `{tabs}` with child-owned generated `tab_normal_bell` and `tab_normal_flashing_bell` style-only formats | Keep native |
| direct `status-bus-workspace` zjstatus command | none | Deleted |

## Contract Items

#### SBO-001
- Type: boundary
- Status: live
- Owner: `yazelix_zellij_bar` child repo
- Statement: Generic bar rendering, tab label rendering, widget-tray token validation, compact/full bar policy, simple fact widgets, runnable non-workspace widget commands, integrated runtime-template rendering, and standalone preset/template packaging belong to `yazelix_zellij_bar`. The main repo consumes the child package command surface as a typed config to rendered plugin-block boundary rather than linking the child crate or maintaining parallel widget implementations
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core zellij_materialization`

#### SBO-002
- Type: behavior
- Status: live
- Owner: Yazelix core status adapter
- Statement: Integrated non-workspace dynamic widgets render from window-local cached facts or child-owned commands. The active-tab workspace widget is the exception: it is pushed by the pane orchestrator into the active tab's `pipe_workspace` widget so async command results cannot display the previously focused tab
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
- Statement: The old direct `status-bus-workspace` command is not part of the supported status-bar path. Generated zjstatus templates must use the child-owned `pipe_workspace` widget for the active-tab workspace label and cache-widget or child command paths for the remaining dynamic widgets
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core substitutes_child_zjstatus_plugin_block`

#### SBO-005
- Type: boundary
- Status: live
- Owner: `yazelix_zellij_bar` child repo plus Yazelix generated Zellij materialization
- Statement: Integrated zjstatus command definitions are rendered by `yazelix_zellij_bar_widget render-yazelix-runtime` from the child runtime KDL template plus typed Yazelix runtime bar config. The child command owns the zjstatus plugin block: widget command names, intervals, formats, render modes, widget tray output, custom text, and tab labels. The main adapter supplies runtime paths and inserts the returned plugin block into generated layouts
- Verification: automated `cargo test` in `luccahuguet/yazelix-zellij-bar` and `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core zellij_materialization`

#### SBO-006
- Type: boundary
- Status: live
- Owner: `yazelix-cursors`, terminal materialization, and ratconfig-backed configuration UI
- Statement: Cursor configuration, effects, and preset inspection are not status-bar widget surfaces. The status bar does not render cursor preset names, swatches, glyphs, or `yzc current` output. `yazelix-cursors` remains the owner of cursor schemes and assets, while terminal materialization applies them and ratconfig exposes inspection/editing.
- Verification: automated `cargo test` in `luccahuguet/yazelix-zellij-bar` and `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_zellij_config_pack`

#### SBO-007
- Type: boundary
- Status: live
- Owner: `yazelix_zellij_bar` child repo plus Yazelix core status adapter
- Statement: Claude, Codex, and OpenCode Go widget implementation belongs to `yazelix_zellij_bar`: display rendering, standalone stdout commands, cache schemas, cache locking, freshness/backoff, provider probing, and explicit cache/database path handling. Yazelix core may choose session-specific paths and widget settings, but must not own the provider widget implementation
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache`

#### SBO-007A
- Type: behavior
- Status: live
- Owner: Yazelix generated Zellij materialization plus pane orchestrator
- Statement: Integrated agent-usage provider refreshes are selected by the active
  `zellij.widget_tray`. A provider that is absent from the rendered tray must
  not be scheduled by the pane orchestrator, while enabled providers continue to
  render from child-owned cached facts with freshness and error backoff. CPU/RAM
  widgets remain cheap cached child commands, and adding new status extras must
  not turn the status bar into an always-measure-everything sampler
- Verification: automated
  `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_zellij_config_pack`; automated
  `cargo test --manifest-path ../yazelix-zellij-pane-orchestrator/Cargo.toml --lib`

#### SBO-008
- Type: boundary
- Status: live
- Owner: Yazelix core status adapter
- Statement: Workspace display remains Yazelix-owned until a separate contract defines a generic standalone fallback. It depends on pane-orchestrator live tab/pane facts, is pushed to the active tab's zjstatus plugin through the `workspace` pipe, and has no standalone non-Yazelix source of truth in SP9
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_bus`

#### SBO-009
- Type: boundary
- Status: live
- Owner: `yazelix_zellij_bar` child repo
- Statement: A non-workspace widget is not standalone unless a non-Yazelix user can run it through a `yazelix_zellij_bar_widget` command or equivalent child-owned API without `yzx`, `yzx_control`, `~/.config/yazelix`, `~/.local/share/yazelix`, pane-orchestrator state, or Yazelix launch-scoped cache paths
- Verification: automated `cargo test` in `luccahuguet/yazelix-zellij-bar`

#### SBO-010
- Type: boundary
- Status: live
- Owner: pane orchestrator plus pinned zjstatus `{tabs}` widget
- Statement: Activity tab markers are rendered from facts, not native tab-name
  mutations. The pane orchestrator owns registered activity facts and recognized
  spinner-prefixed terminal titles, reduces them to alert, busy, or idle, then
  publishes the versioned all-tab activity snapshot to every known zjstatus
  plugin instance through `pipe_tab_activity`. Stale takes priority over busy,
  and busy takes priority over no marker. Spinner-prefixed terminal-title
  activity is active only while the title still has the spinner prefix; when the
  title no longer exposes activity, the fact is removed without waiting for pane
  focus. The pinned zjstatus tabs widget owns marker rendering, appends markers
  to the live Zellij `TabInfo.name`, and must display the raw native tab name
  during Zellij tab rename mode
- Verification: automated
  `cargo test --manifest-path ../yazelix-zellij-pane-orchestrator/Cargo.toml --lib`
  and `cargo test --target x86_64-unknown-linux-gnu` in the pinned zjstatus fork

#### SBO-011
- Type: boundary
- Status: live
- Owner: pane orchestrator plus `yazelix_zellij_bar` child repo
- Statement: The durable activity-bar direction is facts first, rendering
  second. The pane orchestrator produces a versioned all-tabs JSON snapshot
  containing tab id, tab position, clean base tab name, active-tab flag,
  fullscreen/sync/floating indicators, reduced `idle` / `busy` / `alert`
  activity state, and underlying activity facts. `yzx_control zellij
  status-cache-write` stores that snapshot under `tab_activity` in the same
  launch-scoped `status_bar_cache.json` used by other Yazelix bar widgets,
  while preserving heartbeat facts and the previous tab-activity snapshot when
  a cache write omits a new tab-activity payload. The pane orchestrator also
  pushes the same snapshot over `pipe_tab_activity` so the active zjstatus
  instance does not wait for command-widget polling. The integrated child runtime
  template keeps zjstatus `{tabs}` as the live tab source, because that path is
  driven by Zellij `TabUpdate` events and owns correct focus, creation, deletion,
  click, and truncation behavior. The generated `{tabs}` formats also use
  upstream zjstatus bell fields for style-only terminal-BEL presentation, which
  is separate from Yazelix AI-activity facts and does not add Yazelix activity
  marker text.
  `yazelix_zellij_bar_widget tabs` remains a
  child-owned renderer probe for the all-tab activity snapshot contract, but it
  is not the default integrated tab strip. Yazelix's pinned zjstatus consumes
  the snapshot activity state by tab id in the native `{tabs}` path through
  `tab_activity_pipe_name`, while the displayed tab label still comes from live
  Zellij `TabInfo.name`; the runtime gets activity markers without renaming tabs
- Verification: automated
  `cargo test` in `luccahuguet/yazelix-zellij-bar` and
  `cargo test --manifest-path ../yazelix-zellij-pane-orchestrator/Cargo.toml --lib`
  plus `cargo test --target x86_64-unknown-linux-gnu` in the pinned zjstatus fork
  and `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache`

## Deletion And Extraction Plan

Delete-first order:

1. Delete direct pane-orchestrator-per-paint widget commands that the generated templates no longer use
2. Delete or demote weak tests that defend old command names instead of current cache behavior
3. Move runnable non-workspace widget commands to `yazelix_zellij_bar`: CPU, RAM, Claude, Codex, and OpenCode Go
4. Delete provider usage refreshers, cache schemas, lock/backoff implementation, and CPU/RAM scripts from Yazelix once the child command surface exists
5. Keep pane-orchestrator facts, workspace rendering, generated layout integration, session-specific path selection, and runtime path discovery in the integrated runtime unless a future contract defines a reusable status bus

Do not move pane-orchestrator payloads, Home Manager apply semantics, or Yazelix session facts into `yazelix_zellij_bar` or `yazelix-cursors`. Do move non-workspace widget polling/probing/cache behavior when it can be parameterized by explicit paths/env and run outside Yazelix.

## Verification

- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core zellij_materialization`
- `yzx_repo_validator validate-contracts`

## Traceability

- Defended by: `yzx_repo_validator validate-contracts`
