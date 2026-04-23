# Setup Shellhook Welcome Terminal Canonicalization Audit

## Summary

This audit reviews the surviving setup, shellhook, welcome, front-door rendering,
and terminal-launch support surfaces after the earlier `runtime-env.compute`
owner cut, the `yazelix-iwzn` initializer/shellhook audit, and the recent
launch-time bridge collapses.

The main conclusion is not "port setup to Rust." `setup/environment.nu`,
`setup/initializers.nu`, and the startup-profile transport remain honest
Nu/POSIX owners. The real remaining delete-first opportunity in this subsystem
is the front-door renderer stack around `setup/welcome.nu` and
`utils/ascii_art.nu`, plus a narrower terminal-launcher cleanup that should
delete stale metadata and dead exports without hiding the same shell command
matrix behind a fake Rust wrapper.

## 1. Subsystem Snapshot

- Subsystem: setup, shellhook, welcome, front-door rendering, and terminal launch support
- Purpose: bootstrap the managed shell environment, generate shell initializers,
  sync startup-owned bridges, render the welcome/front-door UX, and launch
  managed terminal windows through explicit host-terminal behavior
- User-visible entrypoints:
  - current-shell entry through `shells/posix/start_yazelix.sh`
  - `yzx env`
  - `yzx enter`
  - `yzx launch`
  - `yzx desktop launch`
  - `yzx screen`
  - managed Nushell startup through `nushell/config/config.nu`
- Primary source paths:
  - `nushell/scripts/setup/environment.nu`
  - `nushell/scripts/setup/initializers.nu`
  - `nushell/scripts/setup/welcome.nu`
  - `nushell/scripts/utils/shell_user_hooks.nu`
  - `nushell/scripts/utils/terminal_launcher.nu`
  - `nushell/scripts/utils/ascii_art.nu`
  - `nushell/scripts/utils/startup_profile.nu`
  - `shells/posix/start_yazelix.sh`
  - `shells/posix/runtime_env.sh`
  - `shells/posix/detached_launch_probe.sh`
- External dependencies that matter:
  - `bash`, POSIX shell, and host terminal emulators
  - `starship`, `zoxide`, `atuin`, `mise`, and `carapace`
  - `macchina` for optional welcome machine info
  - terminal size and keypress support from the active terminal

## 2. Must-Not-Lose Behavior

| Behavior | Current contract or source | Current owner | Current verification | Candidate surviving owner |
| --- | --- | --- | --- | --- |
| Runtime setup generates shell initializers and startup-owned bridges under the Yazelix state root without rewriting host shell files or taking ownership of Home Manager symlinks | `docs/specs/runtime_root_contract.md`; `docs/initializer_scripts.md`; `docs/posix_xdg.md`; `docs/specs/rust_migration_matrix.md` | Nu `setup/environment.nu`; Nu `setup/initializers.nu`; Nu `shell_user_hooks.nu`; Rust-generated extern bridge lifecycle | `test_runtime_setup_leaves_existing_host_shell_surfaces_untouched`; `test_runtime_setup_ignores_read_only_host_shell_surfaces`; `test_managed_nushell_config_sources_optional_user_hook`; `test_managed_nushell_config_loads_in_repo_shell_without_runtime_env`; `test_yzx_extern_bridge_reuses_current_fingerprint` in `nushell/scripts/dev/test_shell_managed_config_contracts.nu` | same split, with no new host-surface takeover |
| Startup bootstrap exports explicit runtime, config, state, and logs roots before entering the live shell and keeps shellhook setup measurable | `docs/specs/runtime_root_contract.md`; `docs/specs/v15_trimmed_runtime_contract.md`; `docs/specs/startup_profile_scenarios.md` | POSIX `runtime_env.sh`; POSIX `start_yazelix.sh`; Nu `setup/environment.nu`; Nu `startup_profile.nu` | Rust workspace/runtime surface tests; startup-profile harness checks; `yzx_repo_validator validate-installed-runtime-contract` | same split |
| Terminal launch keeps explicit `terminal.config_mode` behavior, does not implicitly take over external user config files, and surfaces detached-launch failures with real stderr | `docs/specs/runtime_dependency_preflight_contract.md`; `docs/specs/runtime_root_contract.md`; `docs/specs/launch_startup_session_canonicalization_audit.md` | Nu `terminal_launcher.nu`; POSIX `detached_launch_probe.sh`; Nu `launch_yazelix.nu` | `test_generate_all_terminal_configs_keeps_terminal_overrides_opt_in` in `nushell/scripts/dev/test_yzx_generated_configs.nu`; `test_detached_launch_probe_success_path_is_fast`; `test_detached_launch_probe_early_failure_is_visible`; `test_startup_profile_records_detached_terminal_probe` in `nushell/scripts/dev/test_yzx_maintainer.nu` | same shell/posix split, but with smaller metadata ownership |
| Welcome and front-door UX stay width-aware, interruptible, and explicit about skip behavior, and `yzx screen` keeps a live animated screen path | `docs/specs/config_surface_backend_dependence_matrix.md`; `docs/yzx_cli.md`; `yazelix_default.toml`; `config_metadata/main_config_contract.toml` | Nu `setup/welcome.nu`; Nu `utils/ascii_art.nu`; Nu `yzx/screen.nu`; startup callers | `test_screen_style_rejects_static`; `test_game_of_life_screen_cycle_stays_bounded_and_omits_resting_logo`; `test_game_of_life_screen_state_rolls_forward` in `nushell/scripts/dev/test_yzx_screen_commands.nu`; welcome skip semantics currently only indirectly defended by startup tests | smaller canonical renderer plus shell-owned playback/waiting boundary |
| Startup profiling stays one JSONL schema and continues to record shellhook, detached-launch, and materialization boundaries as first-class steps | `docs/specs/startup_profile_scenarios.md` | Nu `utils/startup_profile.nu` plus caller-local instrumentation | `test_startup_profile_report_schema_is_structured_and_summarizable`; `test_startup_profile_records_detached_terminal_probe`; `test_startup_profile_harness_records_real_startup_boundaries`; `test_startup_profile_materialization_reports_generated_runtime_substeps` in `nushell/scripts/dev/test_yzx_maintainer.nu` | same Nu owner unless a future port deletes it end to end |

