# Standalone Yazelix Cursor Distribution

## Summary

`yazelix_cursors` is the standalone Yazelix cursor package for terminal users who want the Yazelix cursor shader look without adopting the full Yazelix workspace runtime.

The primary flake package is `.#yazelix_cursors`. The package exposes the standalone `yzc` binary, and the flake exposes `.#yzc` as an app for direct CLI use. Old cursor package names are not kept as compatibility aliases.

The source repository is [`luccahuguet/yazelix-cursors`](https://github.com/luccahuguet/yazelix-cursors). Yazelix consumes that repository through explicit flake and Cargo dependency pins.

## Decision

Yazelix should ship `yazelix_cursors` from a separate standalone repository. The external repository owns reusable cursor registry parsing, validation, resolution, Ghostty-compatible palette/effect shader generation, shader assets, `yzc`, flake packaging, and CI. `ghostty_cursor_registry.rs` remains the Yazelix-specific loader for `~/.config/yazelix_ghostty_cursors/settings.jsonc`; legacy embedded cursor settings are rejected rather than migrated in the main runtime.

Selected repository and package name: `yazelix-cursors` / `yazelix_cursors`

Previous alternatives considered:

- `yazelix-ghostty-cursors`: accurate while Ghostty was the only shader target, but misleading once Yazelix Terminal consumes the same shader assets
- `ghostty-cursors`: overclaims generic Ghostty ecosystem ownership
- `ghostty_yazelix_cursors`: clear for Ghostty-only users, but puts the terminal brand before the Yazelix product brand

## Scope

- `.#yazelix_cursors` flake package
- generated cursor palette shaders from the extracted `yazelix_ghostty_cursors_default.toml`
- generated Ghostty-compatible cursor effect shaders
- terminal target metadata for Ghostty, yzxterm, Rio, Ratty, and protocol cursor positions
- example Ghostty config snippets under the package output
- `yzc init`, `yzc list`, `yzc list-targets`, `yzc inspect`, and `yzc generate ghostty`
- standalone cursor config at `~/.config/yazelix_ghostty_cursors/settings.jsonc`
- generated Ghostty include at `~/.config/yazelix_ghostty_cursors/ghostty.conf`
- license and provenance notes for shipped and adapted shaders
- a stable import path back into the main Yazelix runtime

## Behavior

- The package output contains complete GLSL files under `share/yazelix/yazelix_cursors/shaders/`
- The package output contains examples under `share/yazelix/yazelix_cursors/examples/`
- The package output contains `bin/yzc`
- The package passthru exposes `yazelixCursorPackageContract` with the exact shader root, generated effect root, required targets, and forbidden stale shader files
- The package and runtime shader roots must not include `build_shaders.nu`; shader generation is Rust-owned
- `yzc list-targets` exposes the child-owned target model for Ghostty, yzxterm, Rio, Ratty, and protocol cursor positions
- Users opt in by running `yzc init`, `yzc generate ghostty`, then adding `config-file = ~/.config/yazelix_ghostty_cursors/ghostty.conf` to their Ghostty config
- `yzc init` creates `~/.config/yazelix_ghostty_cursors/settings.jsonc` and does not overwrite an existing config
- `yzc generate ghostty` copies packaged shaders into `~/.config/yazelix_ghostty_cursors/shaders/`, regenerates data-driven palette and effect shaders from the standalone settings, and writes `~/.config/yazelix_ghostty_cursors/ghostty.conf`
- The package does not edit user Ghostty config files
- The package provides standalone random resolution when `yzc generate ghostty` runs; it does not provide Yazelix runtime per-window reroll behavior
- The package is generated from the same cursor registry and Ghostty palette generator used by Yazelix

## Release Policy

- `yazelix_cursors` versions independently through the `luccahuguet/yazelix-cursors` repository
- Cursor schema changes must remain valid for the main `settings.jsonc` cursor object
- Preset removals need a normal Yazelix upgrade note because users may have copied shader paths or config examples
- Yazelix must pin an explicit flake input and Cargo revision when consuming `yazelix_cursors`

## Yazelix Consumption Boundary

Yazelix continues to own:

- main `~/.config/yazelix/settings.jsonc` runtime schema and config UI metadata
- generated Ghostty config materialization
- invoking runtime random cursor selection
- status-bar cursor widget facts
- terminal package selection
- runtime state placement for launch-scoped yzxterm shader snapshots

`yazelix_cursors` owns:

- cursor preset validation
- cursor registry resolution
- generated Ghostty-compatible palette shader content
- generated Ghostty-compatible effect shader content
- terminal target capability metadata
- standalone Ghostty include generation
- exported Ghostty shader files
- exported Ghostty examples
- standalone cursor config initialization
- public package naming and install instructions for non-Yazelix users
- shader provenance notes

The package must not depend on Zellij, Yazi, Helix, Yazelix pane orchestration, or the Yazelix runtime wrapper.

Yazelix must consume Ghostty-compatible shader assets from the locked `yazelix_cursors` package output. The main repository must not carry `configs/terminal_emulators/ghostty/shaders` as a mirrored source tree or invoke a shader build script during runtime materialization.

## Provenance

The shader direction is inspired by the public Ghostty cursor shader ecosystem, including `ghostty-cursor-shaders`. Yazelix-generated palette shaders are derived from `yazelix_ghostty_cursors_default.toml` and the first-party Ghostty materialization code. Vendored or adapted shader files in `luccahuguet/yazelix-cursors` must keep nearby provenance notes.

## Non-Goals

- Installing or editing a user's Ghostty config
- Exporting Yazelix terminal launcher behavior
- Owning terminal-specific launch paths through this package
- Exporting Yazelix config UI, Home Manager ownership, or runtime orchestration into the cursor repository

## New Terminal Target Criteria

Do not add another terminal target to `yazelix_cursors` until all of these are true:

- there is a terminal-native cursor effect surface with concrete user value
- Yazelix can generate or export it without side effects on user config files
- the terminal-specific package path does not weaken existing Ghostty-compatible shader behavior
- docs can show a small copy-paste install snippet for that terminal
- the feature can be disabled or omitted without affecting existing users

## Acceptance Cases

1. `nix build .#yazelix_cursors` produces a package output with complete cursor palette shaders.
2. The package output includes generated effect shaders such as `generated_effects/tail.glsl`.
3. The package output includes `bin/yzc`.
4. The package output and runtime shader root do not include `build_shaders.nu`.
5. `nix run .#yzc -- --help` shows the standalone command surface.
6. `yzc list-targets` reports `ghostty`, `yzxterm`, `rio`, `ratty`, and `protocol_cursor_positions`.
7. A package-installed `yzc --config-dir <tmp> init` creates standalone JSONC cursor settings.
8. A package-installed `yzc --config-dir <tmp> generate ghostty` writes a Ghostty include and generated shader files under `<tmp>`.
9. The exported shaders come from the extracted `yazelix_cursors` shader generator instead of a parallel hand-maintained package list.
10. `validate-child-release-transaction` rejects a reintroduced main-repo `configs/terminal_emulators/ghostty/shaders` tree or stale cursor package contract metadata.

## Verification

- `nix build .#yazelix_cursors`
- `nix run .#yzc -- --help`
- `nix run .#yzc -- list-targets`
- package-installed `yzc --config-dir <tmp> init`
- package-installed `yzc --config-dir <tmp> generate ghostty`
- `yzx_repo_validator validate-child-release-transaction`
