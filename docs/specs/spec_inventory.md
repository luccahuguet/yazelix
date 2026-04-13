# Spec Inventory

## Summary

This inventory separates current v15 contracts from historical planning notes so old Classic-era specs stop masquerading as live branch behavior.

## Why

Yazelix kept useful design history while the product shape changed quickly. After the v15 trim, several specs still mention removed surfaces such as runtime-local `devenv`, `yazelix_packs.toml`, cached launch-profile reuse, installer-primary flows, and automatic config migrations. Keeping those files is fine; treating all of them as current contracts is not.

## Scope

This file classifies the specs under `docs/specs/`. It does not restate each contract. When a row says historical, the file can still be useful design history, but new implementation work should start from a live v15 contract first.

## Behavior

Status labels:

- `Live`: current v15 contract.
- `Planning`: useful forward-looking guidance, but not a shipped-product guarantee by itself.
- `Historical`: superseded Classic-era or pre-trim design history.
- `Template`: scaffolding only.

| Spec | Status | Notes |
| --- | --- | --- |
| `backend_capability_contract.md` | Planning | Useful backend-boundary framing, but older cached-profile assumptions should not override `v15_trimmed_runtime_contract.md`. |
| `backend_free_workspace_slice.md` | Planning | Useful proof slice for future narrowing; not a separately supported product edition. |
| `config_metadata_centralization_plan.md` | Planning | Still useful for future config metadata consolidation. |
| `config_migration_engine.md` | Historical | Superseded by v15 config-surface rebirth; automatic config migrations are gone. |
| `config_surface_and_launch_profile_contract.md` | Historical | Pre-trim launch-profile and `devenv` ownership contract. |
| `config_surface_backend_dependence_matrix.md` | Historical | Pre-trim pack/backend dependence matrix. |
| `cross_language_runtime_ownership.md` | Live | Current ownership map, with Rust treated as later v15.x/v16 work rather than v15.0 scope. |
| `first_run_upgrade_summary.md` | Live | Current `yzx whats_new` / first-run summary contract, now without live migration-registry probing. |
| `flake_interface_contract.md` | Historical | Earlier installer/front-door phase; package-first v15 contract lives elsewhere. |
| `floating_tui_panes.md` | Live | Current popup/menu transient-pane behavior family. |
| `helix_managed_config_contract.md` | Planning | Useful future Helix config ownership contract; not the main v15 trim gate. |
| `managed_config_migration_transaction_contract.md` | Historical | Superseded with the removed migration/legacy relocation engine. |
| `nixpkgs_package_contract.md` | Live | Current package-shape target. |
| `nonpersistent_window_session_contract.md` | Live | Current default window/session behavior contract. |
| `one_command_install_ux.md` | Historical | Earlier installer-first planning note. |
| `open_window_update_transition_contract.md` | Historical | Older `yzx refresh` and runtime replacement transition model. |
| `package_runtime_first_user_and_maintainer_ux.md` | Historical | Transition-space note before v15 dropped pack sidecars and runtime-local `devenv`. |
| `pane_orchestrator_component.md` | Live | Current internal Zellij plugin component boundary for pipe commands, plugin config keys, runtime wrapper assumptions, and pane identity invariants. |
| `persistent_window_session_contract.md` | Live | Current persistent-session behavior contract. |
| `runtime_activation_state_contract.md` | Historical | Earlier activation model centered on recorded launch profiles and `devenv`. |
| `runtime_dependency_preflight_contract.md` | Live | Current launch-preflight versus doctor/install-smoke boundary. |
| `runtime_distribution_capability_tiers.md` | Live | Current explicit update-owner and distribution-tier model. |
| `runtime_ownership_reduction_matrix.md` | Historical | Pre-trim alternative analysis, not the current contract. |
| `runtime_root_contract.md` | Live | Current root ownership contract. |
| `shell_opened_editors.md` | Live | Current managed-editor versus shell-opened editor boundary. |
| `stale_config_diagnostics.md` | Live | Current unsupported-config diagnostic contract with no migration engine. |
| `supply_chain_hardening.md` | Planning | Useful maintainer policy guidance for shipped/documented tools. |
| `template.md` | Template | Spec scaffold, excluded from validation. |
| `terminal_override_layers.md` | Live | Current terminal preference/override guidance, with Ghostty first-party in the runtime. |
| `test_suite_governance.md` | Live | Current test strength and lane policy. |
| `upgrade_notes_contract.md` | Live | Current structured upgrade notes contract without live migration-id registry enforcement. |
| `v14_boundary_hardening_gate.md` | Historical | v14 release-gate design history. |
| `v15_trimmed_runtime_contract.md` | Live | Primary branch-level v15 contract. Start here for runtime/config/update questions. |
| `yazelix_core_boundary.md` | Planning | Future boundary decision; no separate Core product is supported now. |
| `yzx_command_palette_categories.md` | Live | Current command-palette catalog/category contract. |
| `yzx_command_surface_backend_coupling.md` | Live | Current command-surface coupling map. |

## Non-goals

- Deleting useful history just because it is historical.
- Treating planning specs as release promises.
- Reintroducing removed migration, pack, launch-profile, or runtime-local `devenv` surfaces to make older specs true again.

## Acceptance Cases

1. A maintainer can tell which specs are live before implementing against them.
2. Migration-era and launch-profile-era specs are explicitly historical.
3. The current v15 contract points to `v15_trimmed_runtime_contract.md` first.
4. Future Rust work is framed as later v15.x/v16 work, not a v15.0 prerequisite.

## Verification

- `nu nushell/scripts/dev/validate_specs.nu`
- Manual review of all files listed under `docs/specs/`

## Traceability

- Bead: `yazelix-a3x1`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`

## Open Questions

- The inventory should be revisited after the next large trim or after Rust becomes a live implementation surface.
