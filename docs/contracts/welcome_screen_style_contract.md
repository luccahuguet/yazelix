# Welcome Screen Style Contract

## Summary

This contract defines the retained front-door style surface for the Yazelix
welcome screen and `yzx screen`.

The renderer and style resolution are Rust-owned. The main repo owns Yazelix
product policy and runtime integration; the `yazelix_screen` child crate owns
terminal animation engines, automata, generation logic, shared random
animation-family resolution, file-backed Kitty frame sequence playback, and the
magician source GIF plus host/cache frame generation helpers. Rust owns startup
welcome sequencing, skip/logging behavior, upgrade-summary display, and the
handoff into the renderer.

The retained public shape is:

- welcome keeps `static`, `logo`, `boids`, `boids_predator`,
  `boids_schools`, `mandelbrot`, `magician`,
  `game_of_life_gliders`, `game_of_life_oscillators`,
  `game_of_life_bloom`, and `random`
- `yzx screen` keeps the same animated surface except `static`
- welcome `random` splits evenly across Game of Life, boids, and Mandelbrot
  families while never choosing `static` or `logo`; it includes `magician` only
  when the runtime can resolve Kitty graphics support and magician frame assets
- `yzx screen random` uses the same animation-family pool as welcome `random`
  while never choosing `static` or `logo`
- `boids` remains an alias for `boids_predator`

## Scope

- `settings_default.jsonc`
- `config_metadata/main_config_contract.toml`
- `home_manager/module.nix`
- `docs/yzx_cli.md`
- `rust_core/yazelix_core/src/front_door_render.rs`
- `rust_core/yazelix_core/src/front_door_commands.rs`
- `rust_core/yazelix_core/src/upgrade_summary.rs`
- `https://github.com/luccahuguet/yazelix-screen`
- Rust front-door tests under `rust_core/yazelix_core`

Out of scope:

- terminal-launch ownership outside welcome playback/gating
- popup/menu/editor command families

## Retained Surface

| Style | Welcome | `yzx screen` | Status | Reason |
| --- | --- | --- | --- | --- |
| `static` | yes | no | live | explicit low-motion resting frame for startup only |
| `logo` | yes | yes | live | explicit branded reveal and preview style |
| `boids` | yes | yes | alias | compatibility alias for `boids_predator` |
| `boids_predator` | yes | yes | live | predator/prey flocking variant |
| `boids_schools` | yes | yes | live | species-separated flocking variant |
| `boids_flow` | no | no | deleted | removed after the flow-field variant looked odd in the welcome surface |
| `mandelbrot` | yes | yes | live | Seahorse/Misiurewicz spiral zoom |
| `magician` | yes | yes | live | attributed 1mposter ASCII magician GIF-derived animation rendered through Kitty graphics |
| `game_of_life_gliders` | yes | yes | live | retained default-family live simulation variant |
| `game_of_life_oscillators` | yes | yes | live | retained default-family live simulation variant |
| `game_of_life_bloom` | yes | yes | live | retained default-family live simulation variant |
| `random` | yes | yes | live | welcome and `yzx screen` pick from the same retained non-image animation-family pool, with `magician` admitted only when assets and Kitty graphics are available |
| `game_of_life` | no | no | deleted compatibility alias | do not revive without an explicit contract change |

## Contract Items

#### FRONT-001
- Type: behavior
- Status: live
- Owner: config metadata plus Rust style resolution in
  `front_door_render.rs` and `front_door_commands.rs`
- Statement: The retained public style surface is exactly `static`, `logo`,
  `boids`, `boids_predator`, `boids_schools`, `mandelbrot`, `magician`,
  `game_of_life_gliders`, `game_of_life_oscillators`, `game_of_life_bloom`,
  and `random` for welcome, and the same minus `static` for `yzx screen`
- Verification: `yzx_repo_validator validate-config-surface-contract`;
  Rust `front_door_render` and `front_door_commands` tests;
  `yzx_repo_validator validate-contracts`

#### FRONT-002
- Type: behavior
- Status: live
- Owner: shared random-pool policy in `yazelix_screen`, consumed by
  `front_door_render.rs`
- Statement: welcome `random` and `yzx screen random` split evenly across the
  same default Game of Life, boids, and Mandelbrot families. The Game of Life
  family rotates through
  `game_of_life_gliders`, `game_of_life_oscillators`, and
  `game_of_life_bloom`; the boids family rotates through `boids_predator`,
  and `boids_schools`; the Mandelbrot family resolves to
  `mandelbrot`. The `magician` family resolves to `magician` only after the
  runtime proves Kitty graphics support plus runtime/cached/generated magician
  PNG frame availability. It is not a bucket over `static` or `logo`
- Verification: automated Rust `front_door_render` tests;
  validator `yzx_repo_validator validate-contracts`

#### FRONT-003
- Type: failure_mode
- Status: live
- Owner: Rust `yzx screen` parsing and routing in `front_door_commands.rs`
- Statement: `yzx screen` rejects `static` and only accepts animated screen
  styles
- Verification: automated Rust `front_door_render` and
  `front_door_commands` tests; validator `yzx_repo_validator validate-contracts`

#### FRONT-004
- Type: boundary
- Status: live
- Owner: `yazelix_screen` owns the magician source GIF, optional cached PNG
  frame generation, and reusable Kitty frame sequence rendering; Yazelix
  packaging links the child-owned source GIF into the runtime asset tree and
  `front_door_render.rs` owns product-specific gating and error classification
- Statement: `magician` renders the attributed GIF-derived PNG frame assets
  through Kitty graphics. Missing runtime/cached frame assets, unavailable host
  ImageMagick for frame generation, or unavailable Kitty graphics support
  produce explicit errors for explicit `magician`; `random` skips `magician`
  when those conditions are not satisfied instead of selecting a broken style
- Verification: automated Rust `front_door_render` tests; manual `yzx screen
  magician` or `yzx screen --internal-welcome magician` review in the packaged
  Ghostty/Ratty runtime

#### FRONT-005
- Type: behavior
- Status: live
- Owner: Rust Game of Life engine in `front_door_render.rs`
- Statement: The retained Game of Life screen styles stay live and width-aware:
  they render a rolling state instead of replaying a canned resting frame
- Verification: automated Rust `front_door_render` tests;
  validator `yzx_repo_validator validate-contracts`

#### FRONT-006
- Type: boundary
- Status: live
- Owner: Rust startup path in `launch_commands/enter.rs`
- Statement: Welcome playback remains explicit about skip versus launch gating,
  and Rust owns the final prompt/logging boundary before Zellij handoff
- Verification: automated Rust `yzx_control_front_door.rs`;
  manual startup review for current-shell and `yzx enter` flows

## Remaining Front-Door Floor

The front-door renderer and startup sequence are Rust-owned. The remaining
boundary here is:

- `yazelix_screen` owns reusable animation engines and magician frame assets
- `front_door_render.rs` owns product-specific style resolution and rendering
- `launch_commands/enter.rs` owns startup skip/logging/prompt sequencing

The renderer/data owner does not get to come back in Nu.

## Stop Conditions

- Do not revive the plain `game_of_life` alias
- Do not reintroduce a second renderer or style-policy owner in Nu
- Do not fork the random animation-family pool back into `front_door_render.rs`
- Do not reintroduce shell-local prompt/keypress gating as a startup fallback

## Traceability
- Defended by: `yzx_repo_validator validate-contracts`
- Defended by: `cargo test -p yazelix_core --manifest-path rust_core/Cargo.toml`
