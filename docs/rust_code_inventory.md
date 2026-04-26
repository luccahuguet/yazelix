# Rust Code Inventory

This inventory records the post-migration Rust shape so cleanup work starts from concrete ownership instead of raw line count.

Baseline measured on 2026-04-26:

- `46,583` tracked first-party Rust LOC excluding `target/`
- `92` tracked `.rs` files excluding `target/`
- `42,913` LOC under source files
- `3,670` LOC under Rust integration-test files
- `cargo check -p yazelix_core` reported no Rust warnings
- `cargo check` for the Zellij pane orchestrator initially reported unused plugin layout config fields; this pass removed them
- `record_demo` does not exist in the tracked Rust tree
- `run_visual_verification` is live maintainer sweep code, not dead demo code

Disposition meanings:

- `canonical`: keep as a first-class owner
- `canonical-maintainer`: keep, but it belongs to maintainer tooling rather than user runtime behavior
- `temporary`: keep for now with a deletion/collapse trigger
- `simplified`: kept, and this pass removed stale or redundant code from it
- `split-candidate`: keep behavior, but reconsider crate/module ownership in a later split

This is not a task list. Follow-up execution belongs in Beads.

## Binaries And Core Library

| File | LOC | Owner | Disposition | Reason |
| --- | ---: | --- | --- | --- |
| `rust_core/yazelix_core/src/bin/yzx.rs` | 140 | public CLI dispatcher | canonical | Small Rust front door for public `yzx` routing, including internal Nu family fallback |
| `rust_core/yazelix_core/src/bin/yzx_control.rs` | 965 | public command implementation dispatcher | canonical | Owns user-facing Rust command families and structured errors |
| `rust_core/yazelix_core/src/bin/yzx_core.rs` | 1254 | machine-readable helper protocol | temporary | Still used by startup, setup, wrappers, Home Manager repair, Helix wrapper, and maintainer workflows; collapse only after those machine callers have another stable owner |
| `rust_core/yazelix_core/src/bin/yzx_repo_maintainer.rs` | 283 | maintainer workflow runner | canonical-maintainer | Deliberate private maintainer command surface |
| `rust_core/yazelix_core/src/bin/yzx_repo_validator.rs` | 243 | repo validator dispatcher | canonical-maintainer | Central validator entrypoint; later crate split can move it out of runtime crate |
| `rust_core/yazelix_core/src/lib.rs` | 162 | crate module and re-export surface | canonical | Broad because binaries and integration tests import shared owners directly |
| `rust_core/yazelix_core/src/bridge.rs` | 219 | shared error/envelope model | canonical | Historical name, but the code is now the Rust error contract, not a transitional bridge |
| `rust_core/yazelix_core/src/cli_render.rs` | 60 | terminal text helpers | canonical | Small shared formatting helper |

## User Runtime And Command Owners

