# macOS Support Floor

## Summary

This spec defines the minimum first-party macOS support contract for the top-level flake/package surface. It replaces the earlier implied parity claim with an explicit floor: what is actually guaranteed, what is best-effort, and what is explicitly out of scope for now.

## Why

The current tree contained mixed signals about macOS support. The top-level docs say Yazelix works on Linux and macOS, the runtime includes a darwin-specific Ghostty binary, and the terminal launcher has a macOS Ghostty branch. But the old clone-era macOS app-bundle launcher conflicted with the package-first v15 runtime contract. This spec replaces those mixed signals with one honest floor and keeps full launcher integration separate from the experimental package-first preview.

## Surface Classification

| Surface | Status | Notes |
| --- | --- | --- |
| First-party flake/package installation on macOS | `supported` | Install via `nix profile add github:luccahuguet/yazelix#yazelix` on the darwin flake systems |
| `yzx --version-short` | `supported` | Reports the Yazelix version from the installed runtime on macOS |
| `yzx doctor` | `supported` | Runs diagnostic checks on macOS; Linux-specific checks may report limitations but should not crash |
| `yzx launch` on Ghostty | `supported` | The runtime bundles `ghostty-bin` on darwin; the terminal launcher has a macOS-specific Ghostty branch that omits Linux-only GTK/X11 flags |
| Ghostty shell-integration behavior on macOS | `historical_or_out_of_scope` | Ghostty automatic shell integration on macOS requires either the macOS app bundle or a resource-discovery layout where Ghostty resources are available above the binary. The Yazelix runtime ships Ghostty inside a Nix store path, not as a standalone app bundle, and `yazelix_ghostty.sh` does not provision `GHOSTTY_RESOURCES_DIR`. Without that provisioning, Ghostty shell-integration niceties are not guaranteed on macOS. This may improve if Yazelix provisions the required resource path in a future release, but it is not part of the current floor. |
| `yzx launch` on other terminals | `best_effort` | WezTerm, Kitty, Alacritty, and Foot are PATH-provided alternatives, but macOS launch paths for these terminals have less frequent validation than Ghostty |
| `yzx enter` | `best_effort` | Should work on macOS but has no dedicated macOS-only validation lane |
| Zellij and Yazi behavior | `best_effort` | Core session and file manager should work, but macOS-specific edge cases are best-effort until reported and fixed |
| Package-first macOS launcher preview | `experimental` | `yzx desktop macos_preview install` creates `~/Applications/Yazelix Preview.app` as an opt-in, unsigned, unnotarized, maintainer-unverified preview for community testing. It resolves the default Nix profile or Home Manager profile `yzx` wrapper and does not assume a repo clone. The production stance and promotion gate live in `macos_launcher_productization.md`. |
| Supported Spotlight/Launchpad/Dock app-bundle launcher | `historical_or_out_of_scope` | Yazelix does not ship a supported macOS app-bundle launcher today. The old clone-era bundle was removed instead of being kept as a half-supported surface. The preview above may inform a future supported surface, but it is not that supported surface. |
| Home Manager macOS-specific paths | `best_effort` | The Home Manager module works where Nix and Home Manager are available, but macOS-specific integration paths have no dedicated validation |

## Ghostty on macOS

Ghostty is the intended first-party terminal on macOS and Linux. The runtime bundles `pkgs.ghostty-bin` on darwin (a repackaging of the official signed and notarized macOS binary) and `pkgs.ghostty` on Linux in the default and explicit Ghostty variants.

The supported floor covers opening a Ghostty window on macOS via `yzx launch`. It does not promise automatic Ghostty shell integration on macOS. Ghostty's official docs say automatic shell integration on macOS requires either the macOS app bundle or a layout where Ghostty resources are available above the binary. The Yazelix runtime does not currently provision `GHOSTTY_RESOURCES_DIR` or ship as a macOS app bundle, so shell-integration features that depend on that resource-discovery path are not guaranteed.

Additionally, Ghostty on macOS launches login shells by default. This is a Ghostty platform behavior, not something Yazelix currently overrides or defends in its launch command. If login-shell behavior causes problems for specific Yazelix workflows, that should be reported and tracked separately.

## Package-First Launcher Preview

The macOS launcher preview is intentionally smaller than a supported native app story:

