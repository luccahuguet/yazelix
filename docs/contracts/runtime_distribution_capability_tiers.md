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
   - compatibility tier for older installs that still carry installer-owned runtime artifacts
   - new installs do not create this tier because the flake no longer exposes `#install`
   - the honest recovery path is to reinstall into the default Nix profile or move to Home Manager
2. `home_manager_managed`
   - full packaged-runtime tier
   - Home Manager owns the package path and update transition
   - the honest update command is `yzx update home_manager`, followed by a manual `home-manager switch`
3. `package_runtime`
   - narrowed packaged-runtime tier
   - Yazelix runs directly from a package root
   - when the package is owned by the default Nix profile, `yzx update upstream` is the explicit owner wrapper
   - temporary or one-shot package runtimes still do not imply a retained update owner
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

### Install Artifact Support Matrix

Yazelix keeps install-artifact diagnostics only where they defend a current install owner or a launcher path that can still shadow the current install:

| Artifact or branch | Status | Owner |
| --- | --- | --- |
| Home Manager-owned `config.toml` symlink marker | live support | Install ownership report |
| Home Manager profile `yzx` wrapper and profile desktop entries | live support | Install ownership report |
| Standalone default-profile Yazelix package entries during Home Manager takeover | live support | `yzx home_manager prepare` |
| User-local desktop entries and copied desktop icons that shadow profile/Home Manager entries | live cleanup support | `yzx home_manager prepare` and desktop doctor diagnostics |
| Legacy `~/.local/bin/yzx` wrapper and stale host-shell redirects | live cleanup diagnostics | Install ownership report and `yzx home_manager prepare` for the wrapper file only |
| Old desktop ids such as `yazelix.desktop` or the pre-variant `com.yazelix.Yazelix.desktop` | live cleanup support | Desktop freshness and Home Manager prepare |
| Old mutable `yazelix.toml` and `user_configs/` inputs | unsupported config state | Stale config diagnostics, manual cleanup, or `yzx reset config` |

`yzx home_manager prepare` must not archive unsupported legacy config inputs. Those files are not install-owner artifacts after the trimmed runtime contract; they are config-surface errors handled by the stale-config diagnostic contract.

## Non-goals

- reintroducing a Yazelix-owned runtime updater
- redefining backend capability buckets or launch preflight here

## Acceptance Cases

1. `yzx update` is a chooser/help surface, not a tier-sensitive generic updater.
2. `yzx update upstream` and `yzx update home_manager` are the only dedicated Yazelix update-owner wrappers.
3. Package/store runtimes do not claim they own a mutable installed runtime.
4. Runtime-root-only doctor output does not warn about missing installer-owned runtime artifacts.
5. The tier contract is explicit enough that later simplification work can delete installer ownership cleanly.
6. Home Manager prepare reports only current install-owner artifacts, not unsupported old config inputs such as `yazelix.toml` or `user_configs/`.

## Verification

- integration tests:
  - `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_control_runtime_surface`
  - `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core doctor_commands`
- manual review against:
  - [runtime_root_contract.md](./runtime_root_contract.md)
  - [v15_trimmed_runtime_contract.md](./v15_trimmed_runtime_contract.md)

## Traceability
- Defended by:
  - `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_control_runtime_surface`
  - `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core doctor_commands`
