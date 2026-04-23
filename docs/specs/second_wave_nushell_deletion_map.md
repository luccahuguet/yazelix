# Second-Wave Nushell Deletion Map

## Summary

This map is the plan gate for the second hard Nu-deletion tranche.

The tracked surface after `yazelix-lj7z.3` is `12,101` Nu LOC across `80` files. The
hard floor remains `4,200` LOC. The next tranche is not another audit pass: it
names the live implementation owners for every remaining budget family and
classifies each tracked Nu file by the next deletion action.

## Current Budget Owners

| Family | Current LOC | Target LOC | Live owner |
| --- | ---: | ---: | --- |
| Manual E2E and sweep runners | `881` | `0` | `yazelix-lj7z.9` |
| Deterministic validators | `0` | `0` | `yazelix-lj7z.2` |
| Maintainer and `yzx dev` orchestration | `3,486` | `1,200` | `yazelix-lj7z.3`, `yazelix-lj7z.4` |
| Integration and wrappers | `1,328` | `300` | `yazelix-lj7z.7` |
| Setup and bootstrap | `1,070` | `500` | `yazelix-lj7z.6` |
| Front-door presentation | `2,281` | `950` | `yazelix-lj7z.8` |
| Runtime helpers | `2,483` | `1,050` | `yazelix-lj7z.5`, `yazelix-lj7z.10` |
| Session and desktop host integration | `572` | `200` | `yazelix-lj7z.6` |

## Classification Rules

- `delete`: remove the file or feature surface outright after proving it is weak, duplicated, or historical
- `rust_port`: move deterministic policy, request construction, validation, report shaping, or data transformation to Rust
- `asset_or_data`: move static art, copy, tables, or pattern payloads out of Nu
- `fixed_shell`: retain only direct shell, host, TTY, or external-command execution
- `blocked`: do not implement until the named owner cut lands, because otherwise the result would be a wrapper

## Manual E2E And Sweep Runners

| File | LOC | Next action | Owner |
| --- | ---: | --- | --- |
| `nushell/scripts/dev/config_sweep_runner.nu` | `325` | `rust_port` strong matrix planning and demote live terminal execution to fixed shell | `yazelix-lj7z.9` |
| `nushell/scripts/dev/historical_upgrade_notes_e2e_runner.nu` | `90` | `delete` or port only the retained historical regression to Rust | `yazelix-lj7z.9` |
| `nushell/scripts/dev/stale_config_diagnostics_e2e_runner.nu` | `91` | `rust_port` deterministic stale-config assertions | `yazelix-lj7z.9` |
| `nushell/scripts/dev/upgrade_contract_e2e_runner.nu` | `216` | `rust_port` contract cases, keep no broad Nu runner | `yazelix-lj7z.9` |
| `nushell/scripts/dev/upgrade_summary_e2e_runner.nu` | `159` | `rust_port` summary assertions after front-door copy owner is clear | `yazelix-lj7z.9` |

## Deterministic Validators

`yazelix-lj7z.2` deleted the deterministic Nu validator cluster and moved the
live behavior to Rust `yzx_repo_validator` commands: `validate-nushell-syntax`,
`validate-flake-interface`, `validate-flake-profile-install`,
`validate-nixpkgs-package`, and `validate-nixpkgs-submission`.

## Maintainer And Dev Orchestration

