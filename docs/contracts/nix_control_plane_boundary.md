# Nix control-plane boundary

## Summary

Nix owns package composition, platform gates, Home Manager integration, flake outputs, runtime assembly, and derivation selection

Rust owns typed runtime behavior and validators without taking over Nix or Home Manager semantics

## Ownership

| Concern | Owner | Boundary |
| --- | --- | --- |
| Complete Yazelix package | package builders and `packaging/` | Own the runtime dependency graph and package identity |
| Home Manager API and file rendering | `home_manager/module.nix` | Install one complete package and render only declared sparse files |
| Home Manager activation and desktop integration | `home_manager/runtime_integration.nix` | Repair generated runtime state and gate Linux desktop behavior |
| Semantic config contract | `config_metadata/main_config_contract.toml` and Rust config code | Home Manager treats semantic settings as opaque TOML data rather than duplicating every field |
| Native app config | each native file plus its consuming application | Home Manager only installs an explicitly declared `text` or `source` file |
| Flake packages, apps, checks, and overlays | `flake.nix` and `packaging/flake_outputs.nix` | `flake.nix` wires outputs while package modules own implementation |
| Runtime tool manifest and source modes | `packaging/runtime_tool_registry.nix` | Remain package construction details and are not Home Manager options |
| Runtime tree assembly | `packaging/mk_runtime_tree.nix` | Remains package-owned rather than moving into Rust |
| Platform gates | Nix package and Home Manager files | Use `stdenv.hostPlatform` or explicit Home Manager conditionals |

## Home Manager seam

`home_manager/module.nix` intentionally keeps only one shared native-file option constructor and direct TOML generation

It must not regain

- per-field semantic options
- package-builder argument projection
- runtime tool source or component toggles
- terminal selection
- recursive packaged-default merging
- migration logic

`home_manager/runtime_integration.nix` may depend on the selected package as a complete artifact but must not reconstruct or partially override it

## Runtime tool registry

`packaging/runtime_tool_registry.nix` converts package-builder inputs into runtime packages, exported commands, validation errors, and the runtime manifest

Those source modes describe the selected package, so doctor may report them as package-owned facts but must not direct users to removed Home Manager options

Keep the registry whole while it has one coherent consumer and owner

## Platform behavior

Linux-only desktop entries, graphics wrappers, packages, and helper paths must be gated before shared evaluation

The Home Manager module must evaluate on Darwin without declaring or reading Linux desktop-entry options

Unsupported package behavior must fail at the package boundary with an explicit message rather than through an accidental Linux path lookup

## Rejected cuts

- Moving derivation, package, platform, or Home Manager semantics into Rust
- Adding a Nix framework without deleting an existing ownership layer
- Splitting the runtime tool registry only to reduce a file-size metric
- Reintroducing helper modules for one-use Home Manager projections

## Verification

- `yzx_repo_validator validate-flake-interface`
- `yzx_repo_validator validate-nix-customization-api`
- `yzx_repo_validator validate-config-surface-contract`
- Linux and Darwin Home Manager evaluation
- `shells/posix/yazelix_loc_scorecard.sh <base> HEAD`
