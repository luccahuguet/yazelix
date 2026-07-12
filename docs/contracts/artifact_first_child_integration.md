# Artifact-First Child Integration

## Summary

Yazelix consumes child repositories through locked runtime artifacts wherever the runtime boundary is an artifact boundary. Maintainer and CI workflows build source artifacts; regular users run the artifacts selected by the active flake/package lock.

This is not a wasm-first policy. Wasm is the right artifact only where the runtime ABI is wasm, such as Zellij plugins. Native control-plane behavior stays native.

## Boundary Policy

The supported split is:

| Surface | Runtime form | Source owner | Yazelix owner |
| --- | --- | --- | --- |
| `yzx`, `yzx_core`, `yzx_control` | native executable | Yazelix main repo | command routing, config, materialization, diagnostics, runtime control |
| first-party Zellij plugins | `.wasm` artifact | plugin child repo | locked consumption, generated Zellij integration, runtime placement |
| `zjstatus.wasm` | `.wasm` artifact | upstream `zjstatus` through the current lock | locked package consumption and generated Zellij integration |
| `yazelix_screen` | Rust library plus standalone `yzs` package | `yazelix-screen` child repo | integrated welcome and `yzx screen` behavior |
| `yazelix_cursors` | Rust library plus standalone `yzc` package | `yazelix-cursors` child repo | config UI, settings generation, and Ghostty materialization |
| `yazelix_zellij_bar` | standalone package artifacts and widget binary | `yazelix-zellij-bar` child repo | integrated status-bar adapter and runtime path selection |

## Contract Items

#### AFCI-001
- Type: boundary
- Status: live
- Owner: Yazelix package/runtime architecture
- Statement: Normal Yazelix users should not need adjacent mutable child checkouts. Child source builds belong in maintainer, CI, release, or binary-cache production lanes. User runtimes consume the child artifacts selected by the lock file or package metadata
- Verification: automated `nix build .#yazelix`; manual review of package inputs and runtime tree contents

#### AFCI-002
- Type: boundary
- Status: live
- Owner: Rust control plane
- Statement: The Yazelix control plane remains a native helper. Replacing `yzx_core` or `yzx_control` with wasm is not a supported simplification unless a real wasm host boundary appears. The native helper may link small Rust libraries when in-process typed behavior is materially simpler than shelling out
- Verification: manual `cargo tree --manifest-path rust_core/Cargo.toml -p yazelix_core`

#### AFCI-003
- Type: boundary
- Status: live
- Owner: Zellij plugin packaging
- Statement: First-party Zellij plugin code is consumed as wasm artifacts from locked child packages. The main repo does not track copied first-party wasm or sync stamps as runtime provenance
- Verification: manual review against [First-Party Zellij Plugin Wasm Ownership](./first_party_zellij_plugin_wasm_ownership.md)

#### AFCI-004
- Type: boundary
- Status: live
- Owner: Integrated status-bar adapter
- Statement: The integrated Yazelix status bar should not compile the `yazelix_zellij_bar` child crate into `yazelix_core` solely to render KDL. The child package remains the owner of standalone artifacts, runnable widget commands, and integrated zjstatus plugin-block rendering from its runtime KDL template plus typed config; the main repo owns the session-specific adapter and path selection
- Verification: manual `cargo tree --manifest-path rust_core/Cargo.toml -p yazelix_core`; automated `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core zellij_materialization`

#### AFCI-005
- Type: non_goal
- Status: live
- Owner: runtime architecture
- Statement: Yazelix does not use wasm as a generic plugin/process boundary for native helper behavior, settings materialization, cursor parsing, or terminal animation just to avoid compiling Rust. Artifact boundaries should remove real runtime source coupling, not add a wasm host layer
- Verification: manual architecture review against this contract

## Verification

- `yzx_repo_validator validate-contracts`
- `cargo tree --manifest-path rust_core/Cargo.toml -p yazelix_core`
- `nix build .#yazelix`

## Traceability

- Defended by: [First-Party Zellij Plugin Wasm Ownership](./first_party_zellij_plugin_wasm_ownership.md)
- Defended by: [Standalone Yazelix Screen Distribution](./standalone_yazelix_screen_distribution.md)
- Defended by: [Standalone Cursor Distribution](./standalone_cursor_distribution.md)
- Defended by: [Status Bar Ownership](./status_bar_ownership.md)