| File | LOC | Next action | Owner |
| --- | ---: | --- | --- |
| `nushell/scripts/dev/config_normalize_test_helpers.nu` | `56` | `delete` after deterministic helper callers move to Rust | `yazelix-lj7z.9` |
| `nushell/scripts/dev/materialization_dev_helpers.nu` | `100` | `rust_port` materialization helper requests or delete | `yazelix-lj7z.4` |
| `nushell/scripts/dev/record_demo.nu` | `102` | `delete` or demote to manual non-budget tooling | `yazelix-lj7z.4` |
| `nushell/scripts/dev/record_demo_fonts.nu` | `140` | `delete` or demote to manual non-budget tooling | `yazelix-lj7z.4` |
| `nushell/scripts/dev/sweep/sweep_config_generator.nu` | `71` | `rust_port` sweep matrix data generation | `yazelix-lj7z.9` |
| `nushell/scripts/dev/sweep/sweep_process_manager.nu` | `82` | `fixed_shell` only for live terminal process management | `yazelix-lj7z.9` |
| `nushell/scripts/dev/sweep/sweep_test_combinations.nu` | `91` | `rust_port` deterministic combination generation | `yazelix-lj7z.9` |
| `nushell/scripts/dev/sweep/sweep_test_executor.nu` | `203` | `rust_port` deterministic execution plans, retain terminal spawn only if needed | `yazelix-lj7z.9` |
| `nushell/scripts/dev/sweep_verify.nu` | `73` | `rust_port` deterministic verification or delete if duplicate | `yazelix-lj7z.9` |
| `nushell/scripts/dev/update_yazi_plugins.nu` | `184` | `rust_port` update planning or fold into maintainer update owner | `yazelix-lj7z.4` |
| `nushell/scripts/dev/yzx_test_helpers.nu` | `231` | `delete` after E2E runners move to Rust harness | `yazelix-lj7z.9` |
| `nushell/scripts/maintainer/issue_bead_contract.nu` | `223` | `rust_port` deterministic GitHub/Beads mapping validation | `yazelix-lj7z.4` |
| `nushell/scripts/maintainer/issue_sync.nu` | `271` | `rust_port` issue mapping policy, keep fixed `gh`/`bd` execution only if smaller | `yazelix-lj7z.4` |
| `nushell/scripts/maintainer/plugin_build.nu` | `123` | `fixed_shell` unless deterministic wasm sync policy can move to Rust | `yazelix-lj7z.4` |
| `nushell/scripts/maintainer/repo_checkout.nu` | `89` | `rust_port` helper selection policy, keep no tribal source-checkout fixes | `yazelix-lj7z.4` |
| `nushell/scripts/maintainer/update_workflow.nu` | `768` | `rust_port` release/update/version policy | `yazelix-lj7z.4` |
| `nushell/scripts/maintainer/version_bump.nu` | `268` | `rust_port` version edit and dirty-state policy | `yazelix-lj7z.4` |
| `nushell/scripts/yzx/dev.nu` | `410` | `fixed_shell` router only after maintainer policy moves | `yazelix-lj7z.4` |

## Integration And Wrappers

| File | LOC | Next action | Owner |
| --- | ---: | --- | --- |
| `nushell/scripts/integrations/helix.nu` | `4` | `delete` or inline | `yazelix-lj7z.7` |
| `nushell/scripts/integrations/managed_editor.nu` | `249` | `rust_port` workspace/editor plan, retain process execution only | `yazelix-lj7z.7` |
| `nushell/scripts/integrations/open_dir_in_pane.nu` | `50` | `delete` or fold into typed pane-orchestrator action | `yazelix-lj7z.7` |
| `nushell/scripts/integrations/yazi.nu` | `287` | `rust_port` sidebar/Yazi state shaping, retain `ya` execution only | `yazelix-lj7z.7` |
| `nushell/scripts/integrations/zellij.nu` | `459` | `rust_port` session/action planning into pane orchestrator or Rust control owner | `yazelix-lj7z.7` |
| `nushell/scripts/integrations/zellij_runtime_wrappers.nu` | `95` | `delete` after runtime env request construction moves | `yazelix-lj7z.7` |
| `nushell/scripts/integrations/zoxide_open_in_editor.nu` | `39` | `delete` or inline after managed-editor cut | `yazelix-lj7z.7` |
| `nushell/scripts/zellij_wrappers/launch_sidebar_yazi.nu` | `14` | `delete` as fixed wrapper after direct action exists | `yazelix-lj7z.7` |
| `nushell/scripts/zellij_wrappers/refresh_yazi_sidebar.nu` | `8` | `delete` as fixed wrapper after direct action exists | `yazelix-lj7z.7` |
| `nushell/scripts/zellij_wrappers/yzx_menu_popup.nu` | `19` | `delete` after popup runner direct route exists | `yazelix-lj7z.7` |
| `nushell/scripts/zellij_wrappers/yzx_popup_program.nu` | `104` | `fixed_shell` only if popup process/env handoff remains irreducible | `yazelix-lj7z.7` |

## Setup, Launch, Session, And Desktop

