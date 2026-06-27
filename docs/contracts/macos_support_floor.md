# macOS Support Floor

## Summary

This contract defines the minimum first-party macOS support contract for the top-level flake/package surface. It keeps the Mars terminal honest on macOS: package and diagnostic surfaces are first-party, while macOS-specific Mars bugs are issue-driven because maintainers do not currently own macOS hardware.

## Why

The current tree contained mixed signals about macOS support. The top-level docs say Yazelix works on Linux and macOS, but maintainers do not currently own macOS hardware. This contract keeps the package-first floor honest and keeps full Yazelix app integration separate from the experimental package-first preview.

## Surface Classification

| Surface | Status | Notes |
| --- | --- | --- |
| First-party flake/package installation on macOS | `supported` | Install via `nix profile add github:luccahuguet/yazelix#yazelix` on the darwin flake systems |
| `yzx --version-short` | `supported` | Reports the Yazelix version from the installed runtime on macOS |
| `yzx doctor` | `supported` | Runs diagnostic checks on macOS; Linux-specific checks may report limitations but should not crash |
| `yzx launch` on Mars | `issue_driven_best_effort` | Mars is the packaged terminal, but macOS-specific Mars behavior is maintained from user reports because maintainers do not currently own macOS hardware |
| Host terminals on macOS | `supported_floor` | Configure Ghostty, WezTerm, Kitty, or another capable host terminal to run `yzx enter`; Ghostty is the strongest mature recommendation |
| `yzx enter` | `supported_floor` | The maintained host-terminal entrypoint; macOS-specific edge cases are issue-driven until a dedicated macOS validation lane exists |
| Zellij and Yazi behavior | `best_effort` | Core session and file manager should work, but macOS-specific edge cases are best-effort until reported and fixed |
| Package-first macOS launcher preview | `experimental` | `yzx desktop macos_preview install` creates `~/Applications/Yazelix Preview.app` as an opt-in, unsigned, unnotarized, maintainer-unverified preview for community testing. It resolves the default Nix profile or Home Manager profile `yzx` wrapper and does not assume a repo clone. It is explicitly separate from any supported Spotlight/Launchpad/Dock launcher claim. |
| Supported Spotlight/Launchpad/Dock app-bundle launcher | `historical_or_out_of_scope` | Yazelix does not ship a supported macOS app-bundle launcher today. The old clone-era bundle was removed instead of being kept as a half-supported surface. The preview above may inform a future supported surface, but it is not that supported surface. |
| Home Manager macOS-specific paths | `best_effort` | The Home Manager module works where Nix and Home Manager are available and does not emit Linux desktop-entry definitions on macOS, but macOS-specific integration paths have no dedicated validation |

## Mars And Host Terminals On macOS

Mars is the packaged Yazelix terminal on macOS and Linux because Yazelix can evolve the Rust terminal fork, generated config, cursor behavior, and agent-driven development workflow together. macOS-specific Mars support is issue-driven: users should report `yzx doctor --verbose`, terminal label, macOS version, architecture, launch logs, and whether the same workflow works through a host terminal running `yzx enter`.

Host terminals intentionally own their own macOS app behavior. Configure the terminal's startup command to run `yzx enter`; Ghostty is the most tested mature host-terminal path and may be the better macOS daily-driver choice while Mars macOS reports accumulate. Yazelix does not package a Ghostty app bundle or promise automatic host-terminal shell integration niceties such as command history, cursor positioning, or working-directory tracking.

## Package-First Launcher Preview

The macOS launcher preview is intentionally smaller than a supported native app story:

