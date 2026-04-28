# Yazelix Delete-First Code Inventory

## Summary

This is not another repo-wide LOC scoreboard.

The current repo goal is simpler: delete as much remaining product-side Nushell
ownership as possible without losing real functionality. The useful question is
no longer "which subsystem is largest?" The useful question is "which surviving
Nushell owners can disappear next, and which ones are still honest Nushell
fits?"

The current ranked answer belongs in Beads. This inventory stays as the public
delete-first map and should not become an execution queue.

The previous inventory was stale in two important ways:

- it treated all subsystem LOC as equally actionable
- it still pointed at already deleted Nushell files such as `yzx/env.nu`,
  `yzx/run.nu`, `config_migrations.nu`, and
  `config_migration_transactions.nu`

This version uses size only to rank deletion lanes.

## Current Shape

These numbers are only here to size the next cuts, not to recreate the old
snapshot game:

- runtime-facing Nushell still carries about `11,838` code LOC across `96`
  files under `core`, `setup`, `utils`, `yzx`, `integrations`, and
  `zellij_wrappers`
- maintainer Nushell is still larger overall at about `14,939` code LOC across
  `51` files, but that is a separate trim lane from product-side Nushell
- the old compatibility registry and the tiny Zellij wrapper layer are already
  gone in the current pass, so the remaining bridge work is narrower than this
  inventory’s first version
- the current top-value remaining runtime cuts are launch-time bridge collapse,
  detached-launch probe extraction, deterministic Nu test migration, and the
  family-by-family re-evaluation of likely Nushell survivors

If a change does not delete or materially shrink one of the owners below, it is
not progress toward the current repo goal.

## How To Use This Inventory

1. Delete truly stale or ownerless surfaces first
2. Collapse bridge owners before porting new code
3. Count a Rust migration only when it deletes a Nushell owner end-to-end
4. Treat likely survivors as Nushell-owned unless a proposed port removes the
   owner cleanly instead of adding another wrapper

## Delete Now

The old zero-replacement doc deletes are done. `yazelix-k0f3` verified that
`docs/contracts/yzx_env_run_rust_owner_transition.md` and
`docs/contracts/yzx_command_surface_backend_coupling.md` are already gone from the
tracked tree, and `contracts_inventory.md` keeps them out of active planning.

There are no remaining pure runtime-code deletes with zero replacement work.
Most remaining product-side Nushell either bridges to an existing Rust owner or
still owns real behavior.

## Bridge Layer To Collapse

This is the first serious product-side Nushell deletion lane.

These files already sit beside Rust owners. The right move is not more wrappers.
The right move is one minimal transport layer plus Nushell-only human rendering
where that still adds product value.

| Bridge cluster | Current Nu owners | Approx size | Why it should collapse | Success metric |
| --- | --- | ---: | --- | --- |
| Helper transport and error surfaces | `nushell/scripts/utils/yzx_core_bridge.nu` | smaller than the old `config_parser.nu` bridge owner | The generic argv/JSON/error bridge no longer belongs to `config_parser.nu`. One shared transport layer is enough. | One minimal helper transport remains, but per-domain policy and duplicate error shaping disappear |
| Config, state, env, and preflight shims | dev-only `config_normalize_test_helpers.nu`, `runtime_env.nu`, direct preflight calls, and the fact helpers consumed by `menu.nu`, `popup.nu`, startup, launch, and setup | smaller after `config_state.nu` deletion, the runtime-env request cut, the full-config product owner cut, and the `config_parser.nu` deletion | Rust already owns config-state, runtime-env, transient-pane facts, startup facts, active-surface bootstrap, and config normalization. The old product parser bridge is gone; the surviving Nu normalize shim is dev-only test support. | Keep shrinking machine-owned request shaping where Rust can be the single owner. Do not recreate a generic product-side full-config bridge. |
| Doctor and install report shims | private `doctor_fix.nu` plus caller-local install-ownership helper invocations | much smaller after the public Rust `yzx doctor` cut and the install-ownership bridge collapse | Rust now owns the public doctor report path, summary/rendering, JSON emission, live Zellij plugin-health reporting, and env-derived install-ownership request construction. The surviving Nu work is a private fix helper plus desktop/restart UX that calls the Rust owner directly. | Keep the private fix helper narrow and do not recreate a shared doctor-report bridge, install-ownership bridge, or public Nu doctor owner |
| Runtime materialization bridge | deleted `core/materialization_orchestrator.nu`; caller-local startup and doctor glue remains | `0` live bridge-owner lines | Rust now owns the runtime materialization lifecycle and env-derived request construction through `runtime-materialization.* --from-env` | Do not recreate a shared Nu materialization bridge; keep only caller-local startup profile/failure rendering and doctor repair progress |
| Terminal launch-time compatibility seam | terminal materialization and Ghostty reroll helpers inside `core/launch_yazelix.nu` | smaller than the deleted standalone wrapper; now only launch-adjacent helpers remain | Rust already owns generated terminal writes and Ghostty shader/config generation. The standalone `terminal_configs.nu` wrapper is gone. The surviving Nu seam is terminal filtering, user-facing summary text, and the launch-time Ghostty reroll bridge. | Keep narrowing `launch_yazelix.nu` until only irreducible launch-time compatibility logic remains. Do not recreate a standalone terminal materialization owner |
| Helix shared path and launch compatibility seam | shared Helix path helpers in `utils/common.nu` plus Rust edit/import owners and `shells/posix/yazelix_hx.sh` | standalone wrapper deleted; only shared path helpers and direct Rust-owner call sites remain | Rust already owns Helix template merge policy, reveal-binding enforcement, generated-file writes, import-notice state, and the public `yzx edit` / `yzx import` command bodies. The standalone `helix_config_merger.nu` wrapper is gone. The surviving Nu/POSIX seam is path truth and launch orchestration, not a second generator owner. | Keep Helix path truth shared and non-owning. Do not recreate a standalone Helix materialization wrapper |