| File | LOC | Next action | Owner |
| --- | ---: | --- | --- |
| `nushell/scripts/core/launch_yazelix.nu` | `308` | `rust_port` launch request/materialization planning, retain terminal exec only | `yazelix-lj7z.6` |
| `nushell/scripts/core/start_yazelix.nu` | `143` | `fixed_shell` current-shell entry only | `yazelix-lj7z.6` |
| `nushell/scripts/core/start_yazelix_inner.nu` | `197` | `rust_port` startup planning and welcome handoff, retain Zellij exec only | `yazelix-lj7z.6` |
| `nushell/scripts/setup/environment.nu` | `173` | `fixed_shell` shell environment/bootstrap only | `yazelix-lj7z.6` |
| `nushell/scripts/setup/initializers.nu` | `249` | `rust_port` initializer content generation or move static shell bodies to assets | `yazelix-lj7z.6` |
| `nushell/scripts/core/yzx_session.nu` | `136` | `rust_port` session command planning, retain Zellij process execution only | `yazelix-lj7z.6` |
| `nushell/scripts/yzx/desktop.nu` | `299` | `rust_port` desktop-entry request planning and report shaping | `yazelix-lj7z.6` |
| `nushell/scripts/yzx/launch.nu` | `137` | `fixed_shell` launch delegation only after runtime env request shaping moves | `yazelix-lj7z.6` |

## Front-Door Presentation

| File | LOC | Next action | Owner |
| --- | ---: | --- | --- |
| `nushell/scripts/setup/welcome.nu` | `161` | `rust_port` message construction and welcome renderer call, retain startup prompt only if needed | `yazelix-lj7z.8` |
| `nushell/scripts/utils/ascii_art.nu` | `986` | `rust_port` renderer/simulation; `asset_or_data` static art and shape payloads | `yazelix-lj7z.8` |
| `nushell/scripts/utils/upgrade_summary.nu` | `248` | `rust_port` upgrade summary shaping and copy assembly | `yazelix-lj7z.8` |
| `nushell/scripts/yzx/edit.nu` | `186` | `rust_port` config-surface planning, retain editor process execution only | `yazelix-lj7z.8` |
| `nushell/scripts/yzx/import.nu` | `224` | `rust_port` import planning and diagnostics | `yazelix-lj7z.8` |
| `nushell/scripts/yzx/menu.nu` | `207` | `rust_port` menu metadata, retain `fzf`/popup shell interaction only | `yazelix-lj7z.8` |
| `nushell/scripts/yzx/popup.nu` | `64` | `fixed_shell` transient popup handoff only after contract shaping moves | `yazelix-lj7z.8` |
| `nushell/scripts/yzx/screen.nu` | `117` | `rust_port` renderer/simulation, retain terminal mode only if crossterm is rejected | `yazelix-lj7z.8` |
| `nushell/scripts/yzx/tutor.nu` | `80` | `rust_port` static copy or delete if superseded by docs assets | `yazelix-lj7z.8` |
| `nushell/scripts/yzx/whats_new.nu` | `8` | `delete` after upgrade summary command route is Rust-owned | `yazelix-lj7z.8` |

## Runtime Helpers

