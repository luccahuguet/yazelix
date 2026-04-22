# Spec Inventory

## Summary

This inventory separates live contracts from active planning and from archived
transition notes.

The current failure mode to avoid is treating already-landed migration docs as
if they were still live planning. `yzx_control` already owns `yzx env` and
`yzx run`. `runtime-env.compute` already landed. The current roadmap is no
longer "move env/run off Nushell." The current roadmap is "delete the remaining
Nu bridge owners and full-owner materialization families."

## Current Starting Points

Start from these files first when planning current work:

- `v15_trimmed_runtime_contract.md`
  - the primary live runtime and command-surface contract
- `rust_migration_matrix.md`
  - the primary delete-first roadmap for remaining product-side Nushell removal
- `ranked_nu_deletion_budget.md`
  - the ranked execution budget for the remaining honest Nushell deletion lanes
- `rust_nushell_bridge_contract.md`
  - the current bridge boundary between Nushell and `yzx_core`
- `cross_language_runtime_ownership.md`
  - the current owner map across Nushell, Rust core, Rust plugins, Lua, and
    POSIX shell
- `canonical_contract_item_schema.md`
  - the contract-item ID, status, owner, and verification schema for the
    `yazelix-rdn7` protocol work
- `config_runtime_control_plane_contract_item_pilot.md`
  - the schema pilot that maps one mixed subsystem before protocol docs,
    validators, and broad audits harden around it

## Scope

This inventory classifies the files under `docs/specs/`.

When a row says `Historical`, the file may still be useful design history, but
new implementation work should not start there. When a row says `Planning`, it
is still current enough to guide future work, but it is not a shipped-product
guarantee by itself.

## Removed From Active Planning

The following stale transition specs were removed before the current inventory
because they described deleted public Nushell owners rather than the current
repo. `yazelix-k0f3` re-verified on `2026-04-21` that they are absent from the
tracked tree and should not be recreated as live planning anchors:

- `yzx_env_run_rust_owner_transition.md`
- `yzx_command_surface_backend_coupling.md`

## Behavior

Status labels:

- `Live`: current maintained contract
- `Planning`: useful forward-looking guidance, but not a shipped contract by
  itself
- `Historical`: superseded transition note or pre-trim design history
- `Deletion Candidate`: stale or redundant doc that should be removed or
  rewritten after its retained history is checked
- `Template`: scaffolding only

