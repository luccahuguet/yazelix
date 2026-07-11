# Terminal Launch Contract

## Summary

Yazelix supports capable terminal emulators through `yzx enter` and packages
Kitty as the default managed launch terminal. Ghostty is the host-installed
backup. Kitty and Ghostty retain native terminal-config ownership: Yazelix
launches them with their built-in defaults or their existing user config.

## Scope

- `rust_core/yazelix_core/src/launch_commands/terminal.rs`
- `rust_core/yazelix_core/src/launch_commands/process.rs`
- `shells/posix/detached_launch_probe.sh`
- `shells/posix/desktop_deferred_launch_probe.sh`
- terminal launch and detached-launch tests in Rust launch-command modules

## Contract Items

#### TLAUNCH-001
- Type: boundary
- Status: live
- Owner: Rust `launch_commands/terminal.rs`, Rust `launch_commands/process.rs`, and POSIX `shells/posix/detached_launch_probe.sh`
- Statement: `yzx launch` builds the packaged Kitty argv, prepends the absolute runtime-owned Linux graphics wrapper with structured argv when required, and runs the detached-launch probe. It must not discover the wrapper from ambient `PATH`, assemble an inline shell body, or duplicate startup-launch preflight or terminal materialization ownership
- Verification: automated Rust tests in `rust_core/yazelix_core/src/launch_commands.rs`; validator `yzx_repo_validator validate-contracts`; manual review against `rust_nushell_bridge_contract.md`

#### TLAUNCH-002
- Type: failure_mode
- Status: live
- Owner: Rust `launch_commands/terminal.rs` and native config status
- Statement: Kitty and Ghostty use their built-in defaults when no native config exists and load an existing native config as user-owned state. Yazelix must never move, create, or take ownership of those external terminal configs implicitly
- Verification: automated Rust tests in `rust_core/yazelix_core/src/native_config_status.rs`; validator `yzx_repo_validator validate-config-surface-contract`

#### TLAUNCH-003
- Type: behavior
- Status: live
- Owner: Rust terminal materialization and Rust launch preflight
- Statement: Kitty is the packaged terminal selected by `#yazelix`, `#yazelix_kitty`, and the default profile/Home Manager package. Runtime metadata must identify `kitty`, and launch must resolve the runtime-owned Kitty command without entering the retained Mars-only config materializer
- Verification: automated Rust tests in `rust_core/yazelix_core/src/launch_commands/launch.rs`; automated terminal-materialization tests in `rust_core/yazelix_core/src/terminal_materialization.rs`

#### TLAUNCH-004
- Type: boundary
- Status: live
- Owner: Rust `launch_commands/launch.rs`, Home Manager module, and flake package surface
- Statement: Ghostty is the host-installed launch backup. Rio, WezTerm, Foot, Ratty, Mars, and other capable host terminals are supported through `yzx enter`, with their native config remaining user-owned
- Verification: automated Rust launch parsing tests; validator `validate-flake-interface`; validator `validate-nix-customization-api`

#### TLAUNCH-005
- Type: failure_mode
- Status: live
- Owner: Rust `launch_commands/process.rs` plus POSIX `shells/posix/detached_launch_probe.sh` and `shells/posix/desktop_deferred_launch_probe.sh`
- Statement: Detached launch must be measurable, fast on success, and visible on early terminal death with captured stderr instead of silently succeeding
- Verification: automated Rust tests in `rust_core/yazelix_core/src/launch_commands.rs`; validator `validate-installed-runtime-contract`

#### TLAUNCH-006
- Type: invariant
- Status: live
- Owner: Rust launch materialization, native config status, and doctor runtime reporting
- Statement: A Kitty runtime must not invoke the retained Mars-only terminal-config materializer, advertise a nonexistent generated Kitty config, or surface stale Mars launch logs as active-runtime evidence
- Verification: automated Rust tests in `launch_materialization.rs`, `native_config_status.rs`, and `doctor_runtime_report.rs`

## Traceability

- Defended by: `yzx_repo_validator validate-contracts`
