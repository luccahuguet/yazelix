# Rust/Nushell Bridge Contract

## Summary

Rust owns the public Yazelix command surface and typed runtime behavior. Nushell owns interactive shell configuration, not a second command implementation. The private `yzx_core` executable remains only where POSIX or managed-tool launchers need a packaged subprocess boundary

## Contract Items

#### BRIDGE-001
- Type: ownership
- Status: live
- Owner: Rust `yzx`, `yzx_control`, and command metadata
- Statement: Public command parsing, help, human rendering, and machine reports have one Rust owner. Nushell may source generated extern declarations but must not recreate command behavior or metadata
- Verification: automated `rust_core/yazelix_core/tests/yzx_control_runtime_surface.rs`; validator `yzx_repo_validator validate-contracts`

#### BRIDGE-002
- Type: boundary
- Status: live
- Owner: private `yzx_core` invocation seam
- Statement: Packaged launchers call `yzx_core` from the active runtime root with structured argv or an explicit JSON request. They must not discover another revision from ambient `PATH` or assemble inline shell programs
- Verification: automated `rust_core/yazelix_core/tests/yzx_control_runtime_surface.rs`; automated `rust_core/yazelix_core/tests/yzx_core_runtime_env.rs`

#### BRIDGE-003
- Type: behavior
- Status: live
- Owner: `yzx_core` machine transport
- Statement: Normal helper success writes one JSON success envelope to stdout. Normal helper failure writes one structured JSON error envelope to stderr and exits according to the shared error class
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --bin yzx_core`

#### BRIDGE-004
- Type: ownership
- Status: live
- Owner: Rust command metadata and initializer generation
- Statement: Rust generates the Nushell extern bridge from the canonical command registry. Managed Nushell configuration only sources that generated file and does not inspect or mirror the registry
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core command_metadata`; automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core initializer_commands`

#### BRIDGE-005
- Type: invariant
- Status: live
- Owner: Rust generated-state writers
- Statement: Private helpers write only explicit Yazelix-managed paths, keep deterministic inputs deterministic, and fail visibly instead of taking ownership of user-managed configuration
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core runtime_materialization`

## Current Ownership

| Surface | Owner |
| --- | --- |
| Public `yzx` parsing, help, reports, and prose | Rust `yzx` and `yzx_control` |
| Typed config, runtime, materialization, diagnostics, profiles, and command metadata | `yazelix_core` |
| Packaged subprocess-only helpers | private `yzx_core` under `libexec/` |
| Interactive Nushell behavior and prompt integration | `nushell/config/config.nu` and `stack_prompt_guard.nu` |
| Stable host bootstrap and managed-tool launch trampolines | checked-in POSIX scripts |

The private helper surface is intentionally narrow:

- `runtime-env.compute`
- `runtime-materialization.repair`
- `helix-materialization.generate`

These names are not public `yzx` commands. New public behavior should run in-process through Rust. A new private helper is justified only when an existing packaged launcher cannot call the library directly

## Invocation And Output

- Installed runtimes resolve `$YAZELIX_RUNTIME_DIR/libexec/yzx_core`
- Source-checkout probes may use the matching local build, but installed revisions must never cross-call one another
- Dynamic values travel as argv, explicit environment values, or JSON data, not interpolated shell source
- Machine envelopes keep stable `status`, `command`, error class, code, message, remediation, and details fields
- Human output belongs to the public Rust command or an explicitly human helper mode, never mixed into normal JSON transport

## Generated State

Rust may write generated runtime state only after the caller supplies the owned roots. User `config.toml`, native sidecars, and external tool configuration remain outside that write authority unless a dedicated user command explicitly edits the selected Yazelix-owned file

The generated Nushell extern file is disposable startup glue. Rust refreshes it when command metadata changes; Nushell sources it without becoming a command owner

## Non-goals

- restoring Nushell command wrappers around Rust behavior
- exposing `yzx_core` as a supported user command
- moving Zellij wasm plugin ownership into `rust_core`
- using ambient host state or local caches as product truth
- adding shell-string trampolines around structured Rust commands

## Verification

- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --bin yzx_core`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_control_runtime_surface`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core initializer_commands`
- `yzx_repo_validator validate-contracts`

## Traceability
- Defended by: `yzx_repo_validator validate-contracts`
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --bin yzx_core`
