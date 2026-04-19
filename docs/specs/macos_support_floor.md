# macOS Support Floor

## Summary

This spec defines the minimum first-party macOS support contract for the top-level flake/package surface. It replaces the earlier implied parity claim with an explicit floor: what is actually guaranteed, what is best-effort, and what is explicitly out of scope for now.

## Why

The current tree contained mixed signals about macOS support. The top-level docs say Yazelix works on Linux and macOS, the runtime includes a darwin-specific Ghostty binary, and the terminal launcher has a macOS Ghostty branch. But the old clone-era macOS app-bundle launcher conflicted with the package-first v15 runtime contract. This spec replaces those mixed signals with one honest floor and leaves launcher integration as future package-first work instead of a stale carried surface.

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
| Spotlight/Launchpad/Dock app-bundle launcher | `historical_or_out_of_scope` | Yazelix does not ship a supported macOS app-bundle launcher today. The old clone-era bundle was removed instead of being kept as a half-supported surface. If launcher integration becomes a supported surface, it must be rebuilt to match the package-first ownership model. That work is tracked separately from this spec. |
| Home Manager macOS-specific paths | `best_effort` | The Home Manager module works where Nix and Home Manager are available, but macOS-specific integration paths have no dedicated validation |

## Ghostty on macOS

Ghostty is the intended first-party terminal on both Linux and macOS. The runtime bundles `pkgs.ghostty-bin` on darwin (a repackaging of the official signed and notarized macOS binary) and `pkgs.ghostty` on Linux.

The supported floor covers opening a Ghostty window on macOS via `yzx launch`. It does not promise automatic Ghostty shell integration on macOS. Ghostty's official docs say automatic shell integration on macOS requires either the macOS app bundle or a layout where Ghostty resources are available above the binary. The Yazelix runtime does not currently provision `GHOSTTY_RESOURCES_DIR` or ship as a macOS app bundle, so shell-integration features that depend on that resource-discovery path are not guaranteed.

Additionally, Ghostty on macOS launches login shells by default. This is a Ghostty platform behavior, not something Yazelix currently overrides or defends in its launch command. If login-shell behavior causes problems for specific Yazelix workflows, that should be reported and tracked separately.

## Platform Scope Discipline

- The first-party flake package and the nixpkgs submission draft have separate platform claims. See the nixpkgs package contract spec for the explicit split.
- Do not widen the macOS floor in this spec without adding corresponding validation.
- Do not silently imply macOS parity with Linux in docs or validators that this floor does not defend.

## Validation

Automated:
- `nu nushell/scripts/dev/validate_flake_interface.nu` — checks that all exported flake systems are available according to `meta.platforms`
- `nu nushell/scripts/dev/test_yzx_generated_configs.nu` — includes `test_ghostty_macos_launch_command_omits_linux_specific_flags` defending the macOS Ghostty command shape
- `nu nushell/scripts/dev/validate_specs.nu` — validates this spec is listed in the inventory

Manual smoke gate (maintainer on macOS hardware):
1. Install the package: `nix profile add github:luccahuguet/yazelix#yazelix` on macOS
2. Run `yzx --version-short` and confirm it reports a version
3. Run `yzx doctor --verbose` and confirm it completes without crashing
4. Run `yzx launch --terminal ghostty` and confirm a Ghostty window opens with Yazelix inside
5. Note whether Ghostty shell-integration features are actually present (cursor positioning, command history, working directory tracking) or absent
6. Confirm the docs do not tell macOS users to install or rely on a bundled `Yazelix.app` launcher

## Acceptance Cases

1. Yazelix has a live documented first-party macOS support-floor spec for the top-level flake/package surface, separate from Linux parity claims.
2. The spec classifies at least these surfaces as `supported`, `best_effort`, or `historical_or_out_of_scope`: install, `yzx --version-short`, `yzx doctor`, `yzx launch`, Ghostty shell-integration behavior, and Spotlight/Launchpad/Dock launcher integration.
3. The spec explicitly states that Ghostty is the intended first-party macOS terminal path and does not promise automatic Ghostty shell integration unless the underlying resource-discovery/app-bundle story is actually defended.
4. `README.md` and `docs/installation.md` no longer imply stronger or broader macOS support than the new support floor actually defends, and the stale clone-era app-bundle flow has been removed instead of kept as an implied primary path.
5. The validation story for the support floor is explicit and honest: automated where credible, otherwise a named manual macOS smoke procedure or maintainer gate on macOS hardware.
6. If macOS launcher integration remains a supported surface, the remaining implementation work is tracked separately instead of being left as stale clone-era docs.

## Traceability

- Bead: `yazelix-0nvb`
- Depends on: `yazelix-z5vf` (first-party flake package must report as available on darwin)
- Follow-up: package-first macOS launcher bead if Spotlight/Launchpad/Dock integration becomes an owned surface
- Defended by: `nushell/scripts/dev/validate_flake_interface.nu`
- Defended by: `nushell/scripts/dev/test_yzx_generated_configs.nu::test_ghostty_macos_launch_command_omits_linux_specific_flags`
