# yzs

Standalone terminal screen animations from Yazelix

`yzs` is the standalone terminal command for Yazelix screen animations. It runs outside Zellij, outside a Yazelix session, and without reading `settings.jsonc`, generated state, Home Manager state, or runtime session data

The Rust crate remains `yazelix_screen`. It provides reusable terminal animation primitives for `yzs` and for Yazelix

## What It Contains

- Animation engines for Boids, Mandelbrot, and Game of Life
- Frame production through `ScreenFrameProducer`
- Terminal sizing helpers and alternate-screen rendering helpers
- A standalone `yzs` binary
- Small examples for library consumers

## User Command

Installed standalone command:

```bash
yzs --help
yzs mandelbrot
yzs game_of_life_bloom --cell-style dotted
```

Yazelix users get the same screen surface through the main command:

```bash
yzx screen mandelbrot
yzx screen random
```

## Repository Usage

From the Yazelix repository root:

```bash
cargo run --manifest-path rust_core/Cargo.toml -p yazelix_screen --bin yzs -- --help
cargo run --manifest-path rust_core/Cargo.toml -p yazelix_screen --bin yzs -- mandelbrot
cargo run --manifest-path rust_core/Cargo.toml -p yazelix_screen --bin yzs -- game_of_life_bloom --cell-style dotted
```

With Nix:

```bash
nix build .#yazelix_screen
nix run .#yzs -- --help
nix run .#yzs -- mandelbrot
```

Supported styles:

- `boids`
- `boids_predator`
- `boids_schools`
- `mandelbrot`
- `game_of_life_gliders`
- `game_of_life_oscillators`
- `game_of_life_bloom`
- `random`

## Library Examples

Render one frame without alternate-screen mode:

```bash
cargo run --manifest-path rust_core/Cargo.toml -p yazelix_screen --example render_once
```

Play a style for a bounded number of frames:

```bash
cargo run --manifest-path rust_core/Cargo.toml -p yazelix_screen --example play_style -- mandelbrot 90
cargo run --manifest-path rust_core/Cargo.toml -p yazelix_screen --example play_style -- boids_schools 120
cargo run --manifest-path rust_core/Cargo.toml -p yazelix_screen --example play_style -- game_of_life_gliders 80
```

The second argument is the frame count. The examples use only `yazelix_screen` APIs and standard Rust APIs

## Boundary With Yazelix

`yazelix_screen` owns reusable animation and terminal-rendering primitives. Yazelix product behavior stays outside the crate

The crate must not depend on:

- `yazelix_core`
- `settings.jsonc`
- generated Yazelix config or state
- Zellij session state
- Home Manager install state
- Yazelix command palette or workspace orchestration

`yazelix_core/front_door_render.rs` is the adapter that maps Yazelix settings, commands, diagnostics, and welcome/screen policy onto these primitives

`yzx screen` is the integrated Yazelix command. `yzs` is the standalone command for terminal users who want only the screen animations

## Release Policy

While this crate lives in the Yazelix monorepo, its version is the `version` field in `rust_core/yazelix_screen/Cargo.toml`

External releases should use SemVer. Once published, breaking changes to frame producer traits, style names, terminal-mode helpers, or cell-style parsing require a major version bump

If component tags are used before a separate repository exists, use a component-scoped tag shape such as:

```text
yazelix_screen-v0.1.0
```

Publishing to crates.io or moving to a separate repository is optional until the standalone audience, examples, and release process are valuable enough to justify the maintenance cost

## Verification

From the Yazelix repository root:

```bash
cargo test --manifest-path rust_core/Cargo.toml -p yazelix_screen
cargo check --manifest-path rust_core/Cargo.toml -p yazelix_screen --examples
cargo run --manifest-path rust_core/Cargo.toml -p yazelix_screen --bin yzs -- --help
cargo run --manifest-path rust_core/Cargo.toml -p yazelix_screen --example render_once
nix build .#yazelix_screen
nix run .#yzs -- --help
```