- It is installed only by explicit opt-in: `yzx desktop macos_preview install`
- It creates `~/Applications/Yazelix Preview.app`, marked as Yazelix-managed so uninstall does not take ownership of unrelated app bundles
- It calls `desktop launch` through the active package profile `yzx` wrapper rather than a Nix store runtime, repo checkout, or clone-era `assets/macos` path
- Default-profile installs resolve through the first existing `~/.nix-profile/bin/yzx` or `/etc/profiles/per-user/$USER/bin/yzx`
- Home Manager installs resolve through the same profile wrapper when present
- Missing or non-executable launcher failures are visible through the app script and tell the user to reinstall Yazelix and rerun `yzx desktop macos_preview install`
- Startup failures ask the user to run `yzx doctor --verbose` from Terminal before reporting feedback

This preview does not include code signing, notarization, a DMG, Dock polish, Launch Services guarantees, or maintainer macOS-hardware validation. Those claims require the supported-launcher gate in [`macos_launcher_productization.md`](./macos_launcher_productization.md).

## Platform Scope Discipline

- The first-party flake package and the nixpkgs submission draft have separate platform claims. See the nixpkgs package contract spec for the explicit split.
- Do not widen the macOS floor in this spec without adding corresponding validation.
- Do not silently imply macOS parity with Linux in docs or validators that this floor does not defend.

## Validation

Automated:
- `yzx_repo_validator validate-flake-interface` — checks that all exported flake systems are available according to `meta.platforms`
- `nu nushell/scripts/dev/test_yzx_generated_configs.nu` — includes `test_ghostty_macos_launch_command_omits_linux_specific_flags` defending the macOS Ghostty command shape
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core launch_commands::tests::parse_desktop_args_accepts_macos_preview_action`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core launch_commands::tests::render_macos_preview_launcher_uses_profile_yzx_and_actionable_failures`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core launch_commands::tests::render_macos_preview_info_plist_carries_owned_app_metadata`
- `yzx_repo_validator validate-specs` — validates this spec is listed in the inventory

Manual smoke gate (maintainer on macOS hardware):
1. Install the package: `nix profile add github:luccahuguet/yazelix#yazelix` on macOS
2. Run `yzx --version-short` and confirm it reports a version
3. Run `yzx doctor --verbose` and confirm it completes without crashing
4. Run `yzx launch --terminal ghostty` and confirm a Ghostty window opens with Yazelix inside
5. Note whether Ghostty shell-integration features are actually present (cursor positioning, command history, working directory tracking) or absent
6. Optionally run `yzx desktop macos_preview install`, open `~/Applications/Yazelix Preview.app`, and report whether it launches the active installed runtime
7. Confirm the docs label the app bundle as experimental, unsigned, unnotarized, and not maintainer-validated

## Acceptance Cases

1. Yazelix has a live documented first-party macOS support-floor spec for the top-level flake/package surface, separate from Linux parity claims.
2. The spec classifies at least these surfaces as `supported`, `best_effort`, or `historical_or_out_of_scope`: install, `yzx --version-short`, `yzx doctor`, `yzx launch`, Ghostty shell-integration behavior, and Spotlight/Launchpad/Dock launcher integration.
3. The spec explicitly states that Ghostty is the intended first-party macOS terminal path and does not promise automatic Ghostty shell integration unless the underlying resource-discovery/app-bundle story is actually defended.
4. `README.md` and `docs/installation.md` no longer imply stronger or broader macOS support than the new support floor actually defends, and the stale clone-era app-bundle flow has been removed instead of kept as an implied primary path.
5. The validation story for the support floor is explicit and honest: automated where credible, otherwise a named manual macOS smoke procedure or maintainer gate on macOS hardware.
6. The experimental package-first launcher preview remains distinct from a supported Spotlight/Launchpad/Dock launcher and carries visible package-first install, uninstall, failure, and feedback paths.

## Traceability

- Bead: `yazelix-0nvb`
- Bead: `yazelix-b63b.1`
- Bead: `yazelix-b63b.2`
- Depends on: `yazelix-z5vf` (first-party flake package must report as available on darwin)
- Follow-up: supported package-first macOS launcher if the preview earns a defended app-bundle contract
- Defended by: `yzx_repo_validator validate-flake-interface`
- Defended by: `nushell/scripts/dev/test_yzx_generated_configs.nu::test_ghostty_macos_launch_command_omits_linux_specific_flags`
- Defended by: `rust_core/yazelix_core/src/launch_commands.rs::parse_desktop_args_accepts_macos_preview_action`
- Defended by: `rust_core/yazelix_core/src/launch_commands.rs::render_macos_preview_launcher_uses_profile_yzx_and_actionable_failures`
- Defended by: `rust_core/yazelix_core/src/launch_commands.rs::render_macos_preview_info_plist_carries_owned_app_metadata`
