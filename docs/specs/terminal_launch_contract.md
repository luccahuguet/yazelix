# Terminal Launch Contract

## Summary

This spec defines the retained terminal-launch contract and the deletion budget
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
  `nu nushell/scripts/dev/validate_specs.nu`
- Source: `docs/specs/setup_shellhook_welcome_terminal_canonicalization_audit.md`;
  `docs/specs/launch_startup_session_canonicalization_audit.md`

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
  validator `nu nushell/scripts/dev/validate_config_surface_contract.nu`
- Source: `docs/specs/terminal_override_layers.md`;
  `docs/specs/runtime_dependency_preflight_contract.md`

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
- Source: `docs/specs/runtime_dependency_preflight_contract.md`

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
  validator `nu nushell/scripts/dev/validate_installed_runtime_contract.nu`
- Source: `docs/specs/startup_profile_scenarios.md`

## Deletion Budget For `yazelix-zlt2.2`

`yazelix-zlt2.2` counts as success only if `terminal_launcher.nu` becomes
materially narrower without moving the same shell command matrix behind a fake
Rust wrapper.

### Must delete or demote from `terminal_launcher.nu`

- exported `resolve_nixgl_launch_context`
  - no external callers exist
  - the surviving helper should be private to launch-command construction
- duplicate terminal discovery helpers:
  - `detect_terminal_candidates`
  - `detect_terminal`
  - these duplicate Rust `startup-launch-preflight.evaluate`
- one-caller display wrapper:
  - `get_terminal_display_name`
  - `launch_yazelix.nu` can read the preflight-provided `name` field directly
- `TERMINAL_METADATA` import inside `terminal_launcher.nu`
  - launch titles can use the preflight candidate name already passed in
  - terminal display-name metadata should not be a second owner inside the
    launcher module

### May survive

- `command_exists`
  - still used by sweep tooling as a small PATH probe
- `resolve_terminal_config`
  - still owns the `terminal.config_mode` launch-time fail-fast boundary
- `build_launch_command`
  - still owns platform-specific host-terminal command construction
- `run_detached_terminal_launch`
  - still owns the POSIX detached-launch probe boundary

### Stop conditions

`yazelix-zlt2.2` must stop and record a no-go if any of these are true:

- removing duplicate terminal discovery would require reimplementing preflight
  logic in another Nushell helper
- deleting `get_terminal_display_name` would lose user-visible terminal names
  for failure messages
- shrinking `terminal_launcher.nu` would require implicit takeover of external
  user terminal configs
- a Rust wrapper would be added while Nushell still owns the actual command
  matrix

## Verification Gate

Before `yazelix-zlt2.2` closes, run:

- `nu nushell/scripts/dev/validate_syntax.nu`
- `nu nushell/scripts/dev/test_yzx_generated_configs.nu`
- targeted detached-launch tests from
  `nushell/scripts/dev/test_yzx_maintainer.nu`
- `nu nushell/scripts/dev/validate_installed_runtime_contract.nu`

If the full generated-config suite is too expensive for a small edit, at
minimum run the terminal-launch tests it contains and record the skipped scope
in the bead notes.

## Traceability

- Bead: `yazelix-zlt2.1`
- Informed by: `docs/specs/setup_shellhook_welcome_terminal_canonicalization_audit.md`
- Informed by: `docs/specs/launch_startup_session_canonicalization_audit.md`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
