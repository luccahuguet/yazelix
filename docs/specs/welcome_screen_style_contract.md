# Welcome Screen Style Contract

## Summary

This spec defines the retained front-door style surface for the Yazelix welcome
screen and `yzx screen` after the Rust owner cut in `yazelix-lj7z.8`.

The renderer, style resolution, and Game of Life engine are now Rust-owned.
Nu keeps only the startup-shell sequencing and the tiny runtime handoff used by
welcome/startup callers.

The retained public shape is:

- welcome keeps `static`, `logo`, `boids`, `boids_predator`,
  `boids_schools`, `boids_flow`, `mandelbrot`,
  `game_of_life_gliders`, `game_of_life_oscillators`,
  `game_of_life_bloom`, and `random`
- `yzx screen` keeps the same animated surface except `static`
- welcome `random` splits evenly across Game of Life, boids, and Mandelbrot
  families while never choosing `static` or `logo`
- `yzx screen random` remains the direct Game of Life preview selector
- `boids` remains a legacy alias for `boids_flow`

## Scope

- `yazelix_default.toml`
- `config_metadata/main_config_contract.toml`
- `home_manager/module.nix`
- `docs/yzx_cli.md`
- `rust_core/yazelix_core/src/front_door_render.rs`
- `rust_core/yazelix_core/src/front_door_commands.rs`
- `rust_core/yazelix_core/src/upgrade_summary.rs`
- `nushell/scripts/setup/welcome.nu`
- `nushell/scripts/utils/front_door_runtime.nu`
- Rust front-door tests under `rust_core/yazelix_core`

Out of scope:

- terminal-launch ownership outside welcome playback/gating
- popup/menu/editor command families

## Retained Surface

| Style | Welcome | `yzx screen` | Status | Reason |
| --- | --- | --- | --- | --- |
| `static` | yes | no | live | explicit low-motion resting frame for startup only |
| `logo` | yes | yes | live | explicit branded reveal and preview style |
| `boids` | yes | yes | legacy alias | compatibility alias for `boids_flow` |
| `boids_predator` | yes | yes | live | predator/prey flocking variant |
| `boids_schools` | yes | yes | live | species-separated flocking variant |
| `boids_flow` | yes | yes | live | baseline flow-field flocking variant |
| `mandelbrot` | yes | yes | live | Seahorse/Misiurewicz spiral zoom |
| `game_of_life_gliders` | yes | yes | live | retained default-family live simulation variant |
| `game_of_life_oscillators` | yes | yes | live | retained default-family live simulation variant |
| `game_of_life_bloom` | yes | yes | live | retained default-family live simulation variant |
| `random` | yes | yes | live | welcome picks one retained animation family; `yzx screen` picks one retained Game of Life variant |
| `game_of_life` | no | no | deleted compatibility alias | do not revive without an explicit contract change |

## Contract Items

#### FRONT-001
- Type: behavior
- Status: live
- Owner: config metadata plus Rust style resolution in
  `front_door_render.rs` and `front_door_commands.rs`
- Statement: The retained public style surface is exactly `static`, `logo`,
  `boids`, `boids_predator`, `boids_schools`, `boids_flow`, `mandelbrot`,
  `game_of_life_gliders`, `game_of_life_oscillators`, `game_of_life_bloom`,
  and `random` for welcome, and the same minus `static` for `yzx screen`
- Verification: `yzx_repo_validator validate-config-surface-contract`;
  Rust `front_door_render` and `front_door_commands` tests;
  `yzx_repo_validator validate-specs`

#### FRONT-002
- Type: behavior
- Status: live
- Owner: Rust random-pool policy in `front_door_render.rs`
- Statement: welcome `random` splits evenly across the Game of Life, boids,
  and Mandelbrot families. The Game of Life family rotates through
  `game_of_life_gliders`, `game_of_life_oscillators`, and
  `game_of_life_bloom`; the boids family rotates through `boids_predator`,
  `boids_schools`, and `boids_flow`; the Mandelbrot family resolves to
  `mandelbrot`. It is not a bucket over `static` or `logo`
- Verification: automated Rust `front_door_render` tests;
  validator `yzx_repo_validator validate-specs`

#### FRONT-003
- Type: failure_mode
- Status: live
- Owner: Rust `yzx screen` parsing and routing in `front_door_commands.rs`
- Statement: `yzx screen` rejects `static` and only accepts animated screen
  styles
- Verification: automated Rust `front_door_render` and
  `front_door_commands` tests; validator `yzx_repo_validator validate-specs`

#### FRONT-004
- Type: behavior
- Status: live
- Owner: Rust Game of Life engine in `front_door_render.rs`
- Statement: The retained Game of Life screen styles stay live and width-aware:
  they render a rolling state instead of replaying a canned resting frame
- Verification: automated Rust `front_door_render` tests;
  validator `yzx_repo_validator validate-specs`

#### FRONT-005
- Type: boundary
- Status: live
- Owner: Nu startup-shell gating in `setup/welcome.nu` plus the tiny runtime
  handoff in `front_door_runtime.nu`
- Statement: Welcome playback remains explicit about skip versus launch gating,
  and the startup shell still owns the final prompt/logging boundary even
  though rendering moved to Rust
- Verification: automated Rust `yzx_control_front_door.rs`;
  manual startup review for current-shell and `yzx enter` flows

## Remaining Nu Floor

The front-door cut is already landed. The only surviving Nu boundary here is:

- startup-shell sequencing and skip/logging behavior
- the handoff from startup shell code into the Rust renderer

The renderer/data owner does not get to come back. Any future front-door work
should either:

1. delete more of `setup/welcome.nu` and `front_door_runtime.nu`, or
2. leave them as the tiny irreducible shell boundary

## Stop Conditions

- Do not revive the plain `game_of_life` alias
- Do not reintroduce a second renderer or style-policy owner in Nu
- Do not move shell-local prompt/keypress gating into a fake Rust wrapper just
  to say the file moved

## Traceability

- Bead: `yazelix-7krc.1`
- Bead: `yazelix-lj7z.8`
- Bead: `yazelix-roe0`
- Bead: `yazelix-df9z`
- Defended by: `yzx_repo_validator validate-specs`
- Defended by: `cargo test -p yazelix_core --manifest-path rust_core/Cargo.toml`
