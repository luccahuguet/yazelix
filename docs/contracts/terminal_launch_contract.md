# Terminal Launch Contract

## Summary

Yazelix packages and launches Mars. Other capable terminal emulators remain host-owned entrypoints that start the workspace with `yzx enter`.

## Contract Items

#### TLAUNCH-001
- Type: boundary
- Status: live
- Owner: Rust `launch_commands/terminal.rs`, Rust `launch_commands/process.rs`, and the detached launch helpers
- Statement: `yzx launch` builds the packaged Mars argv and runs the detached launch probe without generating terminal config
- Verification: automated focused Rust launch-command tests and validator `validate-installed-runtime-contract`

#### TLAUNCH-002
- Type: ownership
- Status: live
- Owner: Mars and Rust `launch_commands/terminal.rs`
- Statement: `~/.config/yazelix/mars/config.toml` is the optional complete user-owned Mars config. When it exists, `MARS_CONFIG_HOME` points to its directory. When it is absent, `MARS_CONFIG_HOME` points to the packaged complete config under `share/mars`. Yazelix does not merge the files or inspect ambient `~/.config/mars/config.toml`
- Verification: automated focused Rust `launch_commands::terminal` tests and config UI tests

#### TLAUNCH-003
- Type: boundary
- Status: live
- Owner: Mars, Ratconfig, and the Yazelix config UI host
- Statement: Mars owns its opacity, appearance, fonts, effects, and `[yazelix.cursor]`. Ratconfig exposes the complete TOML document generically. Root Yazelix appearance and cursor settings do not project into Mars
- Verification: config UI tests, Mars package validation, and manual fresh-window testing

#### TLAUNCH-004
- Type: boundary
- Status: live
- Owner: Rust launch commands and Home Manager
- Statement: Non-Mars terminals are not packaged or configured by Yazelix. Their native config stays host-owned and they enter Yazelix through `yzx enter`
- Verification: launch parsing tests plus flake and Home Manager validators

#### TLAUNCH-005
- Type: failure_mode
- Status: live
- Owner: Rust `launch_commands/process.rs` and the detached launch helpers
- Statement: Detached launch failures remain visible with captured stderr instead of silently succeeding
- Verification: automated Rust launch tests and validator `validate-installed-runtime-contract`

## Traceability

- Defended by: `yzx_repo_validator validate-contracts`