| File | LOC | Owner | Disposition | Reason |
| --- | ---: | --- | --- | --- |
| `rust_core/yazelix_core/src/active_config_surface.rs` | 276 | active config path resolver | canonical | Keeps canonical user config ownership explicit and rejects legacy duplicate surfaces |
| `rust_core/yazelix_core/src/command_metadata.rs` | 536 | public command metadata and Nushell extern generation | canonical | The extern bridge is still canonical for Nushell shell completions because `yzx` is an external command there |
| `rust_core/yazelix_core/src/config_commands.rs` | 382 | `yzx config` | canonical | Public config command implementation |
| `rust_core/yazelix_core/src/config_normalize.rs` | 871 | config normalization and validation | canonical | Central config contract owner; legacy rejection paths are user-safety guardrails |
| `rust_core/yazelix_core/src/config_state.rs` | 579 | generated-state hash lifecycle | canonical | Current rebuild-state owner; legacy malformed cache handling remains defensive |
| `rust_core/yazelix_core/src/control_plane.rs` | 667 | shared env/path request construction | canonical | Repeated path/env logic is centralized here rather than duplicated in command families |
| `rust_core/yazelix_core/src/doctor_commands.rs` | 942 | `yzx doctor` report and repair flow | canonical | Rust now owns report/fix coordination for user-facing doctor behavior |
| `rust_core/yazelix_core/src/doctor_config_report.rs` | 376 | doctor config findings | canonical | Shared config-drift diagnostics |
| `rust_core/yazelix_core/src/doctor_helix_report.rs` | 677 | doctor Helix findings | canonical | Detects Helix runtime and managed config issues |
| `rust_core/yazelix_core/src/doctor_runtime_report.rs` | 351 | doctor runtime/distribution findings | canonical | Keeps install/runtime mode diagnostics explicit |
| `rust_core/yazelix_core/src/edit_commands.rs` | 578 | `yzx edit` | canonical | Rust-owned editor/config edit command |
| `rust_core/yazelix_core/src/front_door_commands.rs` | 391 | screen, tutor, upgrade-summary commands | canonical | Public front-door commands, with shell-specific tutor entry still external by nature |
| `rust_core/yazelix_core/src/front_door_render.rs` | 1360 | terminal screen renderer | canonical | Large but cohesive renderer/game-of-life owner after the screen refactor |
| `rust_core/yazelix_core/src/ghostty_materialization.rs` | 482 | Ghostty config generation | canonical | Runtime materialization owner for Ghostty-specific config |
| `rust_core/yazelix_core/src/helix_materialization.rs` | 305 | Helix config generation/import notice | canonical | Rust-owned generated Helix write lifecycle |
| `rust_core/yazelix_core/src/home_manager_commands.rs` | 337 | `yzx home_manager` | canonical | Public Home Manager helper surface |
| `rust_core/yazelix_core/src/import_commands.rs` | 398 | `yzx import` | canonical | Rust-owned import command |
| `rust_core/yazelix_core/src/initializer_commands.rs` | 553 | shell initializer generation | canonical | Still owns generated shell initializer text; external shell semantics justify dedicated code |
| `rust_core/yazelix_core/src/install_ownership_env.rs` | 100 | install ownership env request builder | canonical | Small shared control-plane helper |
| `rust_core/yazelix_core/src/install_ownership_report.rs` | 1105 | install ownership diagnostics | canonical | Large but user-safety-heavy; legacy wrapper detection remains valuable for upgrades |
| `rust_core/yazelix_core/src/internal_nu_runner.rs` | 181 | internal Nu route executor | temporary | Keep while `yzx dev`, menu/edit/import/popup wrappers, and process-heavy flows remain Nu-owned |
| `rust_core/yazelix_core/src/keys_commands.rs` | 601 | `yzx keys` | canonical | Public key-discovery command |
| `rust_core/yazelix_core/src/launch_commands.rs` | 1587 | launch/restart/desktop command orchestration | canonical | Large because it sits on shell/process/terminal boundaries; no safe broad deletion found |
| `rust_core/yazelix_core/src/launch_materialization.rs` | 338 | launch materialization planning | canonical | Separates launch prep from command UX |
| `rust_core/yazelix_core/src/layout_family_contract.rs` | 331 | built-in Zellij layout metadata validator | canonical | Current layout family guardrail |
| `rust_core/yazelix_core/src/profile_commands.rs` | 791 | startup profiling commands | canonical-maintainer | Used by profiling workflows; not runtime critical |
| `rust_core/yazelix_core/src/public_command_surface.rs` | 1326 | public command metadata and routing | canonical | Large but central; prevents duplicated Nu/help/menu command registries |
| `rust_core/yazelix_core/src/runtime_contract.rs` | 1036 | runtime/preflight contract | canonical | Core launch dependency and preflight contract owner |
| `rust_core/yazelix_core/src/runtime_env.rs` | 253 | runtime environment computation | canonical | Small deterministic env owner |
| `rust_core/yazelix_core/src/runtime_materialization.rs` | 707 | generated runtime-state orchestration | canonical | Single materialization lifecycle owner |
| `rust_core/yazelix_core/src/startup_facts.rs` | 168 | startup facts | canonical | Small machine contract for startup shell boundary |
| `rust_core/yazelix_core/src/status_report.rs` | 134 | status report | canonical | Public status data owner |
| `rust_core/yazelix_core/src/support_commands.rs` | 162 | `yzx why` and `yzx sponsor` | canonical | Small public helper commands |
| `rust_core/yazelix_core/src/terminal_materialization.rs` | 487 | terminal config generation | canonical | Runtime materialization owner for terminal emulator configs |
| `rust_core/yazelix_core/src/transient_pane_facts.rs` | 91 | transient pane facts | canonical | Small machine contract for popup/menu wrappers |
| `rust_core/yazelix_core/src/update_commands.rs` | 497 | `yzx update` | canonical | Public update command family; local path migration logic remains current UX |
| `rust_core/yazelix_core/src/upgrade_summary.rs` | 512 | upgrade summary model/rendering | canonical | Current `whats_new` and first-run upgrade summary behavior |
| `rust_core/yazelix_core/src/workspace_asset_contract.rs` | 361 | workspace asset drift checks | canonical | Doctor/validator guardrail for generated runtime state |
| `rust_core/yazelix_core/src/workspace_commands.rs` | 1102 | `yzx cwd`, `yzx reveal`, popup helpers | canonical | Workspace/session integration owner |
| `rust_core/yazelix_core/src/workspace_session_contract.rs` | 166 | workspace session validator | canonical | Maintainer validator for cross-file workspace contracts |
| `rust_core/yazelix_core/src/yazi_materialization.rs` | 1221 | Yazi config generation | canonical | Large but cohesive generated Yazi owner; legacy override rejection is user-safety behavior |
| `rust_core/yazelix_core/src/yazi_render_plan.rs` | 283 | Yazi render plan | canonical | Small deterministic config-plan owner |
| `rust_core/yazelix_core/src/zellij_commands.rs` | 1302 | Zellij control commands | canonical | Owns pane-orchestrator CLI integration |
| `rust_core/yazelix_core/src/zellij_materialization.rs` | 1792 | Zellij config/layout generation | simplified | Kept as canonical; removed dead pane-orchestrator config entries for widget/sidebar data |
| `rust_core/yazelix_core/src/zellij_render_plan.rs` | 539 | Zellij render plan | canonical | Deterministic config/layout plan owner |

