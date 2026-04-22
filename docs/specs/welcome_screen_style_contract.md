# Welcome Screen Style Contract

## Summary

This spec defines the retained front-door style surface for the Yazelix
welcome screen and `yzx screen`. It exists so `yazelix-7krc.2` can delete
real renderer ownership without silently dropping styles, reviving soft
compatibility aliases, or leaving `random` ambiguous again.

The current retained shape is:

- welcome keeps `static`, `logo`, `boids`,
  `game_of_life_gliders`, `game_of_life_oscillators`,
  `game_of_life_bloom`, and `random`
- `yzx screen` keeps the same surface except `static`
- `random` means one of the three explicit Game of Life variants, not any
  animated style
- `logo` and `boids` remain live as explicit opt-in styles, but not part of
  the default `random` pool

This bead is planning-first. It does not shrink the renderer yet. It defines
the retained product contract and the deletion budget that `yazelix-7krc.2`
must satisfy before the lane counts as honest.

## Scope

- `yazelix_default.toml`
- `config_metadata/main_config_contract.toml`
- `home_manager/module.nix`
- `docs/yzx_cli.md`
- `nushell/scripts/utils/ascii_art.nu`
- `nushell/scripts/setup/welcome.nu`
- `nushell/scripts/yzx/screen.nu`
- `nushell/scripts/dev/test_yzx_screen_commands.nu`

Out of scope:

- terminal-launch ownership
- startup shellhook ownership outside welcome display/gating
- a full Rust port of the renderer

## Retained Surface

| Style | Welcome | `yzx screen` | Status | Reason |
| --- | --- | --- | --- | --- |
| `static` | yes | no | live | explicit low-motion resting frame for startup only |
| `logo` | yes | yes | live | explicit branded reveal and preview style |
| `boids` | yes | yes | live | explicit alternate animated preview style |
| `game_of_life_gliders` | yes | yes | live | retained default-family live simulation variant |
| `game_of_life_oscillators` | yes | yes | live | retained default-family live simulation variant |
| `game_of_life_bloom` | yes | yes | live | retained default-family live simulation variant |
| `random` | yes | yes | live | picks one retained Game of Life variant and never `static`, `logo`, or `boids` |
| `game_of_life` | no | no | live non-goal | deleted compatibility alias; do not revive without an explicit contract change |

## Contract Items

#### FRONT-001
- Type: behavior
- Status: live
- Owner: config metadata in `yazelix_default.toml`,
  `config_metadata/main_config_contract.toml`, and
  `home_manager/module.nix`, plus runtime style resolution in
  `nushell/scripts/utils/ascii_art.nu` and `nushell/scripts/yzx/screen.nu`
- Statement: The retained public style surface is exactly `static`, `logo`,
  `boids`, `game_of_life_gliders`, `game_of_life_oscillators`,
  `game_of_life_bloom`, and `random` for welcome, and the same minus `static`
  for `yzx screen`
- Verification: validator
  `nu nushell/scripts/dev/validate_config_surface_contract.nu`; automated
  `nushell/scripts/dev/test_yzx_screen_commands.nu`
  (`test_game_of_life_seed_layouts_are_distinct`,
  `test_screen_style_rejects_static`); validator
  `nu nushell/scripts/dev/validate_specs.nu`
- Source: `docs/yzx_cli.md`;
  `docs/specs/setup_shellhook_welcome_terminal_canonicalization_audit.md`
- Deletion note: `yazelix-7krc.2` may delete renderer code, but it must not
  silently delete one of these retained styles without first changing this
  contract

#### FRONT-002
- Type: behavior
- Status: live
- Owner: `nushell/scripts/utils/ascii_art.nu`
- Statement: `random` means one of the three explicit Game of Life variants:
  `game_of_life_gliders`, `game_of_life_oscillators`, or
  `game_of_life_bloom`. It is not a bucket over `logo`, `boids`, or `static`
- Verification: automated
  `nushell/scripts/dev/test_yzx_screen_commands.nu`
  (`test_random_screen_style_resolves_only_to_retained_game_of_life_pool`);
  validator `nu nushell/scripts/dev/validate_specs.nu`
- Source: `docs/yzx_cli.md`; `yazelix_default.toml`
- Deletion note: if a future contract wants a broader random pool, it must
  change the retained surface explicitly instead of smuggling the change in as
  renderer cleanup

#### FRONT-003
- Type: failure_mode
- Status: live
- Owner: `nushell/scripts/yzx/screen.nu` plus
  `nushell/scripts/utils/ascii_art.nu`
- Statement: `yzx screen` rejects `static` and only accepts animated screen
  styles
