# Package-Runtime-First User And Maintainer UX

> Status: Historical planning note.
> This document captures the transition space before the trimmed v15 branch dropped pack sidecars and runtime-local `devenv` from the normal user contract.
> Do not treat it as the current branch contract. See [v15_trimmed_runtime_contract.md](./v15_trimmed_runtime_contract.md).

## Summary

After the package-runtime-first simplification lane lands, the normal Yazelix product flow should be package-first:

- normal users consume the `yazelix` package or flake package output
- maintainers validate the packaged runtime directly as the main product surface
- `#install` becomes a compatibility/bootstrap surface at most, not the canonical everyday product entrypoint

This is a distribution simplification, not a backend-free redesign.

That means package-runtime-first Yazelix still keeps:

- `yazelix.toml`
- `yazelix_packs.toml`
- backend materialization semantics such as `devenv`, if the backend layer is still retained

## Why

The current flake and installer work solved a real problem: Yazelix needed an honest first-party front door instead of a repo-clone-shaped setup story.

But that installer-first model still carries extra product weight:

- a mutable installed runtime pointer
- a stable launcher shim owned by Yazelix
- installer-specific update and repair expectations
- a user story that still treats packaged runtime use as secondary

If the intended simplification is package-runtime-first Yazelix, the user and maintainer story must be written down explicitly before code deletion starts. Otherwise different follow-on tasks will pull toward different end states.

## Scope

- define the canonical user flow for package-runtime-first Yazelix
- define the canonical maintainer flow for developing and dogfooding that model
- define what `#install` becomes in that model
- define what remains true about `yazelix.toml`, `yazelix_packs.toml`, and `devenv`

## Behavior

### Status

This spec describes the target UX for the package-runtime-first simplification lane.

It does **not** claim that the current released product already follows this model everywhere. The current installer-first flow remains documented separately in:

- [One-Command Install UX](./one_command_install_ux.md)
- [Flake Interface Contract](./flake_interface_contract.md)

### Core Rule

The real product surface should be the package:

- `packages.<system>.yazelix`
- `bin/yzx` from that packaged runtime

The flake installer app is not the long-term center of the product model in this target UX.

### Canonical User Flow

#### One-Off Use

Users should be able to run Yazelix directly from the package surface:

```bash
nix run github:luccahuguet/yazelix#yazelix -- launch
```

Other direct package-surface commands should follow the same shape, for example:

```bash
nix run github:luccahuguet/yazelix#yazelix -- doctor
nix run github:luccahuguet/yazelix#yazelix -- env --no-shell
```

#### Persistent Install

Users should normally install the package, not materialize a Yazelix-owned runtime pointer:

```bash
nix profile add github:luccahuguet/yazelix#yazelix
```

Equivalent supported package-manager flows can also provide the runtime:

- Home Manager
- system package installation
- later nixpkgs consumption

#### First Run

On first run from the packaged runtime:

- Yazelix still bootstraps canonical config surfaces under `~/.config/yazelix/user_configs/` if they are missing
- Yazelix still treats those config files as user-owned intent
- generated configs and runtime state still live under `~/.local/share/yazelix`

#### Configuration

Users continue to configure Yazelix through the same product-level surfaces:

- `yazelix.toml`
- `yazelix_packs.toml`
- `yzx edit`
- `yzx config`
- Home Manager when it renders the same effective config semantics

Package-runtime-first does **not** mean “config goes away.” It means install/update ownership moves to the package manager.

#### Runtime Refresh And Launch

If backend ownership is still retained, the normal runtime commands still make sense:

- `yzx launch`
- `yzx enter`
- `yzx env`
- `yzx run`
- generated-state repair via startup, doctor, and internal helpers

Those commands operate from the packaged runtime root rather than from an installer-owned `runtime/current` identity.

#### Updates

Users choose one explicit update owner for each install:

- upstream/manual installs: `yzx update upstream`
- Home Manager installs: `yzx update home_manager`, then `home-manager switch`
- other package-manager installs: use the owning package-manager flow directly

The canonical update story is no longer an old generic in-app runtime-update surface. The owner choice must stay explicit.

### Canonical Maintainer Flow

#### Repo Development

Maintainers still develop in the repo checkout:

- edit code in the repo
- run targeted tests and validators from the repo
- use the repo development environment for implementation work

This simplification does not require abandoning the repo-oriented development loop.

#### Product Validation

Maintainers should validate the packaged runtime directly as the primary product surface:

```bash
nix build .#yazelix
nix run .#yazelix -- launch
```

For longer dogfooding or PATH-based validation, maintainers may also install the local package output:

```bash
nix profile add .#yazelix
```

The important shift is:

- package-runtime dogfooding becomes the main product validation path
- installer-materialized runtime validation becomes secondary or transitional

#### Release And Support Thinking

Maintainers should reason about failures through the package/runtime model first:

- package/runtime contents
- config semantics
- backend materialization behavior
- workspace/session behavior

They should not default to debugging:

- installer-owned runtime pointers
- stable launcher repair
- mutable install-artifact ownership

unless those surfaces are still intentionally retained for compatibility.

### Role Of `#install`

In this target UX, `nix run ...#install` becomes one of these at most:

1. a compatibility path for users coming from the older installer-owned model
2. a bootstrap helper for environments where a package install is not yet the chosen onboarding step
3. a transitional migration surface while the package-first model is landing

It is **not** the canonical normal path for:

- updates
- maintainer dogfooding
- defining the product's runtime identity

### What Remains True About `devenv`

Package-runtime-first does **not** automatically mean “drop `devenv`.”

As long as backend ownership remains part of Yazelix:

- `devenv` can still materialize the environment
- `yzx run` and internal generated-state repair helpers can still rely on that backend
- pack-driven environment composition can still exist

Only a stronger later backend-free reduction would force the product to narrow or remove those semantics.

## Non-goals

- claiming that the current released product already matches this target everywhere
- dropping `devenv` in this spec alone
- dropping `yazelix.toml` or `yazelix_packs.toml`
- defining the final backend-free `Core` story
- rewriting README install instructions in advance of the actual simplification work

## Acceptance Cases

1. A maintainer can answer “what is the normal package-runtime-first user flow?” with one clear story centered on `#yazelix`, package installs, and package-manager updates.
2. A maintainer can answer “what is the normal maintainer dogfooding path?” with one clear story centered on validating `.#yazelix` rather than an installer-owned runtime pointer.
3. A maintainer can explain what happens to `#install` clearly: it becomes bootstrap/compatibility-oriented rather than the canonical everyday product surface.
4. A maintainer can explain what remains true about `yazelix.toml`, `yazelix_packs.toml`, and `devenv` in this model without conflating package/runtime simplification with backend removal.

## Verification

- manual review against:
  - [runtime_ownership_reduction_matrix.md](./runtime_ownership_reduction_matrix.md)
  - [nixpkgs_package_contract.md](./nixpkgs_package_contract.md)
  - [flake_interface_contract.md](./flake_interface_contract.md)
  - [config_surface_and_launch_profile_contract.md](./config_surface_and_launch_profile_contract.md)
- spec validation:
  - `yzx_repo_validator validate-specs`

## Traceability

- Bead: `yazelix-4buc.1`
- Defended by: `yzx_repo_validator validate-specs`
