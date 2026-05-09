# Contracts Inventory

## Summary

This inventory lists the canonical contracts under `docs/contracts/`. It is not a planning backlog and does not classify historical or evaluation files because those files do not belong in the canonical contracts surface.

## Canonical Contracts

| Contract | Owns |
| --- | --- |
| `backend_free_workspace_slice.md` | Backend-free workspace behavior that remains credible inside the integrated product |
| `canonical_contract_item_schema.md` | Indexed contract-item format and test traceability rules |
| `config_runtime_control_plane_contract.md` | Config/runtime/control-plane ownership and bridge boundaries |
| `contracts_inventory.md` | Canonical contract list and inventory hygiene |
| `cross_language_runtime_ownership.md` | Current language/runtime owner map |
| `desktop_launch_visible_feedback.md` | Desktop launch visible-feedback behavior |
| `first_run_upgrade_summary.md` | First-run and `yzx whats_new` summary behavior |
| `floating_tui_panes.md` | Popup, menu, and transient floating-pane behavior |
| `helix_managed_config_contract.md` | Managed Helix config ownership |
| `keybinding_action_ownership.md` | Semantic keybinding action ownership and native keymap boundaries |
| `macos_support_floor.md` | First-party macOS support floor |
| `nix_customization_surfaces.md` | Supported granular Nix customization surfaces |
| `nixpkgs_package_contract.md` | Nixpkgs-style package shape |
| `native_config_integration_status.md` | Native, managed, imported, read-only, and generated config status vocabulary |
| `nonpersistent_window_session_contract.md` | Default window/session behavior |
| `pane_orchestrator_component.md` | Pane-orchestrator component boundary |
| `pane_orchestrator_tab_local_session_state_seam.md` | Pane-orchestrator tab-local session state boundary |
| `runtime_dependency_preflight_contract.md` | Runtime dependency and launch-preflight behavior |
| `runtime_distribution_capability_tiers.md` | Runtime distribution capability tiers and update ownership |
| `runtime_applied_settings.md` | Settings apply-mode vocabulary and runtime refresh boundary |
| `runtime_root_contract.md` | Runtime root ownership |
| `runtime_shell_floor_contract.md` | Surviving runtime-side shell floor |
| `rust_nextest_harness_boundary.md` | Rust nextest harness boundary |
| `rust_nushell_bridge_contract.md` | Rust/Nushell bridge behavior |
| `shell_opened_editors.md` | Managed-editor versus shell-opened editor boundary |
| `stale_config_diagnostics.md` | Unsupported/stale config diagnostics |
| `standalone_ghostty_cursor_distribution.md` | Standalone `yazelix_ghostty_cursors` package for Ghostty cursor shaders |
| `standalone_yazelix_screen_distribution.md` | Standalone Yazelix screen package and app |
| `startup_profile_scenarios.md` | Structured startup-profile scenarios |
| `standalone_yazelix_zellij_bar_distribution.md` | Standalone `yazelix_zellij_bar` zjstatus preset and package |
| `status_bar_ownership.md` | Status-bar ownership across zjstatus, `yazelix_zellij_bar`, Yazelix core, and pane orchestrator |
| `status_doctor_machine_readable_reports.md` | Machine-readable status and doctor reports |
| `supply_chain_hardening.md` | Supply-chain hardening policy |
| `terminal_launch_contract.md` | Terminal launch/process-boundary behavior |
| `terminal_override_layers.md` | Terminal override layers |
| `test_suite_governance.md` | Governed test lane, strength, and retention rules |
| `toml_tooling_contract.md` | Runtime TOML tooling behavior |
| `upgrade_notes_contract.md` | Structured upgrade notes |
| `v15_trimmed_runtime_contract.md` | Current trimmed runtime branch contract |
| `welcome_screen_style_contract.md` | Welcome and screen style behavior |
| `workspace_session_contract.md` | Workspace tab, sidebar, and session-truth behavior |
| `yazi_integration_boundary.md` | Yazi config pack versus Yazelix editor/sidebar integration boundary |
| `yazelix_zellij_pane_orchestrator_extraction.md` | Standalone Zellij plugin extraction boundary for the pane orchestrator |
| `yazelix_ratconfig_extraction.md` | Config UI extraction boundary for future `yazelix_ratconfig` |
| `yazelix_workspace_extraction.md` | Workspace extraction boundary for future `yazelix_workspace` |
| `yzx_command_palette_categories.md` | Command-palette grouping and exclusion behavior |

## Inventory Rules

- Each row must describe current supported behavior
- Historical notes, evaluations, migration diaries, and implementation plans are excluded from this inventory
- New contracts should be added only after duplicate or weaker docs are deleted or demoted
- Planning records may reference contracts; contracts should not reference planning records

## Verification

- `yzx_repo_validator validate-contracts`
