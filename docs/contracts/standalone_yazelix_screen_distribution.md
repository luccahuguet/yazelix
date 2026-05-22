# Standalone Yazelix Screen Distribution

## Summary

Yazelix exposes the standalone `yzs` command from the external screen repository for terminal users who want the screen animations without starting a Yazelix workspace.

## Why

The screen renderer is a reusable terminal animation crate. Keeping it in an external screen repository gives standalone users a clear product surface while Yazelix consumes the same crate for integrated welcome and screen playback.

## Scope

- `yazelixScreen` flake input pointing at the external screen repository
- Yazelix `.#yzs` and `.#yazelix_screen` package/app aliases forwarded from that input
- standalone `yzs` binary from the external `yazelix_screen` Rust crate
- external screen README, CI, flake, Cargo lock, and examples
- boids, Mandelbrot, and Game of Life animation engines owned outside the Yazelix monorepo

## Behavior

- The `yzs` binary runs outside Zellij and outside a Yazelix session.
- The binary does not read `settings.jsonc` or session config snapshots.
- The binary owns a small explicit CLI: optional style plus optional Game of Life cell style.
- The binary enters alternate-screen/raw mode, renders frames, responds to terminal resize, exits on keypress, and restores terminal state on normal exit.
- The binary supports the animation-engine styles available in the screen crate: `boids`, `boids_predator`, `boids_schools`, `mandelbrot`, `game_of_life_gliders`, `game_of_life_oscillators`, `game_of_life_bloom`, and `random`.
- No explicit style means `random`.
- Library examples in the external crate run without Yazelix runtime/session/config state and demonstrate one-frame rendering plus bounded style playback.
- Yazelix users keep using `yzx screen`; standalone users can run `yzs` directly from a vanilla terminal.
- `yazelix_core` consumes `yazelix_screen` as an external Rust dependency instead of owning duplicate source.

## Non-Goals

- Replacing `yzx screen`
- Reading or mutating Yazelix user config
- Providing shell hooks or terminal-specific startup snippets
- Recreating the branded welcome logo surface outside `yazelix_core`
- Owning the standalone implementation source inside the Yazelix monorepo

## Acceptance Cases

1. In the external screen repository, `cargo test` covers standalone argument parsing, style resolution, and sizing boundaries.
2. In the external screen repository, `cargo check --examples` proves the standalone library examples compile.
3. In the external screen repository, `cargo run --example render_once` prints one frame without entering a Yazelix session.
4. In the external screen repository, `nix build .#yzs` produces a package with `bin/yzs`.
5. In Yazelix, `nix run .#yzs -- --help` prints the forwarded standalone CLI without launching a Yazelix session.
6. Existing `yzx screen` behavior remains owned by `yazelix_core` and unchanged by the standalone package.

## Release Policy

External screen releases should use SemVer tags such as `v0.1.0`. Yazelix pins the external source through both its flake lock and Rust dependency lock.

Publishing to crates.io is optional until the standalone audience and release process justify the maintenance cost.

## Verification

- `cargo test` in the external screen repository
- `cargo check --examples` in the external screen repository
- `cargo run --bin yzs -- --help` in the external screen repository
- `cargo run --example render_once` in the external screen repository
- `nix build .#yzs` in the external screen repository
- `nix run .#yzs -- --help`
