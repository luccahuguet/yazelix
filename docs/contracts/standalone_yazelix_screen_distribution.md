# Standalone Yazelix Screen Distribution

## Summary

Yazelix exposes a standalone `yzs` command for terminal users who want the screen animations without starting a Yazelix workspace.

## Why

The screen renderer is already a reusable terminal animation crate. A small user-facing binary proves the standalone boundary in-repo before any separate repository or broader terminal integration is considered.

## Scope

- `.#yazelix_screen` flake package
- `.#yazelix_screen` flake app
- `.#yzs` flake package and app aliases
- standalone `yzs` binary from the `yazelix_screen` Rust crate
- crate-local `rust_core/yazelix_screen/README.md`
- standalone library examples under `rust_core/yazelix_screen/examples/`
- boids, Mandelbrot, and Game of Life animation engines

## Behavior

- The `yzs` binary runs outside Zellij and outside a Yazelix session.
- The binary does not read `settings.jsonc` or session config snapshots.
- The binary owns a small explicit CLI: optional style plus optional Game of Life cell style.
- The binary enters alternate-screen/raw mode, renders frames, responds to terminal resize, exits on keypress, and restores terminal state on normal exit.
- The binary supports the animation-engine styles available in the screen crate: `boids`, `boids_predator`, `boids_schools`, `mandelbrot`, `game_of_life_gliders`, `game_of_life_oscillators`, `game_of_life_bloom`, and `random`.
- Library examples run without Yazelix runtime/session/config state and demonstrate one-frame rendering plus bounded style playback.
- Yazelix users keep using `yzx screen`; standalone users can run `yzs` directly from a vanilla terminal.

## Non-Goals

- Replacing `yzx screen`
- Reading or mutating Yazelix user config
- Providing shell hooks or terminal-specific startup snippets
- Extracting `yazelix_screen` into a separate repository
- Recreating the branded welcome logo surface outside `yazelix_core`

## Acceptance Cases

1. `cargo test -p yazelix_screen --manifest-path rust_core/Cargo.toml` covers standalone argument parsing, style resolution, and sizing boundaries.
2. `cargo check -p yazelix_screen --manifest-path rust_core/Cargo.toml --examples` proves the standalone library examples compile.
3. `cargo run -p yazelix_screen --manifest-path rust_core/Cargo.toml --example render_once` prints one frame without entering a Yazelix session.
4. `nix build .#yazelix_screen` produces a package with `bin/yzs`.
5. `nix run .#yzs -- --help` prints the standalone CLI without launching a Yazelix session.
6. Existing `yzx screen` behavior remains owned by `yazelix_core` and unchanged by the standalone package.

## Release Policy

`yazelix_screen` uses its crate-local `Cargo.toml` version while it lives in the monorepo. External releases should use SemVer. If component tags are used before a separate repository exists, use a component-scoped shape such as `yazelix_screen-v0.1.0`.

Publishing to crates.io or moving to a separate repository is optional until the standalone README, examples, and release process justify the maintenance cost.

## Verification

- `cargo test -p yazelix_screen --manifest-path rust_core/Cargo.toml`
- `cargo check -p yazelix_screen --manifest-path rust_core/Cargo.toml --examples`
- `cargo run -p yazelix_screen --manifest-path rust_core/Cargo.toml --bin yzs -- --help`
- `cargo run -p yazelix_screen --manifest-path rust_core/Cargo.toml --example render_once`
- `nix build .#yazelix_screen`
- `nix run .#yzs -- --help`
