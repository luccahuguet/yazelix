# Terminal Control Boundary

## Summary

Yazelix emits standard terminal control through typed APIs or small typed helpers. Raw terminal protocol strings are reserved for protocols without a typed Rust or Nushell layer, currently Kitty graphics placement and deletion.

## Contract Items

#### TCB-001
- Type: boundary
- Status: live
- Owner: Rust terminal rendering surfaces
- Statement: Standard ANSI and CSI behavior such as colors, attributes, cursor movement, alternate screen, line wrapping, and screen clearing should be emitted through `crossterm` or a crate-local helper built on `crossterm`, not through ad hoc raw escape string assembly
- Verification: automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core front_door`; automated `cargo test --manifest-path ../yazelix-screen/Cargo.toml`

#### TCB-002
- Type: boundary
- Status: live
- Owner: Nushell runtime scripts
- Statement: Nushell-owned terminal styling should use the `ansi` command for supported attributes and colors instead of raw escape literals
- Verification: automated `yzx_repo_validator validate-nushell-syntax`

#### TCB-003
- Type: non_goal
- Status: live
- Owner: Kitty graphics surfaces
- Statement: Kitty graphics protocol commands remain raw protocol strings because `crossterm` does not model Kitty image placement, deletion, payload transport, or z-index behavior. These strings must stay isolated behind clearly named Kitty-specific functions
- Verification: automated `cargo test --manifest-path ../yazelix-screen/Cargo.toml kitty_commands_use_file_payload_z_index_and_full_cleanup`

## Verification

- `yzx_repo_validator validate-contracts`
- `yzx_repo_validator validate-nushell-syntax`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core front_door`
- `cargo test --manifest-path ../yazelix-screen/Cargo.toml`
