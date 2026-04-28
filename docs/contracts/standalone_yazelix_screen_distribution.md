# Standalone Yazelix Screen Distribution

## Summary

Yazelix exposes a standalone `yazelix_screen` package and flake app for terminal users who want the screen animations without starting a Yazelix workspace.

## Why

The screen renderer is already a reusable terminal animation crate. A small package-first binary proves the standalone boundary in-repo before any separate repository or broader terminal integration is considered.

## Scope

- `.#yazelix_screen` flake package
- `.#yazelix_screen` flake app
- standalone `yazelix_screen` binary from the `yazelix_screen` Rust crate
- boids, Mandelbrot, and Game of Life animation engines

## Behavior

- The binary runs outside Zellij and outside a Yazelix session.
- The binary does not read `yazelix.toml` or session config snapshots.
- The binary owns a small explicit CLI: optional style plus optional Game of Life cell style.
- The binary enters alternate-screen/raw mode, renders frames, responds to terminal resize, exits on keypress, and restores terminal state on normal exit.
- The binary supports the animation-engine styles available in the screen crate: `boids`, `boids_predator`, `boids_schools`, `mandelbrot`, `game_of_life_gliders`, `game_of_life_oscillators`, `game_of_life_bloom`, and `random`.

## Non-Goals

- Replacing `yzx screen`
- Reading or mutating Yazelix user config
- Providing shell hooks or terminal-specific startup snippets
- Extracting `yazelix_screen` into a separate repository
- Recreating the branded welcome logo surface outside `yazelix_core`

## Acceptance Cases

1. `cargo test -p yazelix_screen --manifest-path rust_core/Cargo.toml` covers standalone argument parsing, style resolution, and sizing boundaries.
2. `nix build .#yazelix_screen` produces a package with `bin/yazelix_screen`.
3. `nix run .#yazelix_screen -- --help` prints the standalone CLI without launching a Yazelix session.
4. Existing `yzx screen` behavior remains owned by `yazelix_core` and unchanged by the standalone package.

## Verification

- `cargo test -p yazelix_screen --manifest-path rust_core/Cargo.toml`
- `nix build .#yazelix_screen`
- `nix run .#yazelix_screen -- --help`
