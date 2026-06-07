# Terminal Launch Contract

## Summary

This contract defines the retained terminal-launch contract and the deletion budget
for shrinking `nushell/scripts/utils/terminal_launcher.nu`. The goal is not to
hide launch behavior behind Rust. Terminal launch remains a shell/process
boundary because Yazelix must invoke host terminal binaries, preserve explicit
platform flags, route through `nixGL` when needed, detach through a checked-in
POSIX helper, and surface early terminal death with real stderr.

The delete-first target is the stale metadata and duplicate owner logic that
survived after Rust took over startup-launch preflight and terminal
materialization.

## Scope

- `nushell/scripts/utils/terminal_launcher.nu`
- `nushell/scripts/core/launch_yazelix.nu`
- `shells/posix/detached_launch_probe.sh`
- `shells/posix/desktop_deferred_launch_probe.sh`
- terminal launch and detached-launch tests in
  `nushell/scripts/dev/test_yzx_generated_configs.nu`,
  `nushell/scripts/dev/test_yzx_maintainer.nu`, and
  `nushell/scripts/dev/test_yzx_workspace_commands.nu`

Out of scope:

- a Rust terminal-launch wrapper that still shells out through the same command
  matrix
- moving `terminal.config_mode = user` into implicit config takeover
- changing generated terminal config materialization

## Contract Items

#### TLAUNCH-001
- Type: boundary
- Status: live
- Owner: Nushell `terminal_launcher.nu` and POSIX
  `shells/posix/detached_launch_probe.sh`
- Statement: Terminal launch is an explicit shell/process boundary. The
  surviving owner may build host-terminal argv strings, choose platform flags,
  apply the runtime `nixGL` prefix, and run the detached-launch probe, but it
  must not duplicate startup-launch preflight or terminal materialization
  ownership
- Verification: automated
  `nushell/scripts/dev/test_yzx_generated_configs.nu`
  (`test_managed_wrapper_launch_command_does_not_forward_config_mode_flag`,
  `test_ghostty_linux_launch_command_keeps_linux_specific_flags`,
  `test_ghostty_macos_launch_command_omits_linux_specific_flags`); automated
  `nushell/scripts/dev/test_yzx_maintainer.nu`
  (`test_detached_launch_probe_success_path_is_fast`,
  `test_detached_launch_probe_early_failure_is_visible`); validator
  `yzx_repo_validator validate-contracts`

#### TLAUNCH-002
- Type: failure_mode
- Status: live
- Owner: `terminal_launcher.nu`
- Statement: `terminal.config_mode = user` must fail fast when the selected
  terminal has no real user config at its native path, and Yazelix must never
  move, create, or take ownership of that external config implicitly
- Verification: automated
  `nushell/scripts/dev/test_yzx_generated_configs.nu`
  (`test_terminal_config_mode_user_requires_real_user_config`,
  `test_generate_all_terminal_configs_keeps_terminal_overrides_opt_in`);
  validator `yzx_repo_validator validate-config-surface-contract`
- Source: `docs/contracts/terminal_override_layers.md`;
  `docs/contracts/runtime_dependency_preflight_contract.md`

#### TLAUNCH-003
- Type: behavior
- Status: live
- Owner: `terminal_launcher.nu`
- Statement: Ghostty launch keeps Linux-only GTK/X11 flags on Linux, omits
  those flags on macOS, and routes Linux Ghostty through the runtime
  `yazelix_ghostty.sh` environment wrapper
- Verification: automated
  `nushell/scripts/dev/test_yzx_generated_configs.nu`
  (`test_ghostty_linux_launch_command_keeps_linux_specific_flags`,
  `test_ghostty_macos_launch_command_omits_linux_specific_flags`)
- Source: `docs/contracts/runtime_dependency_preflight_contract.md`

#### TLAUNCH-004
- Type: failure_mode
- Status: live
- Owner: `terminal_launcher.nu` plus
  `shells/posix/detached_launch_probe.sh`
- Statement: Detached launch must be measurable, fast on success, and visible
  on early terminal death with captured stderr instead of silently succeeding
- Verification: automated
  `nushell/scripts/dev/test_yzx_maintainer.nu`
  (`test_startup_profile_records_detached_terminal_probe`,
  `test_detached_launch_probe_success_path_is_fast`,
  `test_detached_launch_probe_early_failure_is_visible`);
  validator `cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_maintainer --bin yzx_repo_validator -- validate-installed-runtime-contract`
- Source: `docs/contracts/startup_profile_scenarios.md`

#### TLAUNCH-005
- Type: behavior
- Status: live
- Owner: `terminal_launcher.nu` and Rust terminal materialization
- Statement: Ratty launch uses a generated `ratty.toml` config, passes it with
  `--config-file`, sets the Yazelix window title, and keeps Ratty's `-e`
  command delimiter as the final flag before the startup script. Generated
  Ratty config keeps Ratty's visible model cursor and animation enabled. On
  Linux, packaged Ratty launches prefer the runtime-owned nixGL Vulkan wrapper
  because Ratty's Bevy/wgpu renderer requires a Vulkan-capable adapter.
- Verification: automated Rust tests in
  `rust_core/yazelix_core/src/launch_commands.rs`
  (`ratty_launch_command_keeps_command_last`,
  `ratty_launch_command_prefers_runtime_vulkan_wrapper`) and
  `rust_core/yazelix_core/tests/yzx_core_config_normalize.rs`
  (`terminal_materialization_generate_from_env_writes_generated_configs`)
- Source: `docs/installation.md`; `docs/terminal_emulators.md`

