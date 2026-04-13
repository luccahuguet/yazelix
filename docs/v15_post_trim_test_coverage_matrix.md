# v15 Post-Trim Test Coverage Matrix

Traceability: `yazelix-qgj7.2.4.1`

This note explains what happened to the old pre-trim test surface after the v15 delete-first pass.
For each materially deleted or rewritten cluster, classify it as one of:

- `Contract removed`
- `Replaced by stronger surviving coverage`
- `Missing replacement coverage`

The point is not to defend every old test file. The point is to make it obvious which old responsibilities were intentionally deleted, which were narrowed and still defended, and where a real gap remains.

## 1. Refresh / launch-profile / devenv-lock era

- Deleted files:
  - `nushell/scripts/dev/test_yzx_refresh_commands.nu`
  - `nushell/scripts/dev/test_reuse_mode.nu`
  - `nushell/scripts/dev/test_build_cores.nu`
  - `nushell/scripts/dev/devenv_lock_contract.nu`
- Classification:
  - `Contract removed` for stale profile reuse, build-shell parallelism propagation, runtime-local `devenv` ownership, lock-derived `devenv` package ownership, and the old public refresh surface.
- Why:
  - v15 no longer treats Yazelix as a broad `devenv` owner. The old tests were mostly defending profile reuse, build-shell command construction, or lock/runtime indirection that no longer belongs to the retained contract.
  - Generated-state repair now lives under startup preflight, doctor guidance, and internal helper paths rather than under a public refresh command.
- Current defenses:
  - Removed runtime-manager and lock ownership is defended by [`nushell/scripts/dev/validate_installed_runtime_contract.nu`](../nushell/scripts/dev/validate_installed_runtime_contract.nu).
  - Removed `core.refresh_output` is rejected as an unsupported config surface by `test_invalid_config_is_classified_as_config_problem` in [`nushell/scripts/dev/test_yzx_core_commands.nu`](../nushell/scripts/dev/test_yzx_core_commands.nu).
  - Startup preflight still points missing generated state at `yzx doctor` through [`nushell/scripts/dev/test_yzx_workspace_commands.nu`](../nushell/scripts/dev/test_yzx_workspace_commands.nu).
  - No-migration diagnostics still cover startup plus doctor via [`nushell/scripts/dev/test_stale_config_diagnostics_e2e.nu`](../nushell/scripts/dev/test_stale_config_diagnostics_e2e.nu).

## 2. Pack sidecar and pack config surface

- Deleted surface:
  - Old pack-sidecar assumptions were spread across bootstrap/refresh/runtime checks rather than one deleted test file.
- Classification:
  - `Contract removed`
- Why:
  - v15 removed the pack sidecar as a maintained user/runtime surface. The right defense is not "old pack tests kept passing"; it is "the runtime and config parser now reject or omit that surface entirely."
- Current defenses:
  - First-run bootstrap no longer materializes `yazelix_packs.toml` in `test_parse_yazelix_config_bootstraps_main_default_surface` in [`nushell/scripts/dev/test_yzx_generated_configs.nu`](../nushell/scripts/dev/test_yzx_generated_configs.nu).
  - Legacy `[packs]` config is rejected by the consolidated `test_parse_yazelix_config_rejects_removed_surfaces_without_rewriting` test in [`nushell/scripts/dev/test_yzx_generated_configs.nu`](../nushell/scripts/dev/test_yzx_generated_configs.nu).
  - Installed runtime and installer no longer ship or seed pack files in [`nushell/scripts/dev/validate_flake_install.nu`](../nushell/scripts/dev/validate_flake_install.nu).
  - Source-level installed-runtime contract checks also forbid the old pack surfaces in [`nushell/scripts/dev/validate_installed_runtime_contract.nu`](../nushell/scripts/dev/validate_installed_runtime_contract.nu).
- Remaining gap:
  - None. The contract is intentionally gone and the absence is defended directly.

## 3. Terminal-wrapper breadth and weak terminal detection smoke

- Deleted files:
  - `nushell/scripts/dev/test_terminal_detection.nu`
- Classification:
  - `Replaced by stronger surviving coverage`
