# Test Surface Inventory

## Purpose

This is the working inventory for the full Yazelix test-audit program.

It exists to make pruning decisions from explicit current reality instead of memory.

## Current Snapshot

- Total `test_*.nu` files: `19`
- Default-lane files: `8`
- Default canonical tests: `52`
- Maintainer-lane files: `8`
- Sweep-lane files: `1`
- Manual-lane files: `0`
- Support files: `0`

## File Inventory

| File | Lane | Test Count | Rough Contract | First Verdict |
| --- | --- | ---: | --- | --- |
| `nushell/scripts/dev/test_yzx_commands.nu` | `default` | `0` | Default-suite entrypoint and profile wrapper | Keep |
| `nushell/scripts/dev/test_yzx_core_commands.nu` | `default` | `13` | Core config migration and config-surface behavior | Audit test-by-test |
| `nushell/scripts/dev/test_yzx_doctor_commands.nu` | `default` | `4` | Doctor diagnosis and safe-fix flows | Audit test-by-test |
| `nushell/scripts/dev/test_yzx_generated_configs.nu` | `default` | `16` | Generated config and merge contracts | Audit test-by-test |
| `nushell/scripts/dev/test_yzx_popup_commands.nu` | `default` | `5` | Popup command and wrapper behavior | Audit test-by-test |
| `nushell/scripts/dev/test_yzx_refresh_commands.nu` | `default` | `1` | Refresh failure reporting contract | Audit test-by-test |
| `nushell/scripts/dev/test_yzx_screen_commands.nu` | `default` | `3` | Welcome screen/style behavior | Audit test-by-test |
| `nushell/scripts/dev/test_yzx_workspace_commands.nu` | `default` | `8` | Launch/workspace/session behavior | Audit test-by-test |
| `nushell/scripts/dev/test_yzx_yazi_commands.nu` | `default` | `2` | Yazi integration behavior | Audit test-by-test |
| `nushell/scripts/dev/test_yzx_maintainer.nu` | `maintainer` | `4` | Maintainer workflow and repo-contract checks | Keep |
| `nushell/scripts/dev/test_yzx_gc_commands.nu` | `maintainer` | `3` | `yzx gc` feedback/phase UX | Keep |
| `nushell/scripts/dev/test_managed_config_contracts.nu` | `maintainer` | `12` | Managed Helix, shell-hook, Nushell bridge, and Zellij plugin-path contracts | Keep |
| `nushell/scripts/dev/test_yzx_helix_doctor_contracts.nu` | `maintainer` | `2` | Helix-specific doctor guidance and stale-config diagnostics | Keep |
| `nushell/scripts/dev/test_config_migrate_e2e.nu` | `maintainer` | `0` | Dedicated migrate end-to-end runner | Keep |
| `nushell/scripts/dev/test_historical_upgrade_notes_e2e.nu` | `maintainer` | `0` | Historical upgrade notes end-to-end runner | Keep |
| `nushell/scripts/dev/test_stale_config_diagnostics_e2e.nu` | `maintainer` | `0` | Stale-config startup diagnostics e2e runner | Keep |
| `nushell/scripts/dev/test_upgrade_contract_e2e.nu` | `maintainer` | `0` | Upgrade contract e2e runner | Keep |
| `nushell/scripts/dev/test_upgrade_summary_e2e.nu` | `maintainer` | `0` | Upgrade summary e2e runner | Keep |
| `nushell/scripts/dev/test_config_sweep.nu` | `sweep` | `0` | Non-visual matrix coverage | Keep |
| `nushell/scripts/dev/record_demo_fonts.nu` | `manual helper` | `0` | Manual font/demo exploration helper, not part of the governed test surface | Keep out of `test_*.nu` |
| `nushell/scripts/dev/yzx_test_helpers.nu` | `helper library` | `0` | Shared test helpers, not part of the governed `test_*.nu` surface | Keep |

## Audit Order

1. Audit the default lane test-by-test.
2. Audit nondefault files with the highest likely dead-weight first:
   - `test_managed_config_contracts.nu`
   - `test_yzx_helix_doctor_contracts.nu`
   - `test_yzx_maintainer.nu`
3. Audit dedicated e2e runners file-by-file.
4. Apply deletions, renames, and runner simplifications in one prune pass.

## Notes

- The default lane already carries `# Strength: N/10` markers and a minimum `7/10` bar.
- This inventory is about the current governed surface only. Historical prune details belong in Beads, not here.

## Preliminary Findings

### First small-file pass

