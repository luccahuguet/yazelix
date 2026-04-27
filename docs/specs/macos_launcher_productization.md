# macOS Launcher Productization

## Summary

This spec defines the production stance for the package-first macOS launcher path. The current launcher remains an explicit unsigned preview, but it must behave like an owned Yazelix app bundle and must not imply signed or notarized support before Yazelix can defend that release path.

## Decision

The current macOS launcher stance is `unsigned preview`.

Yazelix may ship and document `yzx desktop macos_preview install` as an opt-in package-first app bundle preview. Yazelix must not describe that bundle as a supported Spotlight, Launchpad, Dock, signed, or notarized app until a separate release gate proves Developer ID signing, notarization, stapling, and macOS hardware smoke coverage.

This is an explicit product stance, not an accidental limitation:

- The preview app is Yazelix-owned, package-first, and uninstallable
- The preview app resolves the active profile-owned `yzx` wrapper instead of a repo checkout
- The preview app surfaces runtime-missing and startup failures with user-visible messages
- The preview app is unsigned and unnotarized, so Gatekeeper friction is expected
- Release notes and install docs must keep the preview separate from any future supported app bundle

## App Bundle Contract

The preview bundle created by `yzx desktop macos_preview install` must:

- install only at `~/Applications/Yazelix Preview.app`
- include a Yazelix-managed marker before refresh or uninstall can take ownership of the path
- use bundle identifier `com.yazelix.YazelixPreview`
- include ordinary app metadata: package type `APPL`, display name, executable name, minimum macOS version, category, and bundle version fields
- run through the active package profile `yzx` wrapper and invoke `desktop launch`
- avoid repo checkout paths, clone-era `assets/macos` paths, or transient Nix store launcher assumptions
- keep failure messages actionable when the profile launcher is missing or startup fails

## Gatekeeper And Signing

The preview is unsigned and unnotarized. macOS may block the app or require an explicit user override depending on quarantine state, system policy, and how the bundle was installed.

Yazelix should not paper over that with support claims. The honest guidance is:

- use `yzx launch` from Terminal for the supported macOS launch path
- use the preview app only for community testing of the package-first app-bundle shape
- if Gatekeeper blocks the app, report the exact macOS version, install method, and visible Gatekeeper text
- do not ask users to bypass Gatekeeper as a normal supported install step

## Supported App Release Gate

Before Yazelix can promote the launcher from `unsigned preview` to a supported macOS app, a new bead or spec must define and verify:

- Apple Developer ID ownership and certificate handling
- deterministic app bundle or DMG artifact shape
- `codesign` invocation, entitlement stance, and verification
- notarization through `notarytool`
- stapling and post-staple validation
- release artifact checks in the maintainer workflow
- macOS hardware smoke coverage for Finder, Spotlight, Launchpad, and Dock launch
- upgrade and reinstall behavior for both default-profile and Home Manager installs

Until those are true, release packaging must keep the preview label.

## Maintainer Preview Checks

Before advertising a preview change, a maintainer should run:

```bash
cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core launch_commands::tests::parse_desktop_args_accepts_macos_preview_action
cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core launch_commands::tests::render_macos_preview_launcher_uses_profile_yzx_and_actionable_failures
cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core launch_commands::tests::render_macos_preview_info_plist_carries_owned_app_metadata
cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core launch_commands::tests::install_macos_preview_app_writes_managed_bundle
```

On macOS hardware, also run:

```bash
yzx desktop macos_preview install --print-path
codesign -dv --verbose=4 "$HOME/Applications/Yazelix Preview.app"
spctl --assess --type execute --verbose "$HOME/Applications/Yazelix Preview.app"
```

For the current unsigned preview stance, `codesign` and `spctl` may report unsigned or rejected status. That result is useful release evidence, not a failure of the preview contract.

## Traceability

- Bead: `yazelix-b63b.2`
- Parent: `yazelix-b63b`
- Builds on: `yazelix-b63b.1`
- Defended by: `rust_core/yazelix_core/src/launch_commands.rs::render_macos_preview_info_plist_carries_owned_app_metadata`
- Defended by: `rust_core/yazelix_core/src/launch_commands.rs::install_macos_preview_app_writes_managed_bundle`
