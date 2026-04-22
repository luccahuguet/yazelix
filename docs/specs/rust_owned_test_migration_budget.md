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