## 3. Canonical Owner Map

| Concern | Current owner or split boundary | Split kind | Audit judgment |
| --- | --- | --- | --- |
| User-visible shellhook/setup behavior | Nu `setup/environment.nu`; Nu `setup/initializers.nu`; POSIX bootstrap wrappers | intentional | This is still the real shell/process boundary |
| Typed or deterministic setup logic | Rust `runtime-env.compute`; Rust command-metadata extern sync; Nu `startup_profile.nu`; Nu `ascii_art.nu` simulation/render logic | mixed | `runtime-env.compute` already took the honest Rust slice; `startup_profile.nu` remains a coherent Nu owner; `ascii_art.nu` is the main oversized remaining deterministic surface |
| Generated-state writes | Nu initializer generation; Nu user-hook bridge; Rust extern bridge file generation; Nu startup-profile JSONL writes | intentional with small transport seams | The writes are still bound to startup and shell activation rather than a broader Rust domain |
| Shell or process orchestration | Nu `environment.nu`; Nu `terminal_launcher.nu`; POSIX `start_yazelix.sh`; POSIX `detached_launch_probe.sh` | intentional | Keep the orchestration boundary explicit and shell-owned |
| Final human-facing rendering | Nu `setup/welcome.nu`; Nu `utils/ascii_art.nu`; Nu `yzx/screen.nu` | accidental duplication / historical debt | The product surface is real, but the current owner shape is too broad and under-specified |
| Live session or plugin state | process-local keypress/terminal size state only | intentional | No plugin/state-owner cut is hiding here |

## 4. Helper Classification

| Helper or surface | Current class | Why |
| --- | --- | --- |
| `setup/environment.nu` | `shell survivor` | Owns shellhook orchestration, state/log setup, bridge sync, executable-bit repair, and welcome gating |
| `setup/initializers.nu` | `shell survivor` | Generates shell-specific initializer text by calling external tools and writing shell-owned init files |
| `utils/shell_user_hooks.nu` | `shell survivor` | Small transport-only bridge for user-managed Nushell hook sourcing under Yazelix-owned paths |
| `utils/startup_profile.nu` | `shell survivor` | Canonical JSONL profiling schema and shell-owned timing transport |
| `utils/terminal_launcher.nu` | `shell survivor` with bridge debt | Real launch transport still belongs here, but stale metadata ownership and dead exports remain |
| `setup/welcome.nu` | `shell survivor` with bridge debt | Interactive display/waiting is shell-bound, but message assembly and product-surface ownership are too loose |
| `utils/ascii_art.nu` | `Rust-port target` / data-split candidate | 1004 LOC concentrated renderer/simulation logic with weak direct contracts and likely stale style surface |
| `constants.nu` welcome/terminal subsets | `data-only survivor` | Configurable style and terminal metadata should be declarative, not re-decided ad hoc in multiple owners |
| `config_metadata/main_config_contract.toml`, `yazelix_default.toml`, `home_manager/module.nix` welcome-style fields | `data-only survivor` | These already define the public config surface and must stay aligned with the real runtime semantics |

