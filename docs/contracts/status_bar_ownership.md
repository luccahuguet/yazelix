# Status Bar Ownership

## Summary

Yazelix status-bar ownership is split across zjstatus, the standalone `yazelix_bar` child repo, the integrated Yazelix runtime adapter, and the pane orchestrator.

The supported boundary is cache-first: generated zjstatus command widgets read small window-local cache files through `yzx_control zellij status-cache-widget`. They must not call the pane orchestrator directly on every bar paint.

## Ownership Matrix

| Surface | Owner | Status |
| --- | --- | --- |
| zjstatus plugin runtime, layout keys, style tags, command widget intervals, and placeholder expansion | upstream zjstatus plus Yazelix generated KDL | Keep native |
| generic `mode`, `tabs`, `session`, `datetime`, brand, tab-label, and command-placeholder rendering | `yazelix_bar` child crate | Keep child |
| standalone preset generation and package-local `zjstatus.wasm` path substitution | `yazelix_bar` child repo | Keep child |
| widget tray token validation and generic dynamic command placeholders such as `{command_workspace}` | `yazelix_bar` child crate | Keep child |
| workspace, cursor, Claude, Codex, OpenCode Go, CPU, RAM, and version command definitions in the integrated template | Yazelix generated Zellij materialization | Keep adapter |
| status-bus schema decode and inspect-session rendering | Yazelix core plus pane-orchestrator producer | Keep adapter |
| window-local `status_bar_cache.json` writes, heartbeat merges, and cache path discovery | Yazelix core | Keep adapter |
| provider usage cache refreshes and shared-cache locking | Yazelix core | Keep adapter, but split by provider before extraction |
| live sidebar/editor/workspace facts | pane orchestrator | Keep producer |
| direct `status-bus-workspace` zjstatus command | none | Deleted |

## Contract Items

#### SBO-001
- Type: boundary
- Status: live
- Owner: `yazelix_bar` child repo
- Statement: Generic bar rendering, tab label rendering, widget-tray token validation, and standalone preset generation belong to `yazelix_bar`. The main repo should consume that renderer rather than maintain a parallel generic bar renderer
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
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core zjstatus_template_uses_cached_dynamic_widget_helpers`

## Deletion And Extraction Plan

Delete-first order:

1. Delete direct pane-orchestrator-per-paint widget commands that the generated templates no longer use
2. Delete or demote weak tests that defend old command names instead of current cache behavior
3. Move only generic rendering behavior to `yazelix_bar`
4. Split provider usage refreshers into focused Yazelix-core modules before considering any public package
5. Keep pane-orchestrator facts and cache writes in the integrated runtime unless a future contract defines a reusable status bus

Do not move provider usage polling, pane-orchestrator payloads, cache file paths, Home Manager apply semantics, or Yazelix session facts into `yazelix_bar`.

## Verification

- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core status_cache`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core zellij_materialization`
- `yzx_repo_validator validate-contracts`

## Traceability

- Defended by: `yzx_repo_validator validate-contracts`