- `test_yzx_refresh_commands.nu`
  - Current verdict: `keep`
  - Reason: the single test protects a real recovery/diagnostic contract and is cheap.
- `test_yzx_yazi_commands.nu`
  - Current verdict: `keep`
  - Reason: both tests look like real user-facing integration behavior, not trivia.
- `test_yzx_doctor_commands.nu`
  - Current verdict: `keep`
  - Reason: these are strong recovery-path tests and clearly belong in the default lane.
- `test_yzx_screen_commands.nu`
  - Current verdict: `prune candidate`
  - Reason: the file had implementation-shape coverage mixed into otherwise legitimate user-facing screen behavior.

### Default-lane pass in progress

- `test_yzx_workspace_commands.nu`
  - Current verdict: `mostly keep`
  - Reason: the file is centered on launch/session behavior and still looks like real product-contract coverage.
- `test_yzx_popup_commands.nu`
  - Current verdict: `mostly keep`
  - Reason: permission denial and canonical-editor tests are strong; the command/cwd defaults are weaker but still plausibly default-lane behavior.
- `test_yzx_generated_configs.nu`
  - Current verdict: `mixed`
  - Strong cluster:
    - Zellij ownership tests
    - pack split/legacy rejection tests
    - read-only migration-path tests
    - user terminal mode fail-fast
  - Likely demote/remove cluster:
    - welcome-style bootstrap shape
    - Yazi plugin fallback default
    - split-surface bootstrap shape
    - pack-sidecar bootstrap shape
    - possibly schema-only enum rejection checks if a cheaper validator can own them

### Current borderline file shapes

- `test_managed_config_contracts.nu`
  - still broad enough that it may deserve a future split by subsystem

## Nondefault Lane Findings In Progress

- `test_yzx_helix_doctor_contracts.nu`
  - Current verdict: `keep`
  - Reason: both tests defend real Helix-doctor behavior and now have an explicit ownership-based filename.

- `test_managed_config_contracts.nu`
  - Current verdict: `keep for now, still a future split candidate`
  - Reason: it is still a mixed bucket containing:
    - Helix managed-config contracts
    - managed Nushell contracts
    - Zellij `zjstatus` path/permission contracts
    - managed Bash hook contracts
    - Helix import and first-run notice contracts
  - This is still broad, but it is no longer a generic overflow bucket.

### Early nondefault keep candidates

- `test_yzx_doctor_reports_helix_import_guidance_for_personal_config`
- `test_yzx_doctor_warns_when_generated_helix_config_is_stale`
- `test_generate_managed_helix_config_merges_user_config_and_enforces_reveal`
- `test_get_launch_env_wraps_helix_with_managed_wrapper`
- `test_generate_merged_zellij_layouts_use_stable_zjstatus_plugin_path`
- `test_zjstatus_permission_cache_migrates_to_tracked_and_stable_paths`
- `test_managed_nushell_config_loads_generated_yzx_extern_bridge`

## Applied Audit Outcome

- The default lane was pruned and now holds `52` canonical tests.
- The governed `test_*.nu` surface was reduced to `19` files.
- Manual/demo and helper code were moved out of the governed `test_*.nu` namespace.
- Nondefault file names now reflect current ownership instead of generic overflow naming.

## Maintainer and E2E Keep Verdicts

- `test_yzx_maintainer.nu`
  - Keep.
  - Reason: it defends the GitHub/Beads contract and runtime-owned `devenv` resolution, which are maintainer-only but real.

- `test_yzx_gc_commands.nu`
  - Keep.
  - Reason: it protects user-visible `yzx gc` feedback and fail-loudly behavior with cheap command fakes.

- `test_config_migrate_e2e.nu`
  - Keep.
  - Reason: it exercises real multi-case migration flows that would be awkward to defend with smaller unit-style checks.

- `test_historical_upgrade_notes_e2e.nu`
  - Keep.
  - Reason: it protects the historical upgrade-notes coverage floor against silent release-note drift.

- `test_stale_config_diagnostics_e2e.nu`
  - Keep.
  - Reason: it covers startup, refresh, and doctor behavior together for stale/unsupported config states.

- `test_upgrade_contract_e2e.nu`
  - Keep.
  - Reason: it protects the changelog/upgrade-notes CI contract with real git-fixture mutation cases.

- `test_upgrade_summary_e2e.nu`
  - Keep.
  - Reason: it defends the first-run versus manual reopen summary behavior end to end.

- `test_config_sweep.nu`
  - Keep.
  - Reason: it owns the non-visual cross-shell and cross-terminal matrix, which no cheaper lane can replace.