## 5. Survivor Reasons

- `setup/environment.nu`: `irreducible_shell_boundary`
- `setup/initializers.nu`: `external_tool_adapter`
- `utils/shell_user_hooks.nu`: `transport_only`
- `utils/startup_profile.nu`: `canonical_owner`
- `utils/terminal_launcher.nu` transport boundary: `external_tool_adapter`
- `shells/posix/start_yazelix.sh`, `shells/posix/runtime_env.sh`, `shells/posix/detached_launch_probe.sh`: `external_tool_adapter`
- `setup/welcome.nu` display/wait boundary: `irreducible_shell_boundary`
- `setup/welcome.nu` message-building surface: `temporary_bridge_debt`
- `utils/ascii_art.nu` as a 1004-line front-door renderer: `historical_debt`

## 6. Delete-First Findings

### Delete Now

- `utils/ascii_art.nu::get_animated_ascii_art` is exported but only used inside `ascii_art.nu`
- `utils/terminal_launcher.nu::resolve_nixgl_launch_context` is exported but has no external callers
- stale docs/comments around `welcome_style = "random"` should be rewritten or the semantics restored; the current wording is broader than the live code

### Bridge Layer To Collapse

- `setup/welcome.nu` reparses config through `get_session_info` and `get_terminal_info` even though both startup callers already parsed the main config
- welcome and `yzx screen` style ownership is spread across `welcome.nu`, `ascii_art.nu`, `yzx/screen.nu`, and config docs without one explicit live style contract
- `terminal_launcher.nu` still braids config-mode policy, host-terminal detection, nixGL wrapper choice, per-terminal command shaping, and metadata lookup in one file even after `yazelix-p18h`

### Full-Owner Migration

- the front-door renderer stack is the honest remaining big deletion lane in this subsystem
- the first step is not "port ascii_art.nu to Rust"; the first step is to define which welcome and screen styles actually survive, what `random` means, and whether static frame data or stale styles can simply be deleted
- after that decision, the surviving renderer can move to a smaller canonical owner, likely a mix of static data plus a smaller runtime renderer, or a Rust owner only if it deletes the oversized Nu owner end to end

### Likely Survivors

- shellhook orchestration in `setup/environment.nu`
- external-tool initializer generation in `setup/initializers.nu`
- the user-hook bridge in `shell_user_hooks.nu`
- startup-profile transport in `startup_profile.nu`
- terminal-launch transport plus POSIX detachment helpers
- a small shell-owned welcome playback/waiting layer

### No-Go Deletions

- broad Rust port of `setup/environment.nu` or `setup/initializers.nu`
  - stop condition: only reopen if a future move deletes one of those owners substantially end to end instead of adding another bridge helper
- broad Rust port of terminal launch execution
  - stop condition: only reopen if the work deletes the Nu/POSIX launch owner instead of hiding the same shell command matrix behind Rust
- deleting `yzx screen`, logo/boids/the Game of Life variants, or the current welcome UX without an explicit retained-style decision
  - stop condition: decide the real product surface first, then delete stale styles honestly

## 7. Quality Findings

- Duplicate owners:
  - config is parsed once by `environment.nu` and `start_yazelix_inner.nu`, then reparsed inside `setup/welcome.nu`
  - welcome-style public surface is defined in config metadata and defaults, but the actual runtime random pool lives separately in `ascii_art.nu`
- Missing layer problems:
  - there is no live spec that says which welcome styles and `yzx screen` styles are canonical and what `random` means
  - there is no direct executable defense for welcome message composition, welcome skip/logging semantics, or `welcome_style = random`
- Extra layer problems:
  - `setup/welcome.nu` mixes product-copy assembly with terminal UI control
  - `utils/ascii_art.nu` mixes style policy, data/spec tables, animation playback, terminal probing, and simulation in one large file
  - `utils/terminal_launcher.nu` mixes detection, config resolution, metadata, command assembly, and detached transport
- DRY opportunities:
  - pass parsed config or precomputed welcome facts into `build_welcome_message` instead of reparsing config
  - narrow terminal-launch metadata into one canonical data surface rather than keeping ad hoc tables and dead exports in the launcher module
- Weak or orphan tests:
  - `test_yzx_screen_commands.nu` now defends the public Game of Life variants directly, but there is still no equally direct coverage for `logo`, `boids`, or welcome skip/message composition
  - most setup/welcome coverage still points at broad governance surfaces instead of a focused front-door contract