- Why:
  - The deleted file mostly checked trivia such as "a command exists," "metadata fields exist," and "a launch command string contains a terminal name." That was broad but weak.
  - The surviving tests defend real launch behavior and regression-prone platform seams instead.
- Current defenses:
  - Launch-command ownership and Ghostty platform split are defended by [`nushell/scripts/dev/test_yzx_generated_configs.nu`](../nushell/scripts/dev/test_yzx_generated_configs.nu).
  - Desktop fast-path behavior is defended by [`nushell/scripts/dev/test_yzx_workspace_commands.nu`](../nushell/scripts/dev/test_yzx_workspace_commands.nu).
  - Doctor preflight coverage around missing launch assets and layout resolution remains in [`nushell/scripts/dev/test_yzx_doctor_commands.nu`](../nushell/scripts/dev/test_yzx_doctor_commands.nu).
- Remaining gap:
  - None for the retained v15 terminal-launch contract. The surviving tests are narrower and materially stronger than the deleted smoke file.

## 4. Broad config parser smoke

- Deleted files:
  - `nushell/scripts/dev/test_config_parser.nu`
- Classification:
  - `Replaced by stronger surviving coverage`
- Why:
  - The old parser test mostly asserted that a config file existed, parsed, and had plausible-looking fields. That was breadth without much regression power.
  - The current suite focuses on migration, bootstrap, rejection, and narrowed-surface behavior.
- Current defenses:
  - Managed config bootstrap and removed-surface rejection are covered by `test_parse_yazelix_config_rejects_removed_surfaces_without_rewriting`, `test_parse_yazelix_config_bootstraps_main_default_surface`, and `test_config_schema_rejects_removed_enum_values` in [`nushell/scripts/dev/test_yzx_generated_configs.nu`](../nushell/scripts/dev/test_yzx_generated_configs.nu).
  - Removed-field rejection through the public command path is covered by `test_invalid_config_is_classified_as_config_problem` in [`nushell/scripts/dev/test_yzx_core_commands.nu`](../nushell/scripts/dev/test_yzx_core_commands.nu).
- Remaining gap:
  - None. The surviving tests defend user-visible parser behavior better than the old broad smoke checks did.

## 5. Managed config breadth split by subsystem

- Deleted files:
  - `nushell/scripts/dev/test_managed_config_contracts.nu`
  - `nushell/scripts/dev/test_integration.nu`
  - `nushell/scripts/dev/test_nix_scenarios.nu`
- Classification:
  - `Replaced by stronger surviving coverage`
- Why:
  - The monolithic managed-config file was intentionally split so Helix, shell-hook, and runtime-resolution behavior each live with the subsystem they actually defend.
  - The old Nix scenario files were lightweight environment probes rather than strong product-contract tests.
- Current defenses:
  - Shell-hook and runtime-resolution contracts are covered in [`nushell/scripts/dev/test_shell_managed_config_contracts.nu:14`](../nushell/scripts/dev/test_shell_managed_config_contracts.nu), [`nushell/scripts/dev/test_shell_managed_config_contracts.nu:66`](../nushell/scripts/dev/test_shell_managed_config_contracts.nu), [`nushell/scripts/dev/test_shell_managed_config_contracts.nu:123`](../nushell/scripts/dev/test_shell_managed_config_contracts.nu), [`nushell/scripts/dev/test_shell_managed_config_contracts.nu:216`](../nushell/scripts/dev/test_shell_managed_config_contracts.nu), [`nushell/scripts/dev/test_shell_managed_config_contracts.nu:260`](../nushell/scripts/dev/test_shell_managed_config_contracts.nu), and [`nushell/scripts/dev/test_shell_managed_config_contracts.nu:312`](../nushell/scripts/dev/test_shell_managed_config_contracts.nu).
  - Helix-specific managed-config behavior is covered in [`nushell/scripts/dev/test_helix_managed_config_contracts.nu:11`](../nushell/scripts/dev/test_helix_managed_config_contracts.nu), [`nushell/scripts/dev/test_helix_managed_config_contracts.nu:72`](../nushell/scripts/dev/test_helix_managed_config_contracts.nu), and [`nushell/scripts/dev/test_helix_managed_config_contracts.nu:131`](../nushell/scripts/dev/test_helix_managed_config_contracts.nu).
  - Real install/runtime/Nix-path coverage now comes from installed-runtime validators such as [`nushell/scripts/dev/validate_flake_install.nu`](../nushell/scripts/dev/validate_flake_install.nu) and [`nushell/scripts/dev/validate_installed_runtime_contract.nu`](../nushell/scripts/dev/validate_installed_runtime_contract.nu) rather than synthetic PATH-only scenario probes.
