# macOS Support Floor

## Summary

This contract defines the minimum first-party macOS support contract for the top-level flake/package surface. It keeps the default Mars terminal honest on macOS: package and diagnostic surfaces are first-party, Mars-specific macOS bugs are issue-driven because maintainers do not currently own macOS hardware, and Ghostty remains the mature selectable macOS terminal path.

## Why

The current tree contained mixed signals about macOS support. The top-level docs say Yazelix works on Linux and macOS, the runtime includes a darwin-specific Ghostty package, and the terminal launcher has a macOS Ghostty branch. But the old clone-era macOS app-bundle launcher conflicted with the package-first v15 runtime contract. This contract replaces those mixed signals with one honest floor and keeps full Yazelix app integration separate from the experimental package-first preview.

## Surface Classification

| Surface | Status | Notes |
| --- | --- | --- |
| First-party flake/package installation on macOS | `supported` | Install via `nix profile add github:luccahuguet/yazelix#yazelix` on the darwin flake systems |
| `yzx --version-short` | `supported` | Reports the Yazelix version from the installed runtime on macOS |
| `yzx doctor` | `supported` | Runs diagnostic checks on macOS; Linux-specific checks may report limitations but should not crash |
| `yzx launch` on Mars | `issue_driven_best_effort` | Mars is the default terminal, but macOS-specific Mars behavior is maintained from user reports because maintainers do not currently own macOS hardware |
| `yzx launch` on Ghostty | `supported` | The runtime bundles `ghostty-bin` on darwin; the terminal launcher hands generated Ghostty args to the runtime-owned `Applications/Ghostty.app` through `/usr/bin/open -na ... --args`, starts Yazelix with `--initial-command=direct:<runtime>/shells/posix/start_yazelix.sh`, and omits Linux-only GTK/X11 flags |
| Ghostty shell-integration behavior on macOS | `historical_or_out_of_scope` | Yazelix uses the Ghostty app-bundle launch route on macOS, but it does not separately guarantee automatic shell-integration niceties such as command history, cursor positioning, or working-directory tracking. Any remaining shell-integration gaps should be reported and tracked separately. |
| `yzx launch` on other terminals | `best_effort` | WezTerm and Kitty are supported alternatives, but macOS launch paths for these terminals have less frequent validation than Ghostty |
| `yzx enter` | `best_effort` | Should work on macOS but has no dedicated macOS-only validation lane |
| Zellij and Yazi behavior | `best_effort` | Core session and file manager should work, but macOS-specific edge cases are best-effort until reported and fixed |
| Package-first macOS launcher preview | `experimental` | `yzx desktop macos_preview install` creates `~/Applications/Yazelix Preview.app` as an opt-in, unsigned, unnotarized, maintainer-unverified preview for community testing. It resolves the default Nix profile or Home Manager profile `yzx` wrapper and does not assume a repo clone. It is explicitly separate from any supported Spotlight/Launchpad/Dock launcher claim. |
| Supported Spotlight/Launchpad/Dock app-bundle launcher | `historical_or_out_of_scope` | Yazelix does not ship a supported macOS app-bundle launcher today. The old clone-era bundle was removed instead of being kept as a half-supported surface. The preview above may inform a future supported surface, but it is not that supported surface. |
| Home Manager macOS-specific paths | `best_effort` | The Home Manager module works where Nix and Home Manager are available and does not emit Linux desktop-entry definitions on macOS, but macOS-specific integration paths have no dedicated validation |

## Mars And Ghostty On macOS

Mars is the default Yazelix terminal on macOS and Linux because Yazelix can evolve the Rust terminal fork, generated config, cursor behavior, and agent-driven development workflow together. macOS-specific Mars support is issue-driven: users should report `yzx doctor --verbose`, terminal variant, macOS version, architecture, launch logs, and whether the same workflow works under `terminal = "ghostty"`.

Ghostty is the mature selectable macOS terminal path. The runtime bundles `pkgs.ghostty-bin` on darwin (a repackaging of the official signed and notarized macOS binary) and `pkgs.ghostty` on Linux in the explicit Ghostty variant.