- Only-known executable-defense tests:
  - `test_runtime_setup_leaves_existing_host_shell_surfaces_untouched`
  - `test_runtime_setup_ignores_read_only_host_shell_surfaces`
  - `test_startup_profile_records_detached_terminal_probe`
  - `test_detached_launch_probe_success_path_is_fast`
  - `test_detached_launch_probe_early_failure_is_visible`
  - `test_startup_profile_harness_records_real_startup_boundaries`
  - `test_game_of_life_screen_cycle_stays_bounded_and_omits_resting_logo`
  - `test_game_of_life_screen_state_rolls_forward`
- Spec gaps:
  - no dedicated live welcome/screen style contract
  - no live contract for welcome skip/logging/message composition
- Docs drift:
  - `random` now maps to the three explicit Game of Life variants, but there is still no dedicated live spec that makes that retained pool a governed contract

## 8. Deletion Classes And Follow-Up Beads

- `yazelix-7krc`
  - retained behavior: width-aware welcome and `yzx screen`, interactive skip, and no silent feature loss
  - deletion class: `full_owner_migration`
  - candidate surviving owner: smaller front-door renderer stack with explicit retained styles, likely static data plus a smaller runtime renderer
  - verification that must still pass: `test_yzx_screen_commands.nu`, new direct welcome contract tests, `yzx_repo_validator validate-specs`
  - stop condition: do not port the entire current Nu renderer into Rust unless the Nu owner actually shrinks materially
- `yazelix-7krc.1`
  - retained behavior: style surface, random semantics, width-aware rendering, skip behavior
  - deletion class: `full_owner_migration`
  - candidate surviving owner: explicit welcome/screen style contract
  - verification that must still pass: `test_yzx_screen_commands.nu`, config-surface/spec validation
  - stop condition: if a style has no defended product value, delete it instead of preserving it as an alias
- `yazelix-7krc.2`
  - retained behavior: the post-decision surviving front-door renderer
  - deletion class: `full_owner_migration`
  - candidate surviving owner: smaller renderer/data owner than the current `ascii_art.nu` plus `welcome.nu`
  - verification that must still pass: direct welcome/screen tests plus `yzx_repo_validator validate-specs`
  - stop condition: no parallel renderer stacks
- `yazelix-zlt2`
  - retained behavior: explicit host terminal launch behavior, detached-launch visibility, `config_mode=user` non-takeover rule
  - deletion class: `bridge_collapse`
  - candidate surviving owner: Nu/POSIX transport boundary plus a smaller canonical launch-metadata surface
  - verification that must still pass: Rust generated-config/materialization tests, detached-launch tests, `yzx_repo_validator validate-installed-runtime-contract`
  - stop condition: no fake Rust launch wrapper
- `yazelix-zlt2.1`
  - retained behavior: platform-specific launch flags, nixGL selection, config-mode guardrails
  - deletion class: `bridge_collapse`
  - candidate surviving owner: explicit launch contract plus one canonical metadata owner
  - verification that must still pass: launch command and detached-launch regressions
  - stop condition: if the data owner is unclear, stop before editing launch behavior
- `yazelix-zlt2.2`
  - retained behavior: current launch behavior after stale metadata owners are removed
  - deletion class: `bridge_collapse`
  - candidate surviving owner: smaller `terminal_launcher.nu` plus POSIX helpers
  - verification that must still pass: terminal generation/launch regressions and installed-runtime validation
  - stop condition: no compatibility aliases that preserve dead metadata owners

## Verification

- manual review of:
  - `nushell/scripts/setup/environment.nu`
  - `nushell/scripts/setup/initializers.nu`
  - `nushell/scripts/setup/welcome.nu`
  - `nushell/scripts/utils/shell_user_hooks.nu`
  - `nushell/scripts/utils/terminal_launcher.nu`
  - `nushell/scripts/utils/ascii_art.nu`
  - `nushell/scripts/utils/startup_profile.nu`
  - `nushell/scripts/yzx/screen.nu`
  - `docs/specs/startup_profile_scenarios.md`
  - `docs/specs/runtime_root_contract.md`
  - `docs/specs/runtime_dependency_preflight_contract.md`
  - `docs/specs/rust_migration_matrix.md`
- `yzx_repo_validator validate-specs`

## Traceability

- Bead: `yazelix-rdn7.5.5`
- Defended by: `yzx_repo_validator validate-specs`
- Informed by: `docs/specs/runtime_root_contract.md`
- Informed by: `docs/specs/runtime_dependency_preflight_contract.md`
- Informed by: `docs/specs/startup_profile_scenarios.md`
- Informed by: `docs/specs/rust_migration_matrix.md`
