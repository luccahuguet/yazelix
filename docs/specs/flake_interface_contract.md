# Flake Interface Contract

> Status: Historical phase-1 installer/front-door note.
> This document captures the earlier installer-centered flake surface before the trimmed v15 branch demoted `#install` to a compatibility/bootstrap path.
> Do not treat it as the current branch contract. See [v15_trimmed_runtime_contract.md](./v15_trimmed_runtime_contract.md).

## Summary

Yazelix should expose a small, honest top-level flake surface centered on the packaged `yzx` runtime, not on the compatibility installer app.

## Goals

- make the packaged `yazelix` output the normal product entrypoint
- expose a package-ready runtime artifact
- keep the Home Manager module as a first-class declarative surface
- keep `#install` only as a compatibility/bootstrap output while it still exists
- keep `devenv.nix` and the existing runtime assets as the source of truth
- avoid reviving the old `~/.config/yazelix` clone assumption or installer-first identity as the default mental model

## Required Outputs

### `packages.<system>.yazelix`

The canonical package output.

```bash
nix run github:luccahuguet/yazelix#yazelix -- launch
nix profile install github:luccahuguet/yazelix#yazelix
```

It should provide the wrapped `yzx` command and the runtime-local tools it needs.

### `packages.<system>.runtime`

A lower-level runtime package containing the shipped Yazelix runtime tree.

### `packages.<system>.default`

Alias to `packages.<system>.yazelix`.

### `apps.<system>.default`

Alias to the packaged `yzx` entrypoint.

### `apps.<system>.yazelix`

A named app alias to the same packaged `yzx` entrypoint.

## Additional Outputs

- `apps.<system>.install`
  - compatibility/bootstrap surface while installer-managed flows still exist
- `homeManagerModules.default`
  - canonical top-level Home Manager module surface
- `homeManagerModules.yazelix`
  - named alias for the same canonical module
- `checks.<system>.*`
  - focused flake-evaluation and package-surface smoke checks

## Important Constraint

The flake must not become a second environment definition. `devenv.nix` and the shipped runtime assets remain the source of truth.

## Update Behavior

Canonical update story:

- `nix profile upgrade`
- `home-manager switch`
- system rebuilds

Non-goals:

- reinstalling a mutable Yazelix-owned runtime pointer as the default update path
- inventing a second runtime definition outside `devenv.nix`
- treating the compatibility installer app as the primary product identity

## Home Manager Relationship

Home Manager remains a supported first-class integration path through the same top-level flake. Do not keep a second `?dir=home_manager` flake surface as the primary integration path.

## Acceptance Cases

1. The top-level flake has a minimal, explicit output set instead of a vague “maybe devShell, maybe package, maybe app” surface.
2. `packages.<system>.default` points at `packages.<system>.yazelix`, not at the lower-level runtime artifact.
3. `apps.<system>.default` and `apps.<system>.yazelix` resolve to the packaged `yzx` entrypoint.
4. Home Manager remains available from the same top-level flake.
5. `apps.<system>.install` is clearly compatibility-oriented while it still exists.
6. The flake stays thin enough that `devenv.nix` remains the real runtime source of truth.

## Verification

- manual review against [package_runtime_first_user_and_maintainer_ux.md](./package_runtime_first_user_and_maintainer_ux.md)
- CI check: `nu nushell/scripts/dev/validate_flake_interface.nu`
- package-surface probe: `nix run .#yazelix -- --version-short`

## Traceability

- Bead: `yazelix-4buc.2`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu` (structure and required fields check)
