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

Measured after the Nix split pass on `2026-06-06`:

| Surface | Raw lines |
| --- | ---: |
| All tracked `*.nix` files, excluding build outputs | 3271 |
| `home_manager/module.nix` | 84 |
| `home_manager/options.nix` | 487 |
| `home_manager/settings_contract.nix` | 272 |
| `home_manager/runtime_integration.nix` | 244 |
| `flake.nix` | 339 |
| `packaging/flake_outputs.nix` | 110 |
| `packaging/runtime_tool_registry.nix` | 424 |
| `maintainer_shell.nix` | 201 |
| `packaging/mk_runtime_tree.nix` | 181 |

Current focused `tokei` measurement for `flake.nix`, `home_manager`,
`packaging`, `maintainer_shell.nix`, `yazelix_package.nix`, and
`yazelix_runtime_package.nix` reports `3444` Nix lines and `3049` Nix code
lines across `28` Nix entries. The largest remaining single-file pressure
points are
`home_manager/options.nix`, `packaging/runtime_tool_registry.nix`, and
`flake.nix`.

## Ownership

| Concern | Current owner | Desired boundary |
| --- | --- | --- |
| Terminal variant package selection | `flake.nix`, `home_manager/module.nix`, package builder args | One Nix terminal-variant metadata module consumed by flake and Home Manager surfaces |
| Terminal runtime identity | package builders plus generated runtime identity | Package builders own package identity; terminal child packages may expose stable metadata consumed by main Yazelix |
| Home Manager options | `home_manager/options.nix` | Option declarations/defaults stay separate from runtime integration |
| Home Manager settings rendering | `home_manager/settings_contract.nix` | Remain Nix-owned, backed by config metadata and `validate-config-surface-contract` |
| Home Manager activation/runtime materialization | `home_manager/runtime_integration.nix` | Remain Home Manager-owned, with terminal-specific branches isolated |
| Desktop entries | `home_manager/runtime_integration.nix` | Remain Home Manager/Linux-owned with explicit platform gates |
| Flake packages/apps/checks/overlays | `flake.nix`, `packaging/flake_outputs.nix`, `packaging/kgp_package_contracts.nix` | `flake.nix` wires outputs; package/check implementation lives in packaging modules |
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

## Runtime Tool Registry Decision

Keep `packaging/runtime_tool_registry.nix` whole for the current architecture.
It has one coherent owner: converting the selected runtime variant and
`runtimeToolSources` map into runtime packages, exported commands, validation
errors, and the manifest consumed by the packaged runtime.

Splitting it today would mostly separate dependent halves of the same manifest
contract:

- terminal package selection feeds the `terminal` tool entry and yzxterm
  package identity fields
- source-mode validation depends on the full tool metadata map
- bundled package lists, exported commands, and `manifestJson` are projections
  of that same validated map
- Linux graphics wrappers and Linux-only host helper packages are platform gates
  in the package owner, not independent user-facing modules

The next valid split is narrow, not structural: if terminal child package
metadata or graphics-wrapper policy grows again, extract that specific package
selection adapter behind the same registry interface. Do not split tool metadata,
source-mode validation, and manifest rendering into separate files unless a new
consumer needs one of those artifacts independently.

## Rejected Or Deferred Cuts

- Moving Nix package, derivation, platform, or Home Manager semantics into Rust
  is rejected. Rust may validate Nix-owned contracts, but it does not become the
  control-plane owner for those semantics
- `flake-parts` is deferred until the local ownership split proves that
  `flake.nix` still needs a framework. Adding a framework before the boundary
  split would add inputs and indirection without deleting product ownership
- Splitting `runtime_tool_registry.nix` is rejected for now. It is large, but it
  has one coherent owner today: runtime tool package/source-mode metadata,
  validation, and manifest projection

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