## Maintainer Tooling And Validators

| File | LOC | Owner | Disposition | Reason |
| --- | ---: | --- | --- | --- |
| `rust_core/yazelix_core/src/repo_contract_validation.rs` | 3547 | repo/package/release contract validators | split-candidate | Largest file; canonical maintainer behavior, but likely belongs in a separate maintainer crate or modules after `yazelix-9opk.5` |
| `rust_core/yazelix_core/src/repo_issue_sync.rs` | 670 | GitHub/Beads sync tooling | canonical-maintainer | Not user runtime behavior; keep local until maintainer crate/offload decision |
| `rust_core/yazelix_core/src/repo_nu_lint.rs` | 56 | Nushell lint helper | canonical-maintainer | Small validator helper |
| `rust_core/yazelix_core/src/repo_plugin_build.rs` | 476 | pane-orchestrator build/sync workflow | canonical-maintainer | Necessary guardrail for wasm/source freshness |
| `rust_core/yazelix_core/src/repo_sweep_runner.rs` | 1064 | configuration and visual sweep runner | canonical-maintainer | `run_visual_verification` is live visual validation, not demo-only code |
| `rust_core/yazelix_core/src/repo_test_runner.rs` | 591 | maintainer test orchestration | canonical-maintainer | Central maintainer gate |
| `rust_core/yazelix_core/src/repo_update_workflow.rs` | 1438 | maintainer update workflow | canonical-maintainer | Large process-heavy workflow; candidate for maintainer crate split, not deletion |
| `rust_core/yazelix_core/src/repo_validation.rs` | 1142 | generic repo validation helpers | canonical-maintainer | Test traceability and package-purity guardrails |
| `rust_core/yazelix_core/src/repo_version_bump.rs` | 469 | release bump tooling | canonical-maintainer | Maintainer release automation |

## Zellij Pane Orchestrator Plugin