- Verification: automated
  `nushell/scripts/dev/test_yzx_screen_commands.nu`
  (`test_screen_style_rejects_static`); validator
  `nu nushell/scripts/dev/validate_specs.nu`
- Source: `docs/yzx_cli.md`

#### FRONT-004
- Type: behavior
- Status: live
- Owner: `nushell/scripts/utils/ascii_art.nu` Game of Life state helpers plus
  `nushell/scripts/yzx/screen.nu`
- Statement: The retained Game of Life screen styles stay live and width-aware:
  they render a rolling state instead of replaying a short canned logo loop,
  and the screen cycle omits the resting logo frame
- Verification: automated
  `nushell/scripts/dev/test_yzx_screen_commands.nu`
  (`test_game_of_life_screen_cycle_stays_bounded_and_omits_resting_logo`,
  `test_game_of_life_screen_state_rolls_forward`); validator
  `nu nushell/scripts/dev/validate_specs.nu`
- Source: `docs/specs/setup_shellhook_welcome_terminal_canonicalization_audit.md`

#### FRONT-005
- Type: boundary
- Status: live
- Owner: shell-local playback and waiting in
  `nushell/scripts/setup/welcome.nu` and
  `nushell/scripts/utils/ascii_art.nu`
- Statement: Welcome playback remains a shell-owned terminal boundary that is
  width-aware, interruptible, and explicit about skip versus launch gating even
  if deterministic frame generation later moves into a smaller owner
- Verification: manual startup review of current-shell and `yzx enter`
  flows; unverified direct welcome-skip/message automation exit bead
  `yazelix-7krc.2`
- Source: `docs/specs/setup_shellhook_welcome_terminal_canonicalization_audit.md`
- Notes: current automated coverage is indirect and does not directly defend
  welcome skip/logging/message composition

## Deletion Budget For `yazelix-7krc.2`

`yazelix-7krc.2` counts as success only if the following owner seams disappear
or collapse materially.

### `setup/welcome.nu` seams that must disappear

- the `parse_yazelix_config` dependency
- `get_session_info`
- `get_terminal_info`
- config rediscovery inside `build_welcome_message`

Budget judgment:

- `build_welcome_message` may survive only if it consumes explicit startup
  facts passed by the caller instead of reparsing config
- `poll_for_welcome_keypress`, `show_welcome_art`, and `show_welcome` are the
  honest shell-bound seams and may survive

### `ascii_art.nu` seams that must disappear or collapse

- independent style-policy ownership spread across:
  - `WELCOME_STYLE_VALUES`
  - `ANIMATED_WELCOME_STYLE_VALUES`
  - `SCREEN_STYLE_VALUES`
  - `get_welcome_style_random_pool`
  - `resolve_welcome_style`
  - `resolve_screen_style`
- the stale public `get_animated_ascii_art` export

Budget judgment:

- those style-policy symbols may survive only as views over one canonical style
  table or equivalent single owner
- `yazelix-7krc.2` must not leave welcome and screen with separate policy
  owners after the cut
- live renderer helpers for retained styles may survive, but only after the
  style-policy owner cluster above stops being duplicated and ad hoc

### Success metric

`yazelix-7krc.2` is honest only if all of these are true:

1. `welcome.nu` no longer reparses config or rediscovers terminal/session facts
2. style-policy ownership is singular instead of split across multiple lists
   and resolvers
3. the retained public surface from `FRONT-001` and `FRONT-002` still holds
4. no plain `game_of_life` compatibility alias is revived
5. the surviving owner is materially smaller than the current
   `welcome.nu` plus `ascii_art.nu` stack

## Verification Gaps To Carry Forward

- there is still no direct default-lane test for welcome skip/logging/message
  composition
- `logo` and `boids` playback are live, but still lack direct executable
  defenses comparable to the Game of Life tests
- `FRONT-005` is still only manually defended until `yazelix-7krc.2` closes
  the welcome-specific regression gap or records a narrower stop condition

## Stop Conditions

`yazelix-7krc.2` must stop and record a no-go if any of these turn out to be
true:

- deleting the duplicated style-policy seam requires reviving a compatibility
  alias such as plain `game_of_life`
- the only way to shrink `welcome.nu` is to move shell-local waiting, keypress,
  and skip gating behind a fake Rust wrapper
- the renderer can shrink only by silently deleting one of the retained styles
  instead of changing this contract explicitly

## Traceability

- Bead: `yazelix-7krc.1`
- Informed by: `docs/specs/setup_shellhook_welcome_terminal_canonicalization_audit.md`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
- Defended by: `nu nushell/scripts/dev/test_yzx_screen_commands.nu`
