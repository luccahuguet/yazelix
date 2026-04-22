# Rust-Owned Test Migration Budget

## Summary

This document defines the next delete-first budget for moving large deterministic
Nushell test surfaces onto Rust-owned tests without losing the product
contracts those tests currently defend.

The final governed-test end state is `0` surviving Nu test LOC. The migration
rule is selective:

- port strong tests that defend real contracts, regressions, or invariants onto
  Rust-owned nextest suites
- delete weak, duplicated, or trivia-heavy tests instead of porting them
- treat shell-heavy but strong Nu tests as temporary blockers on Rust harness
  work, not as allowed long-term survivors

## Scope

- large governed Nu test files under `nushell/scripts/dev`
- Rust-owned logic in `rust_core/yazelix_core`
- adjacent helper-heavy files where the core assertions are now deterministic
  and Rust-owned
- the Rust test-harness work needed to retire strong Nu tests cleanly

Out of scope:

- maintainer update/release/issue-sync behavior
- shell/process-heavy integration flows that still need real command execution
- mass one-to-one rewrites that preserve old fixture noise without improving the
  owner boundary

Current measured governed Nu test surface: `13,079` LOC.

## Bucket Classification

| File or bucket | Current role | Migration bucket | Why |
| --- | --- | --- | --- |
| `test_yzx_generated_configs.nu` | deterministic config/materialization coverage | `rust_port_landed_then_split_more` | the strongest generated-config and materialization assertions now defend Rust-owned logic; keep deleting the residual Nu duplicates |
| `test_yzx_core_commands.nu` | deterministic public command/control-plane coverage | `rust_port_landed_then_split_more` | the strongest public command and report assertions now belong under Rust-owned nextest suites |
| `test_yzx_workspace_commands.nu`, `test_zellij_plugin_contracts.nu` | workspace/session/plugin contracts | `rust_port_after_harness` | strong assertions remain, but they need shared fixture/process helpers before they can leave Nu honestly |
| `test_yzx_popup_commands.nu`, `test_yzx_yazi_commands.nu`, `test_yzx_doctor_commands.nu`, `test_yzx_helix_doctor_contracts.nu` | mixed popup/Yazi/doctor/editor flows | `rust_port_after_harness` | the good assertions are real contracts, but the current Nu files still mix them with wrapper-heavy execution noise |
| `test_shell_managed_config_contracts.nu` | shell-managed config and extern-bridge contracts | `rust_port_after_harness` | the strong assertions should move once the Rust harness can execute the real shell boundary; they are not allowlisted as permanent Nu tests |
| `test_yzx_maintainer.nu`, `test_config_sweep.nu`, upgrade-summary and stale-config e2e files | maintainer/sweep/e2e coverage | `delete_if_weak_or_replace_elsewhere` | keep only the strong contracts; do not preserve broad Nu omnibus files by default |
| `test_yzx_commands.nu` | command-surface/trivial routing inventory | `delete_if_weak` | this class is the easiest place to delete low-value command-discovery checks instead of porting them |
| Rust `yazelix_core` and plugin tests | Rust-owned deterministic logic | `already_rust_and_should_grow` | these are the canonical surviving owners and should absorb strong replacement coverage |

## Strong-Only Migration Rules

1. Port only tests that defend explicit contracts, regressions, or invariants
2. Delete help-output trivia, command-discovery noise, and redundant fixture
   churn instead of porting it
3. When a strong test still needs a real shell/process boundary, block it on
   the Rust harness work rather than allowing it to survive in Nu indefinitely
4. New Rust-owned test coverage should be nextest-first by default under
   `docs/specs/rust_test_hardening_tools_decision.md`

## Landed First Wave

The first landed wave already moved these clusters into Rust-owned tests:

- generated-config normalization
- runtime materialization lifecycle and missing-artifact repair
- deterministic public command-surface and report-shaping assertions

Those deletions are not the end state. They are the first cut.

## Next Migration Wave

`yazelix-rdn7.4.5.5` chooses this next wave:

1. `yazelix-rdn7.4.5.15`
   - define the shared Rust nextest harness and fixture boundary needed to
     retire strong Nu tests cleanly
2. `yazelix-rdn7.4.5.16`
   - implement the shared Rust helpers and delete redundant Nu test helpers
3. `yazelix-rdn7.4.5.7`
   - finish the next generated-config/render-plan residuals
4. `yazelix-rdn7.4.5.9`
   - port deterministic workspace/session/doctor assertions
5. `yazelix-rdn7.4.5.11`
   - port deterministic managed-config contract assertions
6. `yazelix-rdn7.4.5.13`
   - port the remaining strong `test_yzx_core_commands.nu` command-family cuts