- Remaining gap:
  - None. This is a real strengthening, not just a rename.

## 6. Low-signal harnesses, anti-creep gates, and empty runners

- Deleted files:
  - `nushell/scripts/dev/test_yzx_dev_commands.nu`
  - `nushell/scripts/dev/test_yzx_extra_regressions.nu`
  - `nushell/scripts/dev/validate_default_test_budget.nu`
  - `nushell/scripts/dev/benchmark_config_detection.nu`
- Classification:
  - `Contract removed`
- Why:
  - These files were either empty runners, duplicate aggregators, explicit suite-runtime budget gates, or exploratory helpers that did not defend retained user-visible behavior.
- Current defenses:
  - The retained suite is governed by the stronger traceability and strength validators already in the repo, rather than by a single default-suite wall-clock cap.
- Remaining gap:
  - None. These files were noise, not missing product coverage.

## 7. Surviving high-cost default-suite overlap after the v15.0 trim

Traceability: `yazelix-qgj7.4.7`

This pass re-audited the largest surviving default-suite component files after the popup/menu, runtime, config-migration, and bd cleanup work.

| Area | Classification | Decision |
| --- | --- | --- |
| Generated config removed-surface parser checks | `Replaced by stronger surviving coverage` | Three separate parser tests for removed v14 surfaces were merged into one table-driven `test_parse_yazelix_config_rejects_removed_surfaces_without_rewriting` check. It still covers legacy `[ascii]`, removed `shell.enable_atuin`, and legacy `[packs]`, and it now uniformly asserts no config rewrites or backup churn. |
| Generated config removed enum checks | `Replaced by stronger surviving coverage` | The separate removed-enum tests for `terminal.config_mode = "auto"` and `zellij.widget_tray = ["layout"]` were merged into one table-driven `test_config_schema_rejects_removed_enum_values` check. |
| Generated config terminal, Yazi, Zellij, safe-remove, and transient-pane checks | `Keep` | These still defend distinct launch/platform seams, generated artifact safety, native-versus-managed config ownership, and popup/menu helperless routing. They are heavy but not duplicates. |
| Workspace and Yazi pane-orchestrator fixtures | `Keep, fixture hardened` | The behavior remains distinct and live. The fake `zellij` test fixtures were changed to match pipe command names by argv value instead of brittle positional `$6` assumptions. |
| `yzx status` default-suite assertion | `Keep, assertion narrowed` | The test still defends that `yzx status` reaches the shared environment bootstrap and prints the summary fields. It no longer requires the full config path to be unwrapped in Nushell's pretty table output. |
| Maintainer workflow file | `Keep for maintainer lane` | `test_yzx_maintainer.nu` remains large, but it is outside the default runner and maps to issue sync, update, bump, profile, and plugin-refresh maintainer contracts. No safe delete was found in this pass. |

The default-suite count moved from 92 to 89 canonical tests after the overlap trim, then to 90 when the command-surface help-description contract got a generated-metadata check. The old count-budget cap of 53 was stale relative to the actual suite, so it was reset to the current deliberate count after the consolidation. Future default-suite additions should again fail the budget gate unless they update the cap deliberately.

## Bottom line

- The pack-sidecar era, build-shell parallelism era, runtime-local `devenv` ownership, and lock-derived runtime ownership were intentionally removed and are now defended mainly by absence checks.
- Terminal launch, managed-config behavior, parser behavior, and runtime/install behavior are defended by stronger surviving tests than the deleted broad smoke files provided.
- The main remaining gaps are now around the surviving startup/doctor-generated-state seams rather than around any deleted public refresh command.
