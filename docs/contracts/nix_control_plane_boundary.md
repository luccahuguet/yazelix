# Nix Control-Plane Boundary

## Summary

Nix remains Yazelix's owner for package composition, platform gates, Home
Manager integration, flake outputs, overlays, runtime package assembly, and
derivation selection. Rust owns typed runtime behavior and validators, but it
must not take over package/platform/Home Manager semantics just to reduce Nix
line count.

The shrink target is accidental concentration inside a few Nix files, not Nix
ownership itself. Split large files by product owner while preserving the
current supported customization surfaces in
`docs/contracts/nix_customization_surfaces.md` and the config/runtime ownership
rules in `docs/contracts/config_runtime_control_plane_contract.md`.

## Current Baseline

Measured on `2026-06-06`:

| Surface | Raw lines |
| --- | ---: |
| All tracked `*.nix` files, excluding build outputs | 3214 |
| `home_manager/module.nix` | 1085 |
| `flake.nix` | 630 |
| `packaging/runtime_tool_registry.nix` | 373 |
| `maintainer_shell.nix` | 201 |
| `packaging/mk_runtime_tree.nix` | 181 |

Focused `tokei` measurement for `flake.nix`, `home_manager`, `packaging`,
`maintainer_shell.nix`, `yazelix_package.nix`, `yazelix_runtime_package.nix`,
and `nix-ci.nix` reports `3349` Nix lines and `2896` Nix code lines across
`26` Nix entries. The largest single-file pressure points are
`home_manager/module.nix`, `flake.nix`, and
`packaging/runtime_tool_registry.nix`.

## Ownership

| Concern | Current owner | Desired boundary |
| --- | --- | --- |
| Terminal variant package selection | `flake.nix`, `home_manager/module.nix`, package builder args | One Nix terminal-variant metadata module consumed by flake and Home Manager surfaces |
| Terminal runtime identity | package builders plus generated runtime identity | Package builders own package identity; terminal child packages may expose stable metadata consumed by main Yazelix |
| Home Manager options | `home_manager/module.nix` | Split into option declarations, package selection, settings rendering, desktop entries, activation/runtime materialization, and validation helpers |
| Home Manager settings rendering | `home_manager/module.nix` | Remain Nix-owned, backed by config metadata and `validate-config-surface-contract` |
| Home Manager activation/runtime materialization | `home_manager/module.nix` | Remain Home Manager-owned, with terminal-specific branches isolated |
| Desktop entries | `home_manager/module.nix` | Remain Home Manager/Linux-owned, split into a focused desktop-entry module with explicit platform gates |
| Flake packages/apps/checks/overlays | `flake.nix` | `flake.nix` wires outputs; package/check implementation lives in packaging modules |
| KGP package contracts | inline in `flake.nix` | Move to `packaging/kgp_package_contracts.nix` |
| Runtime tool manifest and source modes | `packaging/runtime_tool_registry.nix` | Keep Nix-owned; split only if a concrete owner appears after terminal/HM extraction |
| Runtime tree assembly | `packaging/mk_runtime_tree.nix` | Keep package-owned; do not move runtime tree semantics into Rust |
| Platform gates | Nix package/HM files | Stay explicit in Nix with `stdenv.hostPlatform` or Home Manager platform conditionals |

## Accepted Split Order

1. Extract KGP package contract implementation out of `flake.nix`
2. Centralize terminal variant metadata in a Nix module consumed by package and
   Home Manager surfaces
3. Split `home_manager/module.nix` by owner: options/defaults, package
   selection, settings rendering, desktop entries, activation/runtime
   materialization, and validation helpers
4. Split flake output wiring from package/check implementation only where
   `flake.nix` still owns logic after the KGP and terminal cuts
5. Re-evaluate `packaging/runtime_tool_registry.nix` after the previous cuts;
   do not split it only to make a smaller file

## Rejected Or Deferred Cuts

- Moving Nix package, derivation, platform, or Home Manager semantics into Rust
  is rejected. Rust may validate Nix-owned contracts, but it does not become the
  control-plane owner for those semantics
- `flake-parts` is deferred until the local ownership split proves that
  `flake.nix` still needs a framework. Adding a framework before the boundary
  split would add inputs and indirection without deleting product ownership
- Splitting `runtime_tool_registry.nix` is deferred. It is large, but it has one
  coherent owner today: runtime tool package/source-mode metadata

## Verification

Use focused checks before heavyweight runtime builds:

- `yzx_repo_validator validate-flake-interface`
- `yzx_repo_validator validate-nix-customization-api`
- `yzx_repo_validator validate-config-surface-contract`
- `nix eval .#packages.$(nix eval --raw --impure --expr builtins.currentSystem).yazelix.name --no-write-lock-file`
- current-system and unsupported-platform eval checks for moved package
  contracts when they are extracted

For cleanup and refactor work, record the before/after result with
`shells/posix/yazelix_loc_scorecard.sh <base> HEAD` as required by
`docs/loc_extraction_scorecard.md`.
