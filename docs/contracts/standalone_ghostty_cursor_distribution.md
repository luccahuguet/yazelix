# Standalone Ghostty Cursor Distribution

## Summary

Yazelix exports its Ghostty cursor shaders as a standalone package surface for Ghostty users who want the Yazelix cursor look without adopting the full Yazelix runtime.

## Why

The cursor shaders have value outside a Yazelix window. The standalone surface should reuse the same generated shader assets as Yazelix, while avoiding implicit ownership of a user's Ghostty config.

## Scope

- `.#ghostty_cursor_shaders` flake package
- generated cursor palette shaders from `yazelix_cursors_default.toml`
- generated Ghostty cursor effect shaders
- example config snippets under the package output

## Behavior

- The package output contains complete GLSL files under `share/yazelix/ghostty_cursor_shaders/shaders/`
- The package output contains examples under `share/yazelix/ghostty_cursor_shaders/examples/`
- Users opt in by adding explicit `custom-shader` lines to their own Ghostty config
- The package does not edit user Ghostty config files
- The package does not provide Yazelix runtime random reroll behavior
- The package is generated from the same cursor registry and Ghostty materialization code used by Yazelix

## Non-Goals

- Installing or editing a user's Ghostty config
- Exporting Yazelix terminal launcher behavior
- Supporting Kitty, WezTerm, Alacritty, or Foot through this package
- Extracting the cursor registry into a separate repository

## Acceptance Cases

1. `nix build .#ghostty_cursor_shaders` produces a package output with complete cursor palette shaders.
2. The package output includes generated effect shaders such as `generated_effects/tail.glsl`.
3. The package output includes at least one example Ghostty config containing absolute `custom-shader` paths into the package output.
4. The exported shaders come from the Yazelix cursor materialization path instead of a parallel hand-maintained package list.

## Verification

- `nix build .#ghostty_cursor_shaders`
