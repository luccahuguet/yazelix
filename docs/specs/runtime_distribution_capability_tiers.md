# Runtime Distribution Capability Tiers

## Summary

Yazelix should describe runtime/distribution behavior in terms of explicit ownership tiers instead of pretending the app owns a mutable runtime updater everywhere.

The important split is simple:

- `yzx update` is the chooser/help surface, not a tier-sensitive updater
- runtime replacement happens through explicit owner commands such as `yzx update upstream` or `yzx update home_manager`, not through a generic in-app runtime updater
- `yzx doctor` reports the active tier and keeps only the install diagnostics that still make sense

## Why

Yazelix currently runs in more than one legitimate distribution shape:

- compatibility installer runtime
- Home Manager-managed runtime
- store/package runtime
- runtime-root-only sessions such as source checkouts

Without an explicit contract, doctor behavior drifts back toward an installer-centric story, and update UX drifts back toward pretending one generic updater exists everywhere.

## Scope

- define the runtime/distribution tiers that `yzx doctor` still reasons about
- define the explicit owner split that the update surface must expose
- keep only the install diagnostics that still make sense after removing the old generic runtime-update story

## Behavior

### Capability Tiers

Yazelix recognizes four concrete runtime/distribution tiers:

1. `installer_managed`
   - compatibility tier for the legacy flake installer path
   - legacy installer-owned artifacts may still exist
   - the honest update command is `yzx update upstream`
2. `home_manager_managed`
   - full packaged-runtime tier
   - Home Manager owns the package path and update transition
   - the honest update command is `yzx update home_manager`, followed by a manual `home-manager switch`
3. `package_runtime`
   - narrowed packaged-runtime tier
   - Yazelix runs directly from a package root
   - package-manager updates still exist, but they are not currently wrapped by a dedicated `yzx update` subcommand
4. `runtime_root_only`
   - narrowed runtime-root tier
   - Yazelix has a usable runtime root but no package-manager-owned distribution surface
   - `yzx update` must not pretend this tier has a generic owner-managed update path

### `yzx update`

- `yzx update` prints the explicit owner choices instead of reporting the active runtime/distribution tier.
- `yzx update` points users at `yzx update upstream` and `yzx update home_manager`.
- `yzx update` warns users not to mix both update owners for the same installed Yazelix runtime.
- `yzx update` may still mention `yzx update nix` as a separate Nix-management utility.

### `yzx doctor`

- `yzx doctor` always reports the active runtime/distribution tier.
- `yzx doctor` no longer performs broad installer-artifact staleness checks.
- Desktop-entry diagnostics remain valid because desktop integration is still an explicit user-visible surface.
- Runtime-root-only mode must not warn about missing installer-owned runtime artifacts or pretend there is an installer repair path.

## Non-goals

- reintroducing a Yazelix-owned runtime updater
- redefining backend capability buckets or launch preflight here

## Acceptance Cases

1. `yzx update` is a chooser/help surface, not a tier-sensitive generic updater.
2. `yzx update upstream` and `yzx update home_manager` are the only dedicated Yazelix update-owner wrappers.
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
