# Integration Glue Canonicalization Audit

## Summary

This audit covers the live glue between Zellij panes, the pane orchestrator,
Yazi, Helix, Neovim, popup/menu wrappers, and workspace retargeting.

The conclusion is not to move integration glue wholesale into Rust. The Rust
pane orchestrator is already the live session-state owner. Nushell still fits
external process adaptation, `ya`/`zellij` command calls, and user-facing
messages. The best deletion lanes are smaller: delete trivial wrapper files
whose only job is to pipe a plugin command, and collapse duplicated config reads
that keep `config_parser.nu` in live integration paths.

## 1. Subsystem Snapshot

- subsystem name: pane, sidebar, Yazi, Zellij, Helix, and editor integration
  glue
- purpose: adapt live external tools to the pane-orchestrator state model,
  retarget workspaces, sync Yazi/sidebar/editor cwd, open transient panes, and
  keep configured editor/Yazi commands honored
- user-visible entrypoints:
  - Yazi file open and open-directory keybindings
  - `yzx cwd`
  - `yzx reveal`
  - `yzx popup`
  - `yzx menu`
  - sidebar layout keybindings
  - zoxide editor plugin path
- primary source paths:
  - `nushell/scripts/integrations/zellij.nu`
  - `nushell/scripts/integrations/yazi.nu`
  - `nushell/scripts/integrations/managed_editor.nu`
  - `nushell/scripts/integrations/zellij_runtime_wrappers.nu`
  - `nushell/scripts/integrations/open_file.nu`
  - `nushell/scripts/integrations/open_dir_in_pane.nu`
  - `nushell/scripts/integrations/zoxide_open_in_editor.nu`
  - `nushell/scripts/zellij_wrappers/*.nu`
  - `rust_plugins/zellij_pane_orchestrator/src/*.rs`
  - `rust_core/yazelix_core/src/workspace_commands.rs`
  - `rust_core/yazelix_core/src/yazi_materialization.rs`
  - `rust_core/yazelix_core/src/zellij_materialization.rs`
- external dependencies that matter:
  - Zellij plugin pipe API
  - Yazi `ya emit-to`
  - configured editor command
  - generated Yazi and Zellij config surfaces

## 2. Must-Not-Lose Behavior

| Behavior | Current contract or source | Current owner | Current verification | Candidate surviving owner |
| --- | --- | --- | --- | --- |
| Pane orchestrator owns active-tab workspace, managed pane identity, sidebar identity, focus context, and Yazi sidebar snapshot | `docs/specs/pane_orchestrator_component.md`; `docs/workspace_session_contract.md` | Rust plugin `zellij_pane_orchestrator` | Rust plugin contract tests; `test_public_yzx_cwd_retargets_workspace_and_syncs_plugin_owned_sidebar`; `test_public_yzx_reveal_uses_session_snapshot_sidebar_state_and_focuses_sidebar` | same |
| `yzx cwd` and `yzx reveal` stay public Rust-owned while still using the pane orchestrator and `ya` for live actions | `docs/specs/v16_rust_cli_rewrite_evaluation.md`; `docs/workspace_session_contract.md` | Rust `workspace_commands.rs` plus plugin and external `ya` | workspace command tests in Nu and Rust | same |
| Opening a Yazi-selected file reuses a managed editor when present, opens a managed pane when missing, retargets workspace state, and syncs sidebar/Yazi state | `docs/workspace_session_contract.md`; Yazi materialization tests | Nu `managed_editor.nu` plus Rust plugin open/set-cwd commands | `test_retarget_workspace_for_path_returns_plugin_owned_sidebar_state_and_editor_status`; `test_sync_post_retarget_workspace_state_handles_missing_editor_and_sidebar_sync`; Yazi materialization tests | Nu remains external adapter; plugin remains live-state owner |
| Configured `yazi_command`, `yazi_ya_command`, `editor_command`, and managed Helix wrapper behavior remain honored | `docs/specs/terminal_override_layers.md`; Helix/Yazi config contracts | Nu `yazi.nu`, `managed_editor.nu`, and `editor_launch_context.nu` | `test_yazi_command_resolvers_honor_defaults_and_overrides`; `test_get_managed_editor_kind_accepts_managed_helix_wrapper_env`; Helix managed config tests | narrower integration facts source plus Nu execution adapters |
| Transient popup/menu identity and wrapper env stay explicit | `docs/specs/floating_tui_panes.md` | Nu `zellij_runtime_wrappers.nu`, popup wrappers, and plugin transient contract | popup tests and plugin transient contract tests | same unless direct plugin actions can delete wrappers |
| Simple layout/sidebar keybindings keep their exact pane-orchestrator behavior | generated Zellij config; pane-orchestrator tests | Nu `zellij_wrappers/next_layout_family.nu`, `previous_layout_family.nu`, `toggle_sidebar_layout.nu`, `open_workspace_terminal.nu` | Zellij materialization tests and plugin command tests | Rust Zellij materialization plus direct pane-orchestrator actions |

## 3. Canonical Owner Map

| Concern | Current owner or split boundary | Split kind | Audit judgment |
| --- | --- | --- | --- |
| Live tab/session/sidebar state | Rust pane orchestrator | intentional | Canonical owner |
| Public workspace commands | Rust `yzx_control` workspace commands | intentional | Canonical owner |
| External `zellij` and `ya` invocation | Nu integration helpers | intentional | External tool adapter |
| Workspace-root derivation for file targets and Git roots | Nu `zellij.nu` plus Rust workspace command logic | mixed | Some duplication, but not a first deletion target |
| Integration config reads | Nu `yazi.nu` and `managed_editor.nu` parse full config | temporary bridge debt | Good bridge-collapse lane |
| Tiny Zellij command wrappers | Nu wrapper files under `zellij_wrappers/` | historical debt | Good delete-wrapper lane |
| Popup/menu/yazi entrypoint wrappers | Nu wrapper files and generated config entrypoints | intentional with bridge debt | Keep until generated config can express a safer direct boundary |