| File | LOC | Next action | Owner |
| --- | ---: | --- | --- |
| `nushell/scripts/utils/atomic_writes.nu` | `73` | `fixed_shell` or delete after Rust owns caller writes | `yazelix-lj7z.5` |
| `nushell/scripts/utils/build_policy.nu` | `80` | `rust_port` build policy facts | `yazelix-lj7z.5` |
| `nushell/scripts/utils/common.nu` | `323` | `rust_port` path/platform facts, leave tiny shell primitives only | `yazelix-lj7z.10` |
| `nushell/scripts/utils/config_contract.nu` | `36` | `delete` after config contract data is Rust-owned | `yazelix-lj7z.5` |
| `nushell/scripts/utils/config_files.nu` | `59` | `rust_port` config file operations or fold into active config owner | `yazelix-lj7z.5` |
| `nushell/scripts/utils/config_paths.nu` | `10` | `delete` or inline | `yazelix-lj7z.5` |
| `nushell/scripts/utils/config_report_rendering.nu` | `52` | `rust_port` report rendering | `yazelix-lj7z.5` |
| `nushell/scripts/utils/constants.nu` | `94` | `asset_or_data` plus Rust facts; leave no catch-all constants bag | `yazelix-lj7z.10` |
| `nushell/scripts/utils/cursor_trail_helpers.nu` | `40` | `rust_port` cursor effect facts | `yazelix-lj7z.5` |
| `nushell/scripts/utils/doctor_fix.nu` | `171` | `rust_port` fix planning, retain user-approved file/process actions only | `yazelix-lj7z.5` |
| `nushell/scripts/utils/doctor_helix.nu` | `26` | `delete` after doctor fix plan owns Helix conflict repair | `yazelix-lj7z.5` |
| `nushell/scripts/utils/editor_launch_context.nu` | `58` | `rust_port` editor env request construction | `yazelix-lj7z.5` |
| `nushell/scripts/utils/environment_bootstrap.nu` | `31` | `delete` after startup env facts are direct Rust calls | `yazelix-lj7z.5` |
| `nushell/scripts/utils/failure_classes.nu` | `22` | `rust_port` failure classification strings | `yazelix-lj7z.5` |
| `nushell/scripts/utils/helix_mode.nu` | `23` | `delete` or inline after editor launch context moves | `yazelix-lj7z.5` |
| `nushell/scripts/utils/integration_facts.nu` | `16` | `delete` after callers invoke Rust directly | `yazelix-lj7z.5` |
| `nushell/scripts/utils/keypress_polling.nu` | `33` | `fixed_shell` only if Rust terminal polling is rejected | `yazelix-lj7z.8` |
| `nushell/scripts/utils/logging.nu` | `23` | `delete` or inline | `yazelix-lj7z.5` |
| `nushell/scripts/utils/runtime_env.nu` | `33` | `delete` after runtime env calls are direct Rust/fixed argv | `yazelix-lj7z.5` |
| `nushell/scripts/utils/safe_remove.nu` | `48` | `rust_port` safe remove semantics or inline | `yazelix-lj7z.5` |
| `nushell/scripts/utils/shell_user_hooks.nu` | `44` | `fixed_shell` generated shell hook bridge only | `yazelix-lj7z.5` |
| `nushell/scripts/utils/startup_facts.nu` | `16` | `delete` after direct Rust startup facts calls | `yazelix-lj7z.5` |
| `nushell/scripts/utils/startup_profile.nu` | `274` | `rust_port` profile record/report shaping | `yazelix-lj7z.6` |
| `nushell/scripts/utils/terminal_launcher.nu` | `305` | `rust_port` terminal request/probe planning, retain process spawn only | `yazelix-lj7z.6` |
| `nushell/scripts/utils/transient_pane_contract.nu` | `82` | `rust_port` transient pane contract shaping | `yazelix-lj7z.5` |
| `nushell/scripts/utils/transient_pane_facts.nu` | `16` | `delete` after direct Rust calls | `yazelix-lj7z.5` |
| `nushell/scripts/utils/yzx_core_bridge.nu` | `371` | `rust_port` or delete general bridge; retain only minimum transport if any | `yazelix-lj7z.5` |
| `nushell/scripts/utils/zjstatus_widget.nu` | `124` | `rust_port` widget request/report shaping | `yazelix-lj7z.5` |

## Cut Order

1. `yazelix-lj7z.2`: finish validators first so CI/pre-commit stop depending on Nu wrappers
2. `yazelix-lj7z.3`: remove the Nu test runner before broader maintainer rewrites
3. `yazelix-lj7z.5` and `yazelix-lj7z.10`: collapse shared bridge/helper imports before editing many callers
4. `yazelix-lj7z.6`: use the smaller helper boundary to cut launch/session/desktop request assembly
5. `yazelix-lj7z.7`: collapse integration wrappers after session/workspace facts are narrowed
6. `yazelix-lj7z.8`: port front-door renderers with an explicit Rust terminal dependency decision
7. `yazelix-lj7z.9`: port or delete E2E/sweep runners after product owners settle
8. `yazelix-lj7z.4`: collapse remaining maintainer release/update/issue policy as the final operational surface pass

## Stop Conditions

- If a child would only add a Rust wrapper above the same Nu owner, stop and narrow the contract instead
- If a shell-heavy survivor remains, lower its file-local budget and record the exact shell/TTY/host behavior it owns
- If a feature has no strong test or live spec, create the missing contract before deleting the only executable behavior

## Verification

- `yzx_repo_validator validate-specs`
- `yzx_repo_validator validate-nushell-budget`
- manual review of `config_metadata/nushell_budget.toml`

## Traceability

- Bead: `yazelix-lj7z`
- Bead: `yazelix-lj7z.1`
- Defended by: `yzx_repo_validator validate-specs`
- Defended by: `yzx_repo_validator validate-nushell-budget`
