# Standalone Yazelix Cursors Distribution

## Summary

`yazelix_cursors` is the standalone Yazelix cursor package for Ghostty users who want the Yazelix cursor shader look without adopting the full Yazelix workspace runtime.

The primary flake package is `.#yazelix_cursors`. `.#ghostty_cursor_shaders` remains a compatibility package attribute for the same output because that name already existed as the first standalone cursor surface. The package exposes the standalone `yzc` binary, and the flake exposes `.#yzc` as an app for direct CLI use.

## Decision

Yazelix should ship `yazelix_cursors` as a standalone package surface now, but should not move the cursor registry and shader generator into a separate repository yet.

The current extraction boundary is a flake package produced from the main Yazelix source tree plus an internal Rust module boundary. `yazelix_cursors.rs` owns reusable cursor registry parsing, validation, resolution, and Ghostty palette shader generation. `ghostty_cursor_registry.rs` remains the Yazelix-specific loader for `settings.jsonc` and config paths. That gives Ghostty users a real external-user artifact while keeping generated Ghostty config behavior, config UI, and Yazelix status-widget integration on one side of the repo boundary until those seams settle.

Selected name: `yazelix_cursors`

Alternatives considered:

- `yazelix_ghostty_cursors`: more explicit, but too narrow if the package later grows previews, docs, or non-Ghostty experiments
- `ghostty_yazelix_cursors`: clear for Ghostty users, but puts the terminal brand before the Yazelix product brand

## Scope

- `.#yazelix_cursors` flake package
- compatibility `.#ghostty_cursor_shaders` flake package attribute
- generated cursor palette shaders from `yazelix_cursors_default.toml`
- generated Ghostty cursor effect shaders
- example Ghostty config snippets under the package output
- `yzc init`, `yzc list`, `yzc inspect`, and `yzc generate ghostty`
- standalone cursor config at `~/.config/yazelix_cursors/settings.jsonc`
- generated Ghostty include at `~/.config/yazelix_cursors/ghostty.conf`
- license and provenance notes for shipped and adapted shaders
- a stable import path back into the main Yazelix runtime

## Behavior

- The package output contains complete GLSL files under `share/yazelix/yazelix_cursors/shaders/`
- The package output contains examples under `share/yazelix/yazelix_cursors/examples/`
- The package output keeps `share/yazelix/ghostty_cursor_shaders` as a compatibility path pointing at the branded package root
- The package output contains `bin/yzc`
- Users opt in by running `yzc init`, `yzc generate ghostty`, then adding `config-file = ~/.config/yazelix_cursors/ghostty.conf` to their Ghostty config
- `yzc init` creates `~/.config/yazelix_cursors/settings.jsonc` and does not overwrite an existing config
- `yzc generate ghostty` copies packaged shaders into `~/.config/yazelix_cursors/shaders/`, regenerates data-driven palette and effect shaders from the standalone settings, and writes `~/.config/yazelix_cursors/ghostty.conf`
- The package does not edit user Ghostty config files
- The package provides standalone random resolution when `yzc generate ghostty` runs; it does not provide Yazelix runtime per-window reroll behavior
- The package is generated from the same cursor registry and Ghostty palette generator used by Yazelix

## Release Policy

- `yazelix_cursors` versions with Yazelix while it is built from the main Yazelix repository
- Cursor schema changes must remain valid for the main `settings.jsonc` cursor object
- Preset removals need a normal Yazelix upgrade note because users may have copied shader paths or config examples
- A future separate repository is justified only after the package has stable docs, preview assets, and an API small enough to consume without importing Yazelix workspace behavior

## Yazelix Consumption Boundary

Yazelix continues to own:

- `settings.jsonc` cursor schema and config UI metadata
- generated Ghostty config materialization
- invoking runtime random cursor selection
- status-bar cursor widget facts
- terminal package selection

`yazelix_cursors` owns:

- cursor preset validation
- cursor registry resolution
- generated Ghostty palette shader content
- standalone Ghostty include generation
- exported Ghostty shader files
- exported Ghostty examples
- standalone cursor config initialization
- public package naming and install instructions for non-Yazelix users
- shader provenance notes

The package must not depend on Zellij, Yazi, Helix, Yazelix pane orchestration, or the Yazelix runtime wrapper.

## Provenance

The shader direction is inspired by the public Ghostty cursor shader ecosystem, including `ghostty-cursor-shaders`. Yazelix-generated palette shaders are derived from `yazelix_cursors_default.toml` and the first-party Ghostty materialization code. Vendored or adapted shader files under `configs/terminal_emulators/ghostty/shaders/` must keep nearby provenance notes.

## Non-Goals

- Installing or editing a user's Ghostty config
- Exporting Yazelix terminal launcher behavior
- Supporting Kitty, WezTerm, Alacritty, or Foot through this package
- Moving the cursor registry into a separate repository before the Ghostty-only package proves its maintenance shape

## Non-Ghostty Criteria

Do not add another terminal to `yazelix_cursors` until all of these are true:

- there is a terminal-native cursor effect surface with concrete user value
- Yazelix can generate or export it without side effects on user config files
- the terminal-specific package path does not weaken the Ghostty contract
- docs can show a small copy-paste install snippet for that terminal
- the feature can be disabled or omitted without affecting Ghostty users

## Acceptance Cases

1. `nix build .#yazelix_cursors` produces a package output with complete cursor palette shaders.
2. `nix build .#ghostty_cursor_shaders` resolves to the same standalone cursor package for compatibility.
3. The package output includes generated effect shaders such as `generated_effects/tail.glsl`.
4. The package output includes `bin/yzc`.
5. `nix run .#yzc -- --help` shows the standalone command surface.
6. A package-installed `yzc --config-dir <tmp> init` creates standalone JSONC cursor settings.
7. A package-installed `yzc --config-dir <tmp> generate ghostty` writes a Ghostty include and generated shader files under `<tmp>`.
8. The exported shaders come from the Yazelix cursor materialization path instead of a parallel hand-maintained package list.

## Verification

- `nix build .#yazelix_cursors`
- `nix build .#ghostty_cursor_shaders`
- `nix run .#yzc -- --help`
- package-installed `yzc --config-dir <tmp> init`
- package-installed `yzc --config-dir <tmp> generate ghostty`