| File | LOC | Owner | Disposition | Reason |
| --- | ---: | --- | --- | --- |
| `rust_plugins/zellij_pane_orchestrator/src/active_tab_session_state.rs` | 176 | active-tab session snapshot | canonical | Live session-state inspect and workspace routing data |
| `rust_plugins/zellij_pane_orchestrator/src/editor.rs` | 185 | managed editor actions | canonical | Plugin-side editor control |
| `rust_plugins/zellij_pane_orchestrator/src/horizontal_focus_contract.rs` | 243 | horizontal focus policy | canonical | Pure policy with tests |
| `rust_plugins/zellij_pane_orchestrator/src/layout.rs` | 239 | layout family and sidebar toggling | simplified | Removed unused override layout config structs; remaining code owns live swap-layout behavior |
| `rust_plugins/zellij_pane_orchestrator/src/lib.rs` | 5 | plugin library module exports | canonical | Exposes pure contract modules to tests |
| `rust_plugins/zellij_pane_orchestrator/src/main.rs` | 232 | Zellij plugin event/pipe dispatcher | simplified | Removed unused configuration parsing; remaining dispatcher is live |
| `rust_plugins/zellij_pane_orchestrator/src/pane_contract.rs` | 124 | pane identity policy | canonical | Pure pane classification contract |
| `rust_plugins/zellij_pane_orchestrator/src/panes.rs` | 599 | Zellij pane-manifest parsing | canonical | Live pane identity/focus/fallback owner |
| `rust_plugins/zellij_pane_orchestrator/src/sidebar_contract.rs` | 115 | sidebar toggle policy | canonical | Pure sidebar behavior contract |
| `rust_plugins/zellij_pane_orchestrator/src/sidebar_yazi.rs` | 118 | sidebar Yazi identity state | canonical | Live sidebar Yazi tracking |
| `rust_plugins/zellij_pane_orchestrator/src/transient.rs` | 298 | popup/transient panes | canonical | Live transient pane opening/toggling |
| `rust_plugins/zellij_pane_orchestrator/src/transient_pane_contract.rs` | 289 | transient pane policy | canonical | Pure transient pane command contract |
| `rust_plugins/zellij_pane_orchestrator/src/workspace.rs` | 277 | workspace state | canonical | Live workspace root/source state |

## Rust Tests

| File | LOC | Owner | Disposition | Reason |
| --- | ---: | --- | --- | --- |
| `rust_core/yazelix_core/tests/repo_upgrade_contract.rs` | 169 | upgrade-note contract tests | canonical | Defends release-note/changelog invariants and no-live-migration policy |
| `rust_core/yazelix_core/tests/yazi_render_plan_metadata_parity.rs` | 76 | config metadata parity test | canonical | Defends source-of-truth consistency for Yazi render metadata |
| `rust_core/yazelix_core/tests/yzx_control_front_door.rs` | 65 | front-door command tests | canonical | Defends `tutor` and `whats_new` behavior |
| `rust_core/yazelix_core/tests/yzx_control_public_commands.rs` | 115 | public command behavior tests | canonical | Defends `why`, `sponsor`, and `keys` outputs; not mere existence checks |
| `rust_core/yazelix_core/tests/yzx_control_runtime_surface.rs` | 509 | runtime public command tests | canonical | Defends `env`, `run`, `update`, status/runtime surfaces |
| `rust_core/yazelix_core/tests/yzx_control_workspace_surface.rs` | 532 | workspace public command tests | canonical | Defends `cwd`, `reveal`, Zellij workspace command behavior |
| `rust_core/yazelix_core/tests/yzx_core_command_metadata.rs` | 174 | command metadata and extern tests | canonical | Extern bridge remains canonical for Nushell shell integration |
| `rust_core/yazelix_core/tests/yzx_core_config_normalize.rs` | 1190 | broad machine helper contract tests | canonical | Large but high-value; candidate for later file split by family, not deletion |
| `rust_core/yazelix_core/tests/yzx_core_owned_facts.rs` | 111 | machine facts tests | canonical | Defends startup/integration/transient facts used by shell boundaries |
| `rust_core/yazelix_core/tests/yzx_core_runtime_env.rs` | 187 | runtime env tests | canonical | Defends deterministic env planning |
| `rust_core/yazelix_core/tests/yzx_core_yazi_materialization.rs` | 219 | Yazi materialization tests | canonical | Defends generated Yazi output and legacy override rejection |
| `rust_core/yazelix_core/tests/yzx_core_yazi_render_plan.rs` | 60 | Yazi render-plan helper tests | canonical | Defends success and error envelopes |
| `rust_core/yazelix_core/tests/yzx_core_zellij_render_plan.rs` | 82 | Zellij render-plan helper tests | canonical | Defends success and error envelopes |
| `rust_core/yazelix_core/tests/support/commands.rs` | 41 | integration-test command helpers | canonical | Shared test support |
| `rust_core/yazelix_core/tests/support/envelopes.rs` | 26 | integration-test envelope helpers | canonical | Shared test support |
| `rust_core/yazelix_core/tests/support/fixtures.rs` | 109 | integration-test fixtures | canonical | Shared test fixtures |
| `rust_core/yazelix_core/tests/support/mod.rs` | 5 | integration-test support module | canonical | Shared test support root |