- It is installed only by explicit opt-in: `yzx desktop macos_preview install`
- It creates `~/Applications/Yazelix Preview.app`, marked as Yazelix-managed so uninstall does not take ownership of unrelated app bundles
- It calls `desktop launch` through the active package profile `yzx` wrapper rather than a Nix store runtime, repo checkout, or clone-era `assets/macos` path
- Default-profile installs resolve through the first existing `~/.nix-profile/bin/yzx` or `/etc/profiles/per-user/$USER/bin/yzx`
- Home Manager installs resolve through the same profile wrapper when present
- Missing or non-executable launcher failures are visible through the app script and tell the user to reinstall Yazelix and rerun `yzx desktop macos_preview install`
- Startup failures ask the user to run `yzx doctor --verbose` from Terminal before reporting feedback

This preview does not include code signing, notarization, a DMG, Dock polish, Launch Services guarantees, or maintainer macOS-hardware validation. Those claims require a future supported-launcher contract with credible release and macOS hardware validation.

## Platform Scope Discipline

- The first-party flake package and the nixpkgs submission draft have separate platform claims. See the nixpkgs package contract for the explicit split.
- Do not widen the macOS floor in this contract without adding corresponding validation.
- Do not silently imply macOS Mars parity with Linux in docs or validators that this floor does not defend.

## Validation

Automated:
- `yzx_repo_validator validate-flake-interface` — checks that all exported flake systems are available according to `meta.platforms`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core launch_commands::tests::parse_desktop_args_accepts_macos_preview_action`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core launch_commands::tests::render_macos_preview_launcher_uses_profile_yzx_and_actionable_failures`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core launch_commands::tests::render_macos_preview_info_plist_carries_owned_app_metadata`
- `yzx_repo_validator validate-contracts` — validates this contract is listed in the inventory

Manual smoke gate (maintainer on macOS hardware):
1. Install the package: `nix profile add github:luccahuguet/yazelix#yazelix` on macOS
2. Run `yzx --version-short` and confirm it reports a version
3. Run `yzx doctor --verbose` and confirm it completes without crashing
4. Run `yzx launch` from the default Mars package variant and confirm whether a Mars window opens with Yazelix inside
5. Configure a host terminal to run `yzx enter` and confirm a Yazelix session starts inside that terminal
6. Optionally run `yzx desktop macos_preview install`, open `~/Applications/Yazelix Preview.app`, and report whether it launches the active installed runtime
7. Confirm the docs label the app bundle as experimental, unsigned, unnotarized, and not maintainer-validated

## Acceptance Cases

1. Yazelix has a live documented first-party macOS support-floor contract for the top-level flake/package surface, separate from Linux parity claims.
2. The contract classifies at least these surfaces as `supported`, `supported_floor`, `issue_driven_best_effort`, `best_effort`, or `historical_or_out_of_scope`: install, `yzx --version-short`, `yzx doctor`, `yzx launch`, host-terminal entry, and Spotlight/Launchpad/Dock launcher integration.
3. The contract explicitly states that Mars is the packaged macOS terminal path, Mars macOS behavior is issue-driven, Ghostty is the strongest mature macOS host-terminal recommendation, and other capable host terminals use `yzx enter`.
4. `README.md` and `docs/installation.md` no longer imply stronger or broader macOS support than the new support floor actually defends, and the stale clone-era app-bundle flow has been removed instead of kept as an implied primary path.
5. The validation story for the support floor is explicit and honest: automated where credible, otherwise a named manual macOS smoke procedure or maintainer gate on macOS hardware.
6. The experimental package-first launcher preview remains distinct from a supported Spotlight/Launchpad/Dock launcher and carries visible package-first install, uninstall, failure, and feedback paths.

## Traceability
- Defended by: `yzx_repo_validator validate-flake-interface`
- Defended by: `rust_core/yazelix_core/src/launch_commands.rs::parse_desktop_args_accepts_macos_preview_action`
- Defended by: `rust_core/yazelix_core/src/launch_commands.rs::render_macos_preview_launcher_uses_profile_yzx_and_actionable_failures`
- Defended by: `rust_core/yazelix_core/src/launch_commands.rs::render_macos_preview_info_plist_carries_owned_app_metadata`
