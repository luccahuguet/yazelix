# Runtime Distribution Capability Tiers

## Summary

Yazelix should describe runtime/distribution behavior in terms of explicit ownership tiers instead of pretending the app owns a mutable runtime updater everywhere.

The important split is simple:

- `yzx update` reports the owning update path for the active tier
- runtime replacement happens in the owning package-manager or compatibility-installer flow, not through an in-app `yzx update runtime` command
- `yzx doctor` reports the active tier and keeps only the install diagnostics that still make sense

## Why

Yazelix currently runs in more than one legitimate distribution shape:

- compatibility installer runtime
- Home Manager-managed runtime
- store/package runtime
- runtime-root-only sessions such as source checkouts

Without an explicit contract, update and doctor behavior drifts back toward an installer-centric story that is no longer true.

## Scope

- define the runtime/distribution tiers that `yzx update` and `yzx doctor` reason about
- define the owner guidance `yzx update` must print in each tier
- keep only the install diagnostics that still make sense after removing `yzx update runtime`

## Behavior

### Capability Tiers

Yazelix recognizes four concrete runtime/distribution tiers:

1. `installer_managed`
   - compatibility tier for the legacy flake installer path
   - legacy installer-owned artifacts may still exist
   - the honest update guidance is to rerun `nix run github:luccahuguet/yazelix#install` or move to a package-managed flow
2. `home_manager_managed`
   - full packaged-runtime tier
   - Home Manager owns the package path and update transition
   - `yzx update` points at reapplying or upgrading Home Manager
3. `package_runtime`
   - narrowed packaged-runtime tier
   - Yazelix runs directly from a package root
   - `yzx update` points at package-manager flows such as `nix profile upgrade`, `home-manager switch`, or system rebuilds
4. `runtime_root_only`
   - narrowed runtime-root tier
   - Yazelix has a usable runtime root but no package-manager-owned distribution surface
   - `yzx update` says to refresh the current runtime root manually or switch to the packaged `#yazelix` surface or Home Manager

### `yzx update`

- `yzx update` always reports the active runtime/distribution tier first.
- `yzx update` no longer exposes `yzx update runtime` or `yzx update all`.
- `yzx update` prints owner guidance for the current tier and may still mention `yzx update nix` as a separate Nix-management utility.

### `yzx doctor`

- `yzx doctor` always reports the active runtime/distribution tier.
- `yzx doctor` no longer performs broad installer-artifact staleness checks.
- Desktop-entry diagnostics remain valid because desktop integration is still an explicit user-visible surface.
- Runtime-root-only mode must not warn about missing installer-owned runtime artifacts or pretend there is an installer repair path.

## Non-goals

- reintroducing a Yazelix-owned runtime updater
- defining a new dedicated `yzx update home_manager` command
- redefining backend capability buckets or launch preflight here

## Acceptance Cases

1. `yzx update` prints owner guidance without exposing `yzx update runtime` or `yzx update all`.
2. Home Manager-managed mode never falls through to installer-specific update advice beyond explicit compatibility guidance.
3. Package/store runtimes do not claim they own a mutable installed runtime.
4. Runtime-root-only doctor output does not warn about missing installer-owned runtime artifacts.
5. The tier contract is explicit enough that later simplification work can delete installer ownership cleanly.

## Verification

- integration tests:
  - `nu -c 'source nushell/scripts/dev/test_yzx_core_commands.nu; run_core_canonical_tests'`
  - `nu -c 'source nushell/scripts/dev/test_yzx_doctor_commands.nu; run_doctor_canonical_tests'`
- manual review against:
  - [runtime_ownership_reduction_matrix.md](./runtime_ownership_reduction_matrix.md)
  - [package_runtime_first_user_and_maintainer_ux.md](./package_runtime_first_user_and_maintainer_ux.md)

## Traceability

- Bead: `yazelix-4buc.4`
- Defended by:
  - `nu -c 'source nushell/scripts/dev/test_yzx_core_commands.nu; run_core_canonical_tests'`
  - `nu -c 'source nushell/scripts/dev/test_yzx_doctor_commands.nu; run_doctor_canonical_tests'`