The point of this lane is not to move one more JSON call into Rust. The point is
to stop keeping one Nushell owner per Rust helper command.

## Big Rust-Port Targets

There is no obvious new large product-side Nushell generator family left to
port. The recent Yazi, Zellij, runtime-materialization, terminal-write, and
Helix-write cuts already moved the real generated-file ownership into Rust.

What survives now is smaller wrapper-collapse work, not one more honest
full-owner Rust migration lane.

Current audit outcome:

- Runtime materialization is no longer a big-port target; `yazelix-ulb2.9` landed the full-owner Rust cut, deleted `generated_runtime_state.nu`, and `yazelix-q0o9.2` deleted the surviving shared Nu bridge
- Yazi is no longer a big-port target; `yazelix-ulb2.3.1` landed the full-owner Rust cut, and `yazelix-vf0u.1` deleted the surviving setup wrapper
- Zellij is no longer a big-port target; `yazelix-ulb2.3.2` landed the full-owner Rust cut, and `yazelix-vf0u.2` deleted the surviving setup wrapper after the semantic/base/settings/plugin/generation/layout Nu owners were already gone
- terminal is no longer a big-port target; `terminal_renderers.nu` and `terminal_configs.nu` are gone, generated terminal writes already live in Rust, and the only surviving Nu seam is narrow launch-time compatibility logic inside `launch_yazelix.nu`
- Helix is no longer a big-port target; generated merge/write/import-notice ownership already lives in Rust, `helix_config_merger.nu` is gone, and the surviving seam is just shared path truth plus launch/editor/import orchestration around the Rust owner
- if a proposal cannot delete one of those surviving wrappers end-to-end, it is not a meaningful migration bead

## Likely Nushell Survivors

An aggressive Rust migration still leaves some surfaces that are cleaner in
Nushell today because they are shell-bound, process-bound, or mostly human UX.

| Surface family | Current owners | Why it is still a good Nushell fit today |
| --- | --- | --- |
| Public CLI UX for intentionally Nu-owned commands | `yzx/menu.nu`, `yzx/dev.nu` | The compatibility registry `core/yazelix.nu` is deleted. Root help, palette inventory, and extern metadata now come from Rust, and `yzx edit`, `yzx import`, `yzx screen`, `yzx tutor`, `yzx whats_new`, `yzx home_manager`, `yzx config`, `yzx cwd`, `yzx reveal`, `yzx doctor`, `yzx why`, `yzx sponsor`, and `yzx keys` are already Rust-owned. The remaining Nu command bodies should stay in Nu only while they remain the honest `fzf`, popup, or maintainer shell boundary |
| Launch and startup process orchestration | `core/launch_yazelix.nu`, `core/start_yazelix.nu`, `core/start_yazelix_inner.nu`, `utils/terminal_launcher.nu`, `shells/posix/*.sh` | This path is shell and process heavy. It is not a good Rust target unless a new deterministic subcore appears that deletes a real owner |
| Shell initializer generation and shellhook setup | `setup/initializers.nu`, `setup/environment.nu` | The typed runtime-env layer already moved to Rust. What remains is external-tool init generation, shell-specific text shaping, bridge sync, logging, and welcome-shellhook orchestration, so a smaller Rust insertion would only add bridge code without deleting the owner |
| Human rendering and front-door UX | `setup/welcome.nu`, `utils/front_door_runtime.nu`, `yzx/menu.nu` | The old renderer/data owners are gone. Rust now owns screen playback, Game of Life, tutor, upgrade-summary shaping, edit, and import. The remaining Nu floor is mostly startup-shell presentation and process handoff |
| Runtime integration glue around live tools | Rust `workspace_commands.rs`, Rust `zellij_commands.rs`, and `zellij_wrappers/*.nu` | The Yazi/editor integration owners are now Rust-owned. The surviving Nu floor is limited to the sidebar Yazi launcher and popup/menu wrappers that still sit at a real shell/process boundary |

These are "likely" survivors, not protected surfaces. A port is still fair game
if it removes the owner cleanly. The bar is just much higher here than in the
bridge and materialization lanes.

## Recommended Delete Order

1. Collapse launch-time request assembly that still survives beside Rust-owned terminal and Ghostty materialization
2. Move the detached terminal probe shell body out of Nushell and into one fixed POSIX helper
3. Port deterministic Nu tests that now defend Rust-owned logic into Rust-owned test buckets
4. Re-evaluate likely Nushell survivors family by family and record explicit no-go boundaries before reopening any broad Rust-port idea

The first public metadata cut landed under `yazelix-ulb2.7`: root help, palette
inventory, and generated externs no longer probe the Nushell command tree. The
follow-up extern lifecycle cut deleted `nushell_externs.nu` by moving generated
extern bridge sync into `yzx_core yzx-command-metadata.sync-externs`. The
follow-up compatibility-registry cut landed under `yazelix-f7hz`, deleting
`core/yazelix.nu` and leaving direct Nu family modules as the only surviving
internal helper path under the Rust root.
