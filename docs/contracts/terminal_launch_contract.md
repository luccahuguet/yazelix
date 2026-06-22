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
  `yazelix_ghostty.sh` environment wrapper. Ghostty launch argv and generated
  Ghostty config do not set a fixed title, because Ghostty treats configured
  titles as authoritative and would ignore the Zellij-owned title escape
  sequence that carries the live session name.
- Verification: automated
  `nushell/scripts/dev/test_yzx_generated_configs.nu`
  (`test_ghostty_linux_launch_command_keeps_linux_specific_flags`,
  `test_ghostty_macos_launch_command_omits_linux_specific_flags`); automated
  Rust tests in `rust_core/yazelix_core/src/launch_commands.rs`
  (`ghostty_launch_does_not_force_window_title`) and
  `rust_core/yazelix_core/src/launch_materialization.rs`
  (`full_launch_materialization_uses_active_terminal_from_request`)
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
  `ratty_launch_command_prefers_runtime_vulkan_wrapper`)
- Source: `docs/installation.md`; `docs/terminal_emulators.md`

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
  (`foot_launch_argv_uses_selected_config_and_working_dir`)

#### TLAUNCH-009
- Type: behavior
- Status: live
- Owner: Rust terminal materialization and Rust launch preflight
- Statement: Rio is the upstream packaged terminal variant selected by
  `terminal = "rio"` or `#yazelix_rio`; it must not depend on the
  private terminal child package or private terminal metadata. Generated Rio config is
  written under the Yazelix state directory and launched through
  `RIO_CONFIG_HOME`, with `terminal.transparency` mapped to Rio window and cell
  opacity. On Linux, when transparency is enabled and an X display exists,
  Yazelix sets `WINIT_UNIX_BACKEND=x11` and clears `WAYLAND_DISPLAY` for the Rio
  process so upstream Rio uses XWayland; this works around upstream Rio issue
  https://github.com/raphamorim/rio/issues/1644 where COSMIC/Wayland ignores
  background opacity. If transparency is `none`, or no X display is available,
  Yazelix leaves Rio's backend selection untouched.
- Verification: automated Rust tests in
  `rust_core/yazelix_core/src/launch_commands/launch.rs`
  (`rio_process_boundary_env_points_at_selected_config_dir`,
  `rio_process_boundary_env_forces_x11_for_transparent_linux_launches`,
  `rio_process_boundary_env_keeps_default_backend_without_x11_display`,
  `rio_launch_argv_uses_selected_config_and_working_dir`)

#### TLAUNCH-010
- Type: behavior
- Status: live
- Owner: Rust terminal materialization and Rust launch preflight
- Statement: Mars is the default packaged terminal variant selected by
  `terminal = "mars"`, `#yazelix`, or `#yazelix_mars`. It consumes the Mars
  child package metadata from `passthru.marsPackageMetadata` and
  `share/mars/package-metadata.json`; missing or malformed metadata is a
  package error, not a fallback trigger. Generated Mars config is written under
  the Yazelix state directory, launched through `MARS_CONFIG_HOME`, and scoped
  to the terminal process boundary so host Rio does not inherit Mars config
  state.
  On Linux, packaged Mars can use the runtime-owned Vulkan wrapper because the
  renderer needs a Vulkan-capable adapter.
- Verification: automated Rust tests in
  `rust_core/yazelix_core/src/launch_commands/launch.rs`
  (`mars_process_boundary_env_clears_host_rio_config_and_sets_app_id`);
  automated terminal-materialization tests in
  `rust_core/yazelix_core/src/terminal_materialization.rs`
  (`mars_serenityos_config_uses_child_emoji_profile_root`)

## Traceability
- Defended by: `yzx_repo_validator validate-contracts`
