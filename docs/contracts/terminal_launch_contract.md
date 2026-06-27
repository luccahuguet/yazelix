# Terminal Launch Contract

## Summary

Yazelix owns one packaged terminal launch path: Mars. Other terminal emulators are host-owned entrypoints and should start Yazelix with `yzx enter`.

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
- Statement: `yzx launch` builds the packaged Mars argv, applies the runtime graphics wrapper when required, and runs the detached-launch probe. It must not duplicate startup-launch preflight or terminal materialization ownership
- Verification: automated Rust tests in `rust_core/yazelix_core/src/launch_commands.rs`; validator `yzx_repo_validator validate-contracts`

#### TLAUNCH-002
- Type: failure_mode
- Status: live
- Owner: Rust `launch_commands/terminal.rs` and native config status
- Statement: `terminal.config_mode = "user"` must fail fast when Mars has no real user config at its native path, and Yazelix must never move, create, or take ownership of external terminal config implicitly
- Verification: automated Rust tests in `rust_core/yazelix_core/src/native_config_status.rs`; validator `yzx_repo_validator validate-config-surface-contract`

#### TLAUNCH-003
- Type: behavior
- Status: live
- Owner: Rust terminal materialization and Rust launch preflight
- Statement: Mars is the packaged terminal selected by `terminal = "mars"`, `#yazelix`, or `#yazelix_mars`. It consumes the Mars child package metadata from `passthru.marsPackageMetadata` and `share/mars/package-metadata.json`; missing or malformed metadata is a package error, not a fallback trigger. Generated Mars config is written under the Yazelix state directory and launched through `MARS_CONFIG_HOME`
- Verification: automated Rust tests in `rust_core/yazelix_core/src/launch_commands/launch.rs`; automated terminal-materialization tests in `rust_core/yazelix_core/src/terminal_materialization.rs`

#### TLAUNCH-004
- Type: boundary
- Status: live
- Owner: Rust `launch_commands/launch.rs`, Home Manager module, and flake package surface
- Statement: Non-Mars terminal emulators are host-owned entrypoints. Yazelix does not package Ghostty, Rio, WezTerm, Kitty, Foot, or Ratty runtime packages, and `yzx launch` does not accept `--term`. Users who prefer another terminal configure that terminal to run `yzx enter`
- Verification: automated Rust launch parsing tests; validator `validate-flake-interface`; validator `validate-nix-customization-api`

#### TLAUNCH-005
- Type: failure_mode
- Status: live
- Owner: Rust `launch_commands/process.rs` plus POSIX `shells/posix/detached_launch_probe.sh` and `shells/posix/desktop_deferred_launch_probe.sh`
- Statement: Detached launch must be measurable, fast on success, and visible on early terminal death with captured stderr instead of silently succeeding
- Verification: automated Rust tests in `rust_core/yazelix_core/src/launch_commands.rs`; validator `validate-installed-runtime-contract`

## Traceability

- Defended by: `yzx_repo_validator validate-contracts`