7. `yazelix-rdn7.4.5.4`
   - delete the remaining redundant Nu tests after the replacement Rust
     coverage lands

## Generated-Config Split

`yazelix-rdn7.4.5.6` narrows `test_yzx_generated_configs.nu` into these
explicit buckets:

| Cluster | Current Nu tests | Bucket | Why |
| --- | --- | --- | --- |
| active-config, helper-selection, config-state, and schema checks | `test_active_config_normalize_uses_runtime_yzx_core_helper_when_present`, `test_active_config_normalize_surfaces_yzx_core_config_errors_without_fallback`, `test_active_config_normalize_rejects_packaged_runtime_missing_yzx_core`, `test_active_config_normalize_source_checkout_uses_explicit_yzx_core_helper`, `test_active_config_normalize_source_checkout_missing_helper_does_not_fallback`, `test_run_yzx_core_request_prefers_newer_source_checkout_helper_over_stale_release`, `test_record_materialized_state_accepts_symlinked_managed_main_config`, `test_user_mode_requires_real_terminal_config`, `test_config_schema_rejects_removed_enum_values` | `strong_rust_port` | these assertions defend Rust-owned `yzx_core` config normalization, helper selection, and config-state behavior rather than an honest surviving Nu owner |
| Yazi and Zellij materialization / render-plan contracts | `test_generate_merged_zellij_config_reuses_unchanged_state_and_invalidates_on_input_change`, `test_generate_merged_yazi_config_rejects_legacy_user_overrides`, `test_generate_merged_yazi_config_syncs_starship_plugin_config`, `test_generate_merged_yazi_config_renders_runtime_placeholders_in_plugins`, `test_generate_merged_yazi_config_skips_unchanged_managed_file_rewrites`, `test_generated_runtime_configs_prefer_active_runtime_over_installed_reference`, `test_generate_merged_zellij_config_carries_sidebar_width_to_layouts_and_plugin_config`, `test_generate_merged_zellij_config_caps_zjstatus_tab_window_with_overflow_markers`, `test_generate_merged_zellij_config_binds_ctrl_y_directly_to_pane_orchestrator_toggle`, `test_generate_merged_zellij_config_keeps_alt_m_pane_orchestrator_message_session_local`, `test_generate_merged_zellij_config_routes_popup_and_menu_through_shared_transient_pane_contract`, `test_generate_merged_zellij_config_sets_on_force_close_by_session_mode`, `test_generate_merged_zellij_config_replaces_conflicting_ui_and_serialization_settings`, `test_generate_merged_zellij_config_uses_native_user_config_without_relocating_it`, `test_generate_merged_zellij_config_prefers_managed_user_config_when_native_config_also_exists` | `strong_rust_port` | these are deterministic contracts around Rust-owned Yazi/Zellij materialization and render-plan logic and should move under `yazelix_core` nextest suites |
| terminal launch and Ghostty wrapper behavior | `test_generate_all_terminal_configs_keeps_terminal_overrides_opt_in`, `test_terminal_override_imports_ignore_yazelix_dir_runtime_root`, `test_ghostty_linux_launch_command_keeps_linux_specific_flags`, `test_ghostty_linux_launch_command_prefers_runtime_owned_nixgl_wrapper`, `test_ghostty_wayland_wrapper_falls_back_to_simple_im_without_active_daemon`, `test_ghostty_wayland_wrapper_preserves_active_ibus_env`, `test_ghostty_macos_launch_command_omits_linux_specific_flags` | `blocked` | these still depend on the surviving terminal-launch and wrapper shell owners; port only after the `yazelix-nuj1`, `yazelix-p18h`, and `yazelix-lnk6` cuts leave an honest Rust owner or a fixed shell-floor seam |
| bounded generated-artifact cleanup | `test_remove_path_within_root_refuses_root_and_outside_targets`, `test_remove_path_within_root_relaxes_read_only_managed_directories_before_recursive_cleanup`, `test_remove_path_within_root_recursive_cleanup_removes_managed_symlinks_without_touching_targets` | `blocked` | these still defend the Nu-owned bounded cleanup helpers and should not be copied into Rust until that owner cut is explicit |
| wrapper argv trivia | `test_managed_wrapper_launch_command_does_not_forward_config_mode_flag` | `weak_delete` | this is internal launch-shape trivia about not leaking a private flag, not a durable user-facing contract worth preserving across the Rust migration |

The next strong Rust port target inside this file is the Zellij/Yazi
materialization and render-plan cluster because it is already fully
deterministic, already Rust-owned, and large enough to delete a meaningful
chunk of Nu assertions in one cut.

## Workspace, Session, Doctor, And Plugin Split

`yazelix-rdn7.4.5.8` narrows the workspace/session/doctor/plugin bucket into
these explicit lanes:

| Cluster | Current Nu tests | Bucket | Why |
| --- | --- | --- | --- |
| workspace/session control-plane truth | `test_yzx_cwd_requires_zellij`, `test_public_yzx_cwd_retargets_workspace_and_syncs_plugin_owned_sidebar`, `test_public_yzx_reveal_prints_sidebar_disabled_copy`, `test_public_yzx_reveal_uses_session_snapshot_sidebar_state_and_focuses_sidebar`, `test_retarget_workspace_for_path_returns_plugin_owned_sidebar_state_and_editor_status`, `test_run_pane_orchestrator_command_raw_targets_session_plugin_without_plugin_configuration` | `strong_rust_port` | these assertions now defend Rust-owned `yzx_control` behavior or pane-orchestrator-owned session truth and should not survive in Nu by habit |
| doctor and Helix-doctor report shaping | `test_yzx_doctor_warns_on_stale_config_fields`, `test_yzx_doctor_reports_rust_owned_range_validation`, `test_yzx_doctor_fix_creates_config_from_default_template`, `test_yzx_doctor_json_reports_structured_findings`, `test_yzx_doctor_json_rejects_fix_mode`, `test_yzx_doctor_omits_installer_artifact_checks_in_runtime_root_only_mode`, `test_yzx_doctor_reports_helix_import_guidance_for_personal_config`, `test_yzx_doctor_warns_when_generated_helix_config_is_stale`, `test_doctor_fix_repairs_missing_managed_generated_layout` | `strong_rust_port` | these are deterministic doctor-report and doctor-fix contracts that already route through Rust-owned config, runtime, and Helix report owners |
| Zellij plugin-path and permission-cache contracts | `test_generate_merged_zellij_layouts_use_stable_zjstatus_plugin_path`, `test_zjstatus_permission_cache_migrates_to_tracked_and_stable_paths`, `test_pane_orchestrator_permission_cache_migrates_run_commands_to_tracked_and_stable_paths`, `test_legacy_popup_runner_artifacts_are_trimmed_from_merge_and_permissions`, `test_zjstatus_terminal_widget_falls_back_to_configured_terminal_without_env_hint` | `strong_rust_port` | these are deterministic plugin-path, permission, and generated-layout contracts around Rust-owned Zellij materialization and plugin state |
| startup, launch, and desktop host integration | `test_startup_rejects_missing_working_dir`, `test_startup_bootstrap_runtime_env_exports_state_and_logs_dirs`, `test_launch_rejects_file_working_dir`, `test_yzx_cli_desktop_launch_ignores_hostile_shell_env`, `test_yzx_desktop_launch_uses_leaf_launch_module_with_clean_env`, `test_yzx_desktop_launch_propagates_fast_path_failures_without_fallback`, `test_desktop_fast_path_rejects_bootstrap_terminal_substitution_for_explicit_terminal`, `test_desktop_fast_path_uses_direct_host_terminal_during_reload_instead_of_stale_wrapper`, `test_desktop_fast_path_rerolls_ghostty_random_cursor_config_per_window`, `test_yzx_edit_resolves_managed_helix_wrapper_from_canonical_launch_env`, `test_launch_here_path_uses_requested_directory_for_nonpersistent_sessions`, `test_yzx_launch_rejects_removed_here_flag`, `test_launch_here_path_warns_when_existing_persistent_session_ignores_it`, `test_launch_falls_through_after_immediate_terminal_failure`, `test_startup_materializes_missing_managed_layout_before_handoff`, `test_startup_custom_layout_override_fails_clearly`, `test_launch_requires_runtime_launch_script` | `blocked` | these still depend on surviving startup, launch, desktop, terminal, and wrapper shell owners and should wait for the `yazelix-w6sz.2.2`, `yazelix-w6sz.5.2`, `yazelix-nuj1.2`, `yazelix-p18h.2`, and `yazelix-lnk6.*` cuts |
| lightweight-route module trivia | `test_yzx_cli_menu_uses_lightweight_menu_module`, `test_yzx_cli_popup_uses_lightweight_popup_module`, `test_yzx_cli_enter_uses_lightweight_enter_module` | `weak_delete` | these assert implementation routing trivia rather than durable user-facing behavior and should be deleted instead of copied into Rust |
| installer-shadowing and desktop doctor details | `test_yzx_doctor_reports_stale_desktop_entry_exec`, `test_yzx_doctor_accepts_manual_stable_wrapper_desktop_entry`, `test_yzx_doctor_reports_non_terminal_desktop_entry`, `test_yzx_doctor_accepts_home_manager_install_artifacts`, `test_yzx_doctor_reports_shadowing_manual_desktop_entry_for_home_manager`, `test_yzx_doctor_reports_shadowing_manual_yzx_wrapper_for_profile_owner`, `test_yzx_doctor_reports_stale_store_pinned_shell_shadowing`, `test_yzx_doctor_reports_linux_ghostty_graphics_support_that_depends_on_host_path`, `test_yzx_doctor_reports_missing_runtime_launch_assets`, `test_yzx_doctor_respects_layout_override_for_shared_preflight` | `blocked` | the assertions are real, but they still sit on desktop-launcher, PATH-shadowing, and runtime-preflight shell boundaries that do not have a clean Rust owner cut yet |