#### TLAUNCH-006
- Type: behavior
- Status: live
- Owner: Rust terminal materialization, Rust launch preflight, and the
  `yazelix-terminal` child wrapper
- Statement: Yazelix Terminal launch uses the config id
  `yzxterm`, resolves the executable command as the child-owned
  `yazelix-terminal-desktop` wrapper, passes the generated config directory
  with `YAZELIX_TERMINAL_CONFIG`, clears ambient `RIO_CONFIG_HOME` at that
  process boundary, marks the child environment for Yazelix Terminal
  sanitization, passes `YAZELIX_TERMINAL_APP_ID` so the terminal window matches
  the integrated Yazelix desktop entry, and does not add an outer Yazelix
  graphics wrapper around the child wrapper. Generated Yazelix Terminal config
  is derived from the packaged
  child profile selected by `YAZELIX_TERMINAL_PROFILE` or
  `YAZELIX_TERMINAL_EFFECTS`: `full` keeps Rio trail cursor and strips packaged
  `custom-shader` entries, `baseline` uses the packaged no-effects profile, and
  `shaders` uses the packaged shader profile while replacing packaged shader
  references with the generated Rio decoration shader for the active cursor
  settings. Shader-profile launches use a launch-scoped generated config and
  shader directory under `terminal_launches/<launch-id>/`, while full and
  baseline profiles use the stable generated config root. Launch-scoped shader
  snapshots are retained as ordinary Yazelix state for the user session and may
  be pruned by future maintenance; running terminals must not depend on a
  mutable shared shader directory. The generated config injects the current
  `terminal.transparency` as `[window].opacity` with cell opacity enabled
  whenever transparency is not `none`. The generated yzxterm config is
  Yazelix-owned state; it must not become the host Rio config for plain `rio`
  launches.
- Verification: automated Rust tests in
  `rust_core/yazelix_core/src/runtime_contract.rs`
  (`launch_preflight_maps_yzxterm_to_child_wrapper_command`),
  `rust_core/yazelix_core/src/launch_commands.rs`
  (`yzxterm_launch_command_uses_child_wrapper_without_outer_graphics_wrapper`),
  `rust_core/yazelix_core/src/launch_commands/launch.rs`
  (`yzxterm_process_boundary_env_clears_host_rio_config`),
  `rust_core/yazelix_core/src/launch_materialization.rs`
  (`yzxterm_shader_profile_uses_scoped_terminal_state_dir`),
  and `rust_core/yazelix_core/tests/yzx_core_config_normalize.rs`
  (`terminal_materialization_generate_from_env_writes_generated_configs`,
  `terminal_materialization_yzxterm_shader_profile_injects_rio_decoration_shader`)
- Source: `docs/installation.md`; `docs/terminal_emulators.md`

#### TLAUNCH-007
- Type: behavior
- Status: live
- Owner: Rust desktop launch plus
  `shells/posix/desktop_deferred_launch_probe.sh`; inner child-process PID
  evidence beyond the terminal process belongs to the `yazelix-terminal` child
  wrapper
- Statement: Desktop-deferred Yazelix Terminal launches write bounded per-launch
  logs under `YAZELIX_STATE_DIR/logs/terminal_launch`. The log name is based on
  the executable basename, so yzxterm logs use
  `yazelix_terminal_desktop_*.log`. Each fresh log records timestamps, argv,
  config environment, helper PID, terminal-or-wrapper PID, captured
  stdout/stderr, any early exit status observable by the desktop helper, and
  terminal lifetime evidence. After the short startup probe, the detached helper
  records `lifetime_status=watching`, waits for the terminal-or-wrapper PID,
  and appends `final_exit_status` plus `final_exit_kind=exit`/`final_exit_code`
  or `final_exit_kind=signal`/`final_signal`. The main runtime records
  `child_pid=not_observable_by_desktop_probe` when the child-owned wrapper is
  the only process boundary that can observe the inner Rio child PID. Doctor
  reports final lifetime evidence, active lifetime watchers, metadata-only
  logs, stale/missing metadata, or no captured launch evidence for active
  yzxterm runtimes without warning unrelated terminal variants. A log ending
  after short-probe metadata such as
  `exit_status=not_observed_after_probe_window` is not sufficient crash
  observability.
- Verification: automated Rust tests in
  `rust_core/yazelix_core/src/launch_commands.rs`
  (`desktop_deferred_launch_helper_records_lifetime_status`,
  `launch_probe_log_path_uses_command_basename`) and
  `rust_core/yazelix_core/src/doctor_runtime_report.rs`
  (`yzxterm_launch_log_finding_reports_lifetime_logs`,
  `yzxterm_launch_log_finding_warns_on_metadata_only_logs`,
  `yzxterm_launch_log_finding_is_scoped_to_yzxterm_runtime`)

#### TLAUNCH-008
- Type: behavior
- Status: live
- Owner: Rust terminal materialization and Rust launch preflight
- Statement: Foot is a Linux-only packaged terminal variant. Launch uses the
  native `foot` command with generated `foot.ini`, passes the application id,
  window title, working directory, and startup command through Foot's native CLI
  boundary, and does not fall back to another terminal when selected. Generated
  Foot config maps Yazelix transparency into Foot color alpha.
- Verification: automated Rust tests in
  `rust_core/yazelix_core/src/launch_commands/launch.rs`
  (`foot_launch_argv_uses_selected_config_and_working_dir`) and
  `rust_core/yazelix_core/tests/yzx_core_config_normalize.rs`
  (`terminal_materialization_foot_uses_foot_ini`)

## Traceability
- Defended by: `yzx_repo_validator validate-contracts`