## Migration-Era Surface Decisions

| Surface | Decision | Rationale |
| --- | --- | --- |
| `yzx_core` binary | Temporary canonical helper | Still called from startup/setup/sidebar Yazi/Menu/Home Manager/Helix/POSIX flows. Collapsing it now would recreate shell glue or destabilize active runtime paths |
| `nushell/scripts/utils/yzx_core_bridge.nu` consumers | Out of Rust scope for this pass | Rust code is canonical, but the Nu transport still exists at real shell/process boundaries. Collapse belongs to a separate Nu bridge-deletion bead |
| `command_metadata.rs` extern bridge | Canonical | Nushell needs extern definitions for external `yzx`; this is not just migration scaffolding |
| `internal_nu_runner.rs` | Temporary | Public Rust routing still deliberately delegates process-heavy `yzx dev` and remaining Nu-owned surfaces |
| Legacy root config and Yazi override rejection | Canonical user-safety guardrails | These fail fast instead of silently adopting stale user files |
| Legacy installer/wrapper diagnostics | Canonical doctor behavior | These still help users recover from older installs |
| Legacy popup-runner cleanup | Temporary safety cleanup | Low-cost stale-artifact cleanup; can be removed in a later major release if old runtimes are no longer supported |
| `migration_available` upgrade-note handling | Canonical historical renderer | Current releases should not use it, but historical upgrade notes still render accurately |
| `run_visual_verification` | Canonical maintainer visual sweep | It validates terminal/sweep behavior and is not a dead demo function |
| `record_demo` | Not present | No tracked Rust function or symbol with that name exists |

## Maintainer Tooling Boundary

`yazelix-9opk.5` decided that maintainer-only Rust tooling should stay in this repository but move out of the product runtime crate. The durable decision is recorded in `docs/rust_maintainer_tooling_boundary.md`.

Accepted target shape:

- keep `yazelix_core` as the product/runtime crate for shipped helpers and user-facing behavior
- add an in-repo `yazelix_maintainer` crate for `repo_*` modules and `yzx_repo_validator` / `yzx_repo_maintainer`
- reject a separate repository for now because validators, release tooling, sync stamps, Beads/GitHub state, and CI checks are tightly coupled to this checkout
- keep `workspace_asset_contract.rs` and `layout_family_contract.rs` in `yazelix_core` because `yzx doctor` uses them
- move `workspace_session_contract.rs` with maintainer validators unless a runtime caller appears

Implementation belongs in `yazelix-9opk.5.1`. The Rust ownership/LOC budget should wait until that crate split lands.

## Code Removed In This Pass

The pane orchestrator no longer parses or stores these unused plugin configuration values:

- `widget_tray_segment`
- `custom_text_segment`
- `sidebar_width_percent`

Those values are already owned by Rust Zellij materialization and rendered into generated Zellij config/layout files. The loaded plugin only needs `runtime_dir`, `popup_width_percent`, and `popup_height_percent`.

The pane-orchestrator package also now disables the native test harness for the plugin binary. That binary is a Zellij WASM host artifact, so package-level `cargo test` should run the pure Rust library contract tests instead of trying to link host-only Zellij imports.

## Test Audit Outcome

No Rust test file was deleted in this pass. The audited integration tests defend current behavior rather than pure command existence:

- public command outputs and error fallbacks
- machine-readable helper envelopes
- generated materialization behavior
- config and upgrade contract invariants
- workspace/session behavior
- metadata parity

The largest test file, `yzx_core_config_normalize.rs`, is a split candidate because it covers several machine helper families in one file. It is not a deletion candidate because its assertions defend current generated-state, runtime-materialization, config, and install-ownership contracts.

The pane-orchestrator test surface is library-owned. Its package manifest explicitly disables native bin tests because the plugin binary imports Zellij host functions that only exist under the WASM plugin runtime.