The next strong Rust port target is the workspace/session control-plane truth
cluster plus the Zellij plugin-path contract cluster. Those assertions already
depend on typed Rust or pane-orchestrator owners and give `yazelix-rdn7.4.5.9`
the cleanest delete-first starting point.

## Managed-Config Split

`yazelix-rdn7.4.5.10` narrows the managed-config Nu tests into these explicit
lanes:

| Cluster | Current Nu tests | Bucket | Why |
| --- | --- | --- | --- |
| Helix materialization and runtime-env contracts | `test_generate_managed_helix_config_merges_user_config_and_enforces_reveal`, `test_get_runtime_env_wraps_helix_with_managed_wrapper`, `test_get_runtime_env_exports_curated_toolbin_and_keeps_runtime_local_yzx` | `strong_rust_port` | these defend deterministic Helix materialization and runtime-env behavior that already belongs to Rust-owned owners |
| extern bridge rendering and refresh semantics | `test_yzx_extern_bridge_reuses_current_fingerprint`, `test_yzx_extern_bridge_probe_ignores_host_nushell_config`, `test_yzx_extern_bridge_keeps_previous_bridge_when_refresh_fails` | `strong_rust_port` | these are deterministic Rust-owned command-metadata and extern-bridge contracts and should move into `yazelix_core` tests instead of staying in Nu |
| shell initializer, runtime resolution, and runtime setup behavior | `test_generate_merged_zellij_config_wraps_nu_default_shell`, `test_managed_nushell_config_sources_optional_user_hook`, `test_managed_nushell_config_loads_in_repo_shell_without_runtime_env`, `test_managed_bash_config_sources_optional_user_hook`, `test_managed_fish_config_does_not_export_helix_mode_env`, `test_source_checkout_runtime_resolution_beats_installed_runtime`, `test_runtime_resolution_fails_fast_without_valid_runtime_root`, `test_runtime_setup_leaves_existing_host_shell_surfaces_untouched`, `test_runtime_setup_ignores_read_only_host_shell_surfaces`, `test_yazelix_hx_ignores_legacy_runtime_alias_and_uses_wrapper_runtime_root`, `test_yzx_import_helix_copies_personal_config_with_force_backups` | `blocked` | these still depend on surviving shell initializer, wrapper, runtime-resolution, or import owners and should wait for the setup/bootstrap and runtime-helper cuts rather than pretending there is already a clean Rust owner |
| weak-delete bucket | none currently | `weak_delete` | this slice does not need fake parity deletions; the remaining non-portable assertions are blocked on real owner cuts instead of being worthlessly preserved in Nu |

The next strong Rust port target is the extern-bridge plus runtime-env /
Helix-materialization cluster, because those assertions already sit behind
Rust-owned deterministic owners and do not need the remaining shell-floor work
to move honestly.

## What Cannot Survive

These are not valid long-term steady states:

- "keep this strong test in Nu because it talks to a shell"
- "port every current assertion one-to-one even if the Rust copy is still weak"
- "leave a large omnibus file in Nu because only part of it is ready"
- "keep help-output or route-listing checks because they are cheap"

If a test is weak, delete it. If it is strong, port it once the harness can
defend the real contract honestly.

## Verification Gate

- `nu nushell/scripts/dev/validate_default_test_traceability.nu`
- `nu nushell/scripts/dev/validate_rust_test_traceability.nu`
- `nix develop -c cargo nextest run --profile ci --manifest-path rust_core/Cargo.toml -p yazelix_core`
- later plugin-owned Rust ports should use the same nextest-first policy

## Acceptance

1. The governed Nu end state is explicit: no surviving Nu tests
2. The next strong migration wave is named concretely instead of as a blanket rewrite
3. Weak tests are explicitly deleted instead of quietly preserved
4. Strong shell-heavy tests are explicitly blocked on Rust harness work rather
   than marked as permanent Nu survivors

## Traceability

- Bead: `yazelix-rdn7.4.5.1`
- Bead: `yazelix-rdn7.4.5.5`
- Informed by: `docs/specs/governed_test_traceability_inventory.md`
- Informed by: `docs/specs/rust_nextest_harness_boundary.md`
- Informed by: `docs/specs/rust_test_hardening_tools_decision.md`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
