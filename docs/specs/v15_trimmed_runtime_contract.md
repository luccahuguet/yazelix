# v15 Trimmed Runtime Contract

## Summary

This spec is the current branch-level contract for the trimmed v15 line.

v15.0 is the slimmed-down non-Rust reboot. Selective Rust may land later in v15.x when it clearly pays for itself; v16 is the Rust-forward release target.

The product surface is now centered on:

- a fixed packaged runtime
- explicit install/update owners
- managed `user_configs/` configuration
- workspace/session commands
- helperless fast popup/menu transient panes
- generated-state repair instead of backend/profile orchestration

It is no longer centered on:

- `yazelix_packs.toml`
- dynamic pack graphs
- the old runtime-local `devenv` layer
- cached launch-profile reuse
- automatic config migrations
- a generic Yazelix-owned runtime updater

## Why

Several older specs still describe the pre-trim planning space where Yazelix owned more runtime/package-manager behavior. That history is useful, but it is not the current branch contract anymore.

This file exists so current docs and current specs can point at one authoritative post-trim description instead of reusing planning notes that were written before the deletions actually landed.

## Scope

- define the current v15 user-facing runtime/config/update boundary
- define what still belongs to the normal product surface
- define what moved to maintainer-only or historical territory
- give other docs a safe current contract to point at

## Current Contract

### Runtime Surface

- The normal product runtime is the packaged `yazelix` runtime.
- That runtime ships a fixed toolset rather than a user-managed package graph, but interactive shells only export the curated user-facing tool surface instead of the full helper closure.
- Runtime tool versions come from the locked `nixpkgs` input. Maintainer update pins record the Nix helper version and the Nixpkgs-provided Nushell version, so upstream Nushell releases only become runtime bumps after they land in Nixpkgs or Yazelix deliberately changes that ownership model.
- Ghostty is built into the Yazelix runtime on Linux and macOS as the first-party terminal path.
- WezTerm, Kitty, Alacritty, and Foot remain supported alternatives when the user provides those binaries on `PATH`.
- The runtime does not ship a runtime-local `devenv`.

### Config Surface

- The canonical user config surface is `~/.config/yazelix/user_configs/`.
- The main config is `user_configs/yazelix.toml`.
- Managed override directories such as Zellij, Yazi, Helix, and shell user hooks remain part of that user-owned config surface.
- The trimmed v15 branch does not have a `yazelix_packs.toml` sidecar and does not expose a first-class pack graph.
- Legacy or removed config fields fail fast instead of degrading silently.
- v15 does not ship a config-migration engine. Users moving from very old config shapes should compare with the current template manually or use `yzx config reset` as a blunt fresh-start path.

### Generated State

- Generated Zellij/Yazi configs, shell initializers, logs, and repair hashes live under `~/.local/share/yazelix`.
- Those files are derived artifacts, not canonical handwritten config.
- Generated-state repair is an internal runtime responsibility surfaced through startup preflight, `yzx doctor`, and maintainer canaries rather than through a public refresh command.
- Generated-state repair does not rebuild or reuse a cached backend launch profile.

### Update And Distribution Ownership

- Users choose one explicit update owner per install.
  - `yzx update upstream` for default-profile installs of `#yazelix`
  - `yzx update home_manager` for Home Manager installs
- The flake no longer exposes `#install`.
- The product no longer promises a generic in-app runtime updater that owns every install mode.

### Maintainer Boundary

- `maintainer_shell.nix` defines the repo development shell for maintainer workflows.
- That maintainer shell is not the normal user runtime contract.
- Maintainer-only commands may still touch flake inputs, repo profiling flows, or release automation.
- Those maintainer semantics should not leak back into user-facing runtime docs.

## Non-goals

- reintroducing dynamic pack management on the trimmed v15 branch
- treating cached launch-profile reuse as a current product guarantee
- restoring the old runtime-local `devenv` layer as part of the normal shipped runtime
- restoring automatic config migrations as a normal v15 product surface
- pretending the compatibility installer is the canonical everyday product flow
- treating v15.0 as the Rust release

## Acceptance Cases

1. A current v15 user can understand the product without learning about `yazelix_packs.toml`, launch-profile reuse, runtime-local `devenv`, or automatic config migrations.
2. Current docs explain generated-state repair through startup and `yzx doctor` rather than through a public refresh command.
3. Current docs explain update ownership through explicit owner commands rather than a generic runtime updater.
4. Current docs distinguish the normal packaged runtime from maintainer-only `nix develop` workflows.
5. Current roadmap docs describe v15.0 as the trimmed non-Rust reboot and v16 as the Rust-forward release.

## Verification

- `nu nushell/scripts/dev/validate_installed_runtime_contract.nu`
- `nu nushell/scripts/dev/validate_flake_install.nu all`
- `nu nushell/scripts/dev/test_yzx_workspace_commands.nu`
- `nu nushell/scripts/dev/test_yzx_generated_configs.nu`
- `nu nushell/scripts/dev/test_yzx_maintainer.nu`
- `nu nushell/scripts/dev/test_stale_config_diagnostics_e2e.nu`

## Traceability

- Bead: `yazelix-qgj7.2.4.3`
- Defended by: `nu nushell/scripts/dev/validate_installed_runtime_contract.nu`
- Defended by: `nu nushell/scripts/dev/validate_flake_install.nu all`
- Defended by: `nu nushell/scripts/dev/test_yzx_workspace_commands.nu`
- Defended by: `nu nushell/scripts/dev/test_yzx_generated_configs.nu`
- Defended by: `nu nushell/scripts/dev/test_yzx_maintainer.nu`
