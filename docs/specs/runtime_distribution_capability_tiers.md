# Runtime Distribution Capability Tiers

## Summary

Yazelix should describe install/update/doctor behavior in terms of explicit runtime/distribution capability tiers instead of assuming every invocation owns a mutable installer-managed runtime.

The important split is not branding. It is whether the current Yazelix mode actually owns:

- a stable mutable runtime identity
- a stable launcher/update surface
- installer-owned repair promises

## Why

The current repository already supports more than one runtime/distribution shape:

- installer-managed flake installs
- Home Manager-managed installs
- store/package runtimes that run directly from the package path
- runtime-root-only sessions such as source checkouts or narrower future workspace-oriented slices

Without an explicit contract:

- `yzx update runtime` keeps pretending every mode can rerun the flake installer
- `yzx doctor` keeps warning about missing `runtime/current` or `~/.local/bin/yzx` even when those paths are intentionally out of scope
- later Core-readiness work keeps mixing “this mode is narrower” with “this mode is broken”

## Scope

- define the runtime/distribution tiers that install/update/doctor reason about
- define which tiers own `yzx update runtime`
- define when installer-artifact doctor checks should run
- keep the current command family while making behavior honest per tier

## Behavior

### Capability Tiers

Yazelix currently recognizes four concrete runtime/distribution tiers:

1. `installer_managed`
   - full runtime/distribution tier
   - the flake installer owns the stable runtime identity, `~/.local/share/yazelix/runtime/current`, and the stable `~/.local/bin/yzx` launcher
   - `yzx update runtime` is valid here
2. `home_manager_managed`
   - full runtime/distribution tier
   - Home Manager owns the stable runtime identity and launcher/update transition
   - `yzx update runtime` is intentionally unavailable here because Home Manager owns the update path
3. `package_runtime`
   - narrowed runtime/distribution tier
   - Yazelix runs directly from a packaged runtime root
   - installer-owned `runtime/current` and `~/.local/bin/yzx` are out of scope
   - updates belong to the package manager or rebuild flow that provided the package
4. `runtime_root_only`
   - narrowed runtime/distribution tier
   - Yazelix has a usable runtime root but no installer-owned distribution surface
   - this is the honest shape for source/runtime-root sessions and the closest current analogue to a future narrower workspace-oriented mode

### `yzx update`

- `yzx update` should report the active runtime/distribution tier first.
- In `installer_managed`, it should advertise `yzx update all` and `yzx update runtime`.
- In `home_manager_managed`, it should point the user at reapplying or upgrading the Home Manager configuration instead of advertising `yzx update runtime` as valid.
- In `package_runtime`, it should point the user at package-manager or rebuild flows such as `nix profile upgrade`, Home Manager rebuilds, or system rebuilds.
- In `runtime_root_only`, it should say that no mutable installed-runtime update surface exists and point users either at `nix run github:luccahuguet/yazelix#install` for a full install or at updating the current runtime root manually.

### `yzx update runtime` and `yzx update all`

- `yzx update runtime` and `yzx update all` are only valid in `installer_managed`.
- In every other tier, they should fail clearly with mode-specific guidance instead of attempting the installer path anyway.
- Home Manager-managed mode should never silently fall through to the installer update path.
- Packaged/store runtimes should never pretend to own a mutable installed runtime.

### `yzx doctor`

- `yzx doctor` should always report the active runtime/distribution tier.
- Installer-owned runtime artifact checks belong only to the full tiers:
  - `installer_managed`
  - `home_manager_managed`
- In narrowed tiers, doctor should skip installer-owned artifact checks with an explicit informational result instead of warning that installer-owned paths are missing.
- Desktop-entry diagnostics may still run outside the full tiers because desktop integration remains an explicit user-local integration surface rather than an installer-only invariant.

### Mode-Specific Repair Guidance

- `yzx doctor` must not imply that `yzx doctor --fix` can repair runtime ownership.
- Missing `devenv` or other backend/runtime ownership problems should point at the owning tier's real repair/update path:
  - installer-managed: `yzx update runtime`
  - Home Manager-managed: reapply or upgrade Home Manager
  - package runtime: upgrade or reinstall the package
  - runtime-root-only: provide the backend/runtime manually or materialize a full install explicitly

## Non-goals

- introducing a separate supported `Yazelix Core` product now
- adding a new dedicated `yzx update home_manager` command
- redefining launch preflight or backend capability buckets
- treating every source checkout as a broken install

## Acceptance Cases

1. `yzx update runtime` succeeds only in installer-managed mode.
2. Home Manager-managed mode never falls through to the flake installer update path.
3. Package/store runtimes do not claim they own a mutable installed runtime.
4. Runtime-root-only doctor output does not warn about missing installer-owned `runtime/current` or stable-launcher paths.
5. The tier contract is explicit enough that later Core-readiness and release-gate work can refer to it directly.

## Verification

- integration tests:
  - `nu -c 'source nushell/scripts/dev/test_yzx_core_commands.nu; run_core_canonical_tests'`
  - `nu -c 'source nushell/scripts/dev/test_yzx_doctor_commands.nu; run_doctor_canonical_tests'`
- manual review against:
  - [backend_capability_contract.md](./backend_capability_contract.md)
  - [yazelix_core_boundary.md](./yazelix_core_boundary.md)
  - [nixpkgs_package_contract.md](./nixpkgs_package_contract.md)

## Traceability

- Bead: `yazelix-zjyw`
- Defended by:
  - `nu -c 'source nushell/scripts/dev/test_yzx_core_commands.nu; run_core_canonical_tests'`
  - `nu -c 'source nushell/scripts/dev/test_yzx_doctor_commands.nu; run_doctor_canonical_tests'`