The supported floor covers opening a Ghostty window on macOS via `yzx launch`. The launch command uses `/usr/bin/open -na <runtime>/Applications/Ghostty.app --args`, preserves the generated config path and working directory, and passes Yazelix startup through Ghostty's `--initial-command=direct:<runtime>/shells/posix/start_yazelix.sh` form so the first Ghostty surface opens the Yazelix workspace directly. It does not separately promise automatic Ghostty shell integration niceties such as command history, cursor positioning, or working-directory tracking.

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

This preview does not include code signing, notarization, a DMG, Dock polish, Launch Services guarantees, or maintainer macOS-hardware validation. Those claims require a future supported-launcher contract with credible release and macOS hardware validation.

## Platform Scope Discipline

- The first-party flake package and the nixpkgs submission draft have separate platform claims. See the nixpkgs package contract for the explicit split.
- Do not widen the macOS floor in this contract without adding corresponding validation.
- Do not silently imply macOS Mars parity with Linux in docs or validators that this floor does not defend.

## Validation

Automated:
- `yzx_repo_validator validate-flake-interface` — checks that all exported flake systems are available according to `meta.platforms`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core launch_commands::tests::ghostty_macos_launch_uses_app_bundle_open`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core launch_commands::tests::parse_desktop_args_accepts_macos_preview_action`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core launch_commands::tests::render_macos_preview_launcher_uses_profile_yzx_and_actionable_failures`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core launch_commands::tests::render_macos_preview_info_plist_carries_owned_app_metadata`
- `yzx_repo_validator validate-contracts` — validates this contract is listed in the inventory

Manual smoke gate (maintainer on macOS hardware):
1. Install the package: `nix profile add github:luccahuguet/yazelix#yazelix` on macOS
2. Run `yzx --version-short` and confirm it reports a version
3. Run `yzx doctor --verbose` and confirm it completes without crashing
4. Run `yzx launch` from the default Mars package variant and confirm whether a Mars window opens with Yazelix inside
5. Run `yzx launch --term ghostty` or use `programs.yazelix.terminal = "ghostty"` and confirm a Ghostty window opens with Yazelix inside
6. Note whether Ghostty shell-integration features are actually present (cursor positioning, command history, working directory tracking) or absent
7. Optionally run `yzx desktop macos_preview install`, open `~/Applications/Yazelix Preview.app`, and report whether it launches the active installed runtime
8. Confirm the docs label the app bundle as experimental, unsigned, unnotarized, and not maintainer-validated

## Acceptance Cases

1. Yazelix has a live documented first-party macOS support-floor contract for the top-level flake/package surface, separate from Linux parity claims.
2. The contract classifies at least these surfaces as `supported`, `best_effort`, or `historical_or_out_of_scope`: install, `yzx --version-short`, `yzx doctor`, `yzx launch`, Ghostty shell-integration behavior, and Spotlight/Launchpad/Dock launcher integration.
3. The contract explicitly states that Mars is the default macOS terminal, Mars macOS behavior is issue-driven, and Ghostty remains the mature selectable macOS path without promising automatic Ghostty shell integration unless the underlying resource-discovery/app-bundle story is actually defended.
4. `README.md` and `docs/installation.md` no longer imply stronger or broader macOS support than the new support floor actually defends, and the stale clone-era app-bundle flow has been removed instead of kept as an implied primary path.
5. The validation story for the support floor is explicit and honest: automated where credible, otherwise a named manual macOS smoke procedure or maintainer gate on macOS hardware.
6. The experimental package-first launcher preview remains distinct from a supported Spotlight/Launchpad/Dock launcher and carries visible package-first install, uninstall, failure, and feedback paths.

## Traceability
- Defended by: `yzx_repo_validator validate-flake-interface`
- Defended by: `rust_core/yazelix_core/src/launch_commands.rs::ghostty_macos_launch_uses_app_bundle_open`
- Defended by: `rust_core/yazelix_core/src/launch_commands.rs::parse_desktop_args_accepts_macos_preview_action`
- Defended by: `rust_core/yazelix_core/src/launch_commands.rs::render_macos_preview_launcher_uses_profile_yzx_and_actionable_failures`
- Defended by: `rust_core/yazelix_core/src/launch_commands.rs::render_macos_preview_info_plist_carries_owned_app_metadata`
