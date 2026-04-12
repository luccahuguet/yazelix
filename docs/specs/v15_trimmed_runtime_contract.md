# v15 Trimmed Runtime Contract

## Summary

This spec is the current branch-level contract for the trimmed v15 line.

The product surface is now centered on:

- a fixed packaged runtime
- explicit install/update owners
- managed `user_configs/` configuration
- workspace/session commands
- generated-state repair instead of backend/profile orchestration

It is no longer centered on:

- `yazelix_packs.toml`
- dynamic pack graphs
- the old runtime-local `devenv` layer
- cached launch-profile reuse
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
- That runtime ships a fixed toolset rather than a user-managed package graph.
- Ghostty is built into the Yazelix runtime on Linux and macOS as the first-party terminal path.
- WezTerm, Kitty, Alacritty, and Foot remain supported alternatives when the user provides those binaries on `PATH`.
- The runtime does not ship a runtime-local `devenv`.

### Config Surface

- The canonical user config surface is `~/.config/yazelix/user_configs/`.
- The main config is `user_configs/yazelix.toml`.
- Managed override directories such as Zellij, Yazi, Helix, and shell user hooks remain part of that user-owned config surface.
- The trimmed v15 branch does not have a `yazelix_packs.toml` sidecar and does not expose a first-class pack graph.
- Legacy or removed config fields fail fast instead of degrading silently.

### Generated State

- Generated Zellij/Yazi configs, shell initializers, logs, and repair hashes live under `~/.local/share/yazelix`.
- Those files are derived artifacts, not canonical handwritten config.
- `yzx refresh` owns generated-state repair only.
- `yzx refresh` does not rebuild or reuse a cached backend launch profile.

### Update And Distribution Ownership

- Users choose one explicit update owner per install.
  - `yzx update upstream` for upstream/manual installs
  - `yzx update home_manager` for Home Manager installs
- `#install` remains a compatibility/bootstrap surface.
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
- pretending the compatibility installer is the canonical everyday product flow

## Acceptance Cases

1. A current v15 user can understand the product without learning about `yazelix_packs.toml`, launch-profile reuse, or runtime-local `devenv`.
2. Current docs explain `yzx refresh` as generated-state repair rather than backend/profile orchestration.
3. Current docs explain update ownership through explicit owner commands rather than a generic runtime updater.
4. Current docs distinguish the normal packaged runtime from maintainer-only `nix develop` workflows.

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