## 4. Survivor Reasons

- Rust pane orchestrator: `canonical_owner`
- Rust workspace commands: `canonical_owner`
- Nu `zellij.nu`: `external_tool_adapter`
- Nu `yazi.nu`: `external_tool_adapter` with `temporary_bridge_debt` for config
  reads
- Nu `managed_editor.nu`: `external_tool_adapter` with `temporary_bridge_debt`
  for config reads and some post-retarget orchestration
- Nu `zellij_runtime_wrappers.nu`: `transport_only`
- Nu `open_file.nu`, `open_dir_in_pane.nu`, `zoxide_open_in_editor.nu`:
  `external_tool_adapter`
- Nu simple Zellij command wrappers: `historical_debt`
- Nu popup/menu wrappers: `external_tool_adapter`

## 5. Delete-First Findings

### Delete Now

- No wrapper deletion should happen in the audit itself.
- The simple Zellij wrapper set is ready for a dedicated delete-wrapper bead.

### Bridge Layer To Collapse

- `yazi.nu` and `managed_editor.nu` still parse the full Yazelix config to
  answer small integration facts:
  - sidebar enabled
  - Yazi command
  - `ya` command
  - configured editor versus managed Helix wrapper
- That keeps integration glue tied to `config_parser.nu` even though the plugin
  already owns live state and Rust owns normalized config semantics.

### Full-Owner Migration

- No broad integration full-owner migration is honest.
- Moving `ya emit-to`, `zellij action`, or editor launching into Rust would
  mostly wrap external process calls, not delete a deterministic owner.

### Likely Survivors

- `zellij.nu` as the plugin pipe adapter
- `yazi.nu` as the Yazi CLI adapter after config reads shrink
- `managed_editor.nu` as the editor/Yazi/process adapter
- popup/menu wrappers until generated config can directly express the same
  identity, geometry, and close/refresh behavior

### No-Go Deletions

- Deleting `yazi.nu`
  - stop condition: it still owns `ya` command execution and user-visible
    sidebar disabled/outside-Zellij messages
- Deleting `managed_editor.nu`
  - stop condition: it still owns configured editor execution, managed pane
    fallback behavior, and no-sidebar Yazi sync
- Folding pane-orchestrator truth into `rust_core`
  - stop condition: live session truth belongs to the plugin, not the public
    CLI helper crate

## 6. Quality Findings

- duplicate owners:
  - sidebar identity previously duplicated in Nu is now narrowed to the plugin
    snapshot, but Nu still reconstructs action context around it
  - integration config facts are parsed repeatedly rather than passed in or
    computed through one owner
- missing layer problems:
  - no explicit integration facts boundary says which config facts integration
    glue may read
  - simple Zellij actions are not separated from wrappers that truly need Nu
    process logic
- extra layer problems:
  - tiny wrappers for next/previous/toggle/open-workspace-terminal add process
    hops over one plugin pipe command
- DRY opportunities:
  - generated Zellij config can likely pipe simple plugin commands directly
  - integration config reads can collapse into one facts source
- weak or orphan tests:
  - integration tests are mostly strong regressions, but many still lack
    concrete indexed contract IDs
- only-known executable-defense tests:
  - plugin tests for workspace, sidebar, transient, focus, and pane contracts
  - `test_public_yzx_cwd_retargets_workspace_and_syncs_plugin_owned_sidebar`
  - `test_public_yzx_reveal_uses_session_snapshot_sidebar_state_and_focuses_sidebar`
  - `test_yazi_command_resolvers_honor_defaults_and_overrides`
  - `test_sync_post_retarget_workspace_state_handles_missing_editor_and_sidebar_sync`
- spec gaps:
  - no indexed contract items for the integration facts boundary
  - workspace/session contracts exist, but many tests still map to broad file
    references rather than item IDs

## 7. Deletion Classes And Follow-Up Beads

| Bead | Retained behavior | Deletion class | Candidate surviving owner | Verification that must still pass | Explicit stop condition |
| --- | --- | --- | --- | --- | --- |
| `yazelix-niec` | simple layout/sidebar/open-workspace keybindings keep identical pane-orchestrator actions | `delete_now` / `bridge_collapse` | Rust Zellij materialization plus pane orchestrator | Zellij materialization tests; plugin command tests | keep wrappers if direct KDL cannot preserve session-local target or error visibility |
| `yazelix-4xf2` | Yazi command overrides, sidebar enabled state, managed editor detection, and sidebar sync | `bridge_collapse` | explicit integration facts or Rust config/control facts; Nu external adapters | Yazi command resolver tests; managed editor tests; workspace/Yazi sync tests | stop if the only result is moving `ya`/`zellij` process calls into Rust wrappers |

## Verification

- manual review of:
  - `nushell/scripts/integrations/*.nu`
  - `nushell/scripts/zellij_wrappers/*.nu`
  - `rust_plugins/zellij_pane_orchestrator/src/*.rs`
  - `rust_core/yazelix_core/src/workspace_commands.rs`
  - generated Yazi/Zellij materialization tests
- `nu nushell/scripts/dev/validate_specs.nu`

## Traceability

- Bead: `yazelix-rdn7.5.4`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
- Informed by: `docs/workspace_session_contract.md`
- Informed by: `docs/specs/pane_orchestrator_component.md`
- Informed by: `docs/specs/floating_tui_panes.md`