| Spec | Status | Notes |
| --- | --- | --- |
| `active_config_surface_owner_cut_budget.md` | Planning | Current active-config surface owner-cut budget; useful for remaining config-surface bridge collapse |
| `backend_capability_contract.md` | Historical | Older backend-era framing kept only as design history; current delete-first work should start from the v15 runtime and Rust migration docs instead |
| `backend_free_workspace_slice.md` | Planning | Still useful as a proof-mode boundary for backend-free workspace behavior; not a separately supported product edition |
| `canonical_contract_item_schema.md` | Planning | Current schema decision for indexed contract items, test traceability, and delete-first feature-preservation rules; use it for `yazelix-rdn7` pilot/docs/validator work |
| `config_metadata_centralization_plan.md` | Planning | Still useful for deleting duplicated config metadata across default config, Home Manager, and parser consumers |
| `config_migration_engine.md` | Historical | Superseded; the automatic config migration engine is gone |
| `config_runtime_control_plane_canonicalization_audit.md` | Planning | Completed subsystem audit that feeds the ranked deletion budget; not a live product contract by itself |
| `config_runtime_control_plane_contract_item_pilot.md` | Planning | Pilot mapping for config/runtime/control-plane ownership, bridge debt, and test traceability; use it before broad `yazelix-rdn7` protocol docs or validators harden |
| `config_surface_and_launch_profile_contract.md` | Historical | Pre-trim launch-profile and backend-era contract |
| `config_surface_backend_dependence_matrix.md` | Historical | Pre-trim pack and backend dependence analysis |
| `cross_language_runtime_ownership.md` | Live | Current language/runtime owner map, including landed Rust core helpers and the remaining mixed seams |
| `desktop_launch_visible_feedback.md` | Live | Current desktop-entry visible feedback contract |
| `first_run_upgrade_summary.md` | Live | Current `yzx whats_new` and first-run summary contract |
| `flake_interface_contract.md` | Historical | Earlier installer-first contract, superseded by the current package/runtime shape |
| `floating_tui_panes.md` | Live | Current popup and transient-pane behavior family |
| `full_config_nushell_owner_cut_budget.md` | Historical | Delete-first budget for the now-landed product-side full-config owner cut; keep as decision history rather than active planning |
| `governed_test_traceability_inventory.md` | Planning | Current governed-test inventory for `yazelix-rdn7.4`; feeds test delete/demote/quarantine work |
| `helix_managed_config_contract.md` | Planning | Still useful future Helix ownership contract; not yet the main delete-first migration lane |
| `integration_glue_canonicalization_audit.md` | Planning | Completed integration-glue audit that names wrapper deletion and config-read collapse lanes |
| `launch_bootstrap_rust_migration.md` | Historical | Transition record for the landed `runtime-env.compute` slice and the explicit stop condition for further v15.x launch/bootstrap Rust work |
| `launch_startup_session_canonicalization_audit.md` | Planning | Completed launch/startup/session audit that documents shell-process no-go boundaries and smaller bridge cuts |
| `likely_nushell_survivor_owner_cut_decisions.md` | Planning | Current family-by-family no-go and follow-up decision record for the remaining likely Nushell survivors |
| `macos_support_floor.md` | Live | Current first-party macOS support floor |
| `maintainer_harness_canonicalization_audit.md` | Planning | Completed maintainer/dev/validator harness audit; separates product Nu deletion from harness cleanup |
| `managed_config_migration_transaction_contract.md` | Historical | Superseded with the removed migration and relocation engine |
| `nixpkgs_package_contract.md` | Live | Current package-shape target |
| `nonpersistent_window_session_contract.md` | Live | Current default window and session behavior contract |
| `one_command_install_ux.md` | Historical | Earlier installer-first planning note |
| `open_window_update_transition_contract.md` | Historical | Older transition note for open-window updates |
| `package_runtime_first_user_and_maintainer_ux.md` | Historical | Transition-space note from before v15 dropped older runtime and pack sidecar assumptions |
| `pane_orchestrator_component.md` | Live | Current internal pane-orchestrator component boundary |
| `pane_orchestrator_tab_local_session_state_seam.md` | Planning | Current pane-orchestrator seam proposal; not a shipped contract yet |
| `persistent_window_session_contract.md` | Live | Current persistent-session behavior contract |
| `public_yzx_command_surface_canonicalization_audit.md` | Planning | Completed public command-surface audit; records Clap no-go and the core registry deletion lane |
| `ranked_nu_deletion_budget.md` | Planning | Current ranked deletion queue synthesized from the subsystem audits; use it to choose the next honest Nushell cuts |
| `runtime_activation_state_contract.md` | Historical | Earlier activation model centered on recorded launch profiles and backend-era reuse |
| `runtime_dependency_preflight_contract.md` | Live | Current launch-preflight versus doctor/install-smoke boundary |
| `runtime_distribution_capability_tiers.md` | Live | Current distribution-tier and update-owner model |
| `runtime_env_config_state_shim_collapse_budget.md` | Planning | Current deletion budget for config/state/env shim collapse; useful as bridge-collapse context |
| `runtime_ownership_reduction_matrix.md` | Historical | Pre-trim alternative analysis, not the current branch contract |
| `runtime_root_contract.md` | Live | Current runtime-root ownership contract |
| `rust_owned_test_migration_budget.md` | Planning | Current Nu-to-Rust migration budget for the largest deterministic governed test files |
| `rust_migration_matrix.md` | Planning | Primary remaining roadmap for deleting product-side Nushell owners, especially the bridge layer and materialization families |
| `rust_nushell_bridge_contract.md` | Live | Current bridge contract for `yzx_core` helper insertion behind Nushell-owned surfaces |
| `rust_test_hardening_tools_decision.md` | Planning | Current keep/reject decision for `cargo-nextest`, `cargo-mutants`, and `cargo-fuzz`; use it before adding Rust hardening tooling |
| `setup_shellhook_welcome_terminal_canonicalization_audit.md` | Planning | Completed setup/shellhook/welcome/terminal audit; spawned the front-door and terminal-launch deletion lanes |
| `shell_opened_editors.md` | Live | Current managed-editor versus shell-opened editor boundary |
| `spec_docs_contract_alignment_audit.md` | Planning | Current docs/spec alignment audit; feeds contract-item migration and historical-doc cleanup |
| `spec_inventory.md` | Live | Maintained inventory of specs by current planning status |
| `status_doctor_machine_readable_reports.md` | Live | Current structured-report contract for `yzx status` and `yzx doctor` |
| `stale_config_diagnostics.md` | Live | Current unsupported-config diagnostic contract without a migration engine |
| `startup_profile_scenarios.md` | Live | Current structured startup-profile scenario contract |
| `supply_chain_hardening.md` | Planning | Current maintainer policy guidance for shipped and documented tools |
| `template.md` | Template | Spec scaffold, excluded from validation |
| `terminal_launch_contract.md` | Live | Current retained terminal launch/process-boundary contract and deletion budget |
| `terminal_override_layers.md` | Live | Current terminal preference and override guidance |
| `test_suite_governance.md` | Live | Current test-strength and lane policy |
| `upgrade_notes_contract.md` | Live | Current structured upgrade-notes contract |
| `v14_boundary_hardening_gate.md` | Historical | v14 release-gate history |
| `v15_trimmed_runtime_contract.md` | Live | Primary branch-level contract; start here for runtime, config, and update questions |
| `v16_rust_cli_rewrite_evaluation.md` | Planning | Secondary planning note for any broader Rust public-CLI move, only after bridge and materialization deletion already shrank the remaining Nu owners |
| `welcome_screen_style_contract.md` | Live | Current indexed welcome and screen style contract |
| `workspace_session_contract.md` | Live | Current tab-local workspace, sidebar identity, and session-truth contract |
| `yazelix_core_boundary.md` | Planning | Future product-boundary decision; there is no separate supported Core edition today |
| `yzx_command_palette_categories.md` | Live | Current command-palette grouping and exclusion contract |

## Non-goals

- deleting useful design history just because it is historical
- treating planning specs as release promises
- reviving deleted migration, launch-profile, or backend-era surfaces just to
  make older docs true again

## Acceptance Cases

1. A maintainer can tell which specs are live before implementing against them
2. Already-landed transition docs no longer masquerade as live planning
3. The current runtime contract points at `v15_trimmed_runtime_contract.md`
   first
4. The current Rust roadmap points at remaining owners rather than deleted
   `yzx env.nu` and `yzx run.nu` wrappers

## Verification

- `nu nushell/scripts/dev/validate_specs.nu`
- manual review of all files listed under `docs/specs/`

## Traceability

- Bead: `yazelix-a3x1`
- Follow-up cleanup: `yazelix-k0f3`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`

## Open Questions

- Revisit this inventory after the next real owner deletion, not after every
  helper insertion or bridge reshuffle
