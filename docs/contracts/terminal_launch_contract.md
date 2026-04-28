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
- `nushell/scripts/utils/constants.nu` terminal metadata used by launch
- `shells/posix/detached_launch_probe.sh`
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

## Traceability
- Defended by: `yzx_repo_validator validate-contracts`
