# Yazelix Delete-First Code Inventory

## Summary

This is not another repo-wide LOC scoreboard.

The current repo goal is simpler: delete as much remaining product-side Nushell
ownership as possible without losing real functionality. The useful question is
no longer "which subsystem is largest?" The useful question is "which surviving
Nushell owners can disappear next, and which ones are still honest Nushell
fits?"

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
- the current Nu bridge layer around `yzx_core` and `yzx_control` is only about
  `1.5k` raw lines, but it is the highest-leverage product-side deletion lane
  because Rust already owns the typed work behind it

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
`docs/specs/yzx_env_run_rust_owner_transition.md` and
`docs/specs/yzx_command_surface_backend_coupling.md` are already gone from the
tracked tree, and `spec_inventory.md` keeps them out of active planning.

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
| Config, state, env, and preflight shims | `config_parser.nu`, `config_state.nu`, `runtime_env.nu`, plus direct preflight calls in `core/start_yazelix.nu`, `core/launch_yazelix.nu`, and `yzx/launch.nu` | smaller after `runtime_contract_checker.nu` deletion | Rust already owns config-state, runtime-env, and runtime-contract computation. The generic preflight veneer is gone, `config_parser.nu` is config-normalize specific, and the surviving Nu work is mostly shell execution plus a smaller amount of request shaping. | Keep shrinking machine-owned request shaping where Rust can be the single owner; leave only execution and user-facing rendering in Nu |
| Doctor and install report shims | `doctor_report_bridge.nu` plus `doctor.nu` | smaller after the public `yzx status` Rust owner cut deleted `status_report.nu` and the old per-report shims | Rust already owns the structured findings. One shared doctor-report transport seam is enough, but `doctor.nu` still owns the renderer, live Zellij plugin health checks, and fix flow. | Keep one report-transport seam and keep only the human rendering helpers and fix actions that still add product value. `yazelix-osco.2` says this is still not enough to justify a public Rust doctor cut |
| Runtime materialization bridge | deleted `core/materialization_orchestrator.nu`; caller-local startup and doctor glue remains | `0` live bridge-owner lines | Rust now owns the runtime materialization lifecycle and env-derived request construction through `runtime-materialization.* --from-env` | Do not recreate a shared Nu materialization bridge; keep only caller-local startup profile/failure rendering and doctor repair progress |
| Terminal launch-time compatibility seam | terminal materialization and Ghostty reroll helpers inside `core/launch_yazelix.nu` | smaller than the deleted standalone wrapper; now only launch-adjacent helpers remain | Rust already owns generated terminal writes and Ghostty shader/config generation. The standalone `terminal_configs.nu` wrapper is gone. The surviving Nu seam is terminal filtering, user-facing summary text, and the launch-time Ghostty reroll bridge. | Keep narrowing `launch_yazelix.nu` until only irreducible launch-time compatibility logic remains. Do not recreate a standalone terminal materialization owner |
| Helix shared path and launch compatibility seam | shared Helix path helpers in `utils/common.nu` plus direct Rust helper consumers in `doctor_report_bridge.nu`, `yzx/edit.nu`, `yzx/import.nu`, and `shells/posix/yazelix_hx.sh` | standalone wrapper deleted; only shared path helpers and direct Rust-owner call sites remain | Rust already owns Helix template merge policy, reveal-binding enforcement, generated-file writes, and import-notice state. The standalone `helix_config_merger.nu` wrapper is gone. The surviving Nu/POSIX seam is path truth and launch/editor/import orchestration, not a second generator owner. | Keep Helix path truth shared and non-owning. Do not recreate a standalone Helix materialization wrapper |

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
| Public CLI UX for intentionally Nu-owned commands | `core/yazelix.nu`, `core/yzx_workspace.nu`, `core/yzx_session.nu`, `core/yzx_doctor.nu`, `yzx/edit.nu`, `yzx/import.nu`, `yzx/menu.nu`, `yzx/popup.nu`, `yzx/screen.nu`, `yzx/tutor.nu`, `yzx/whats_new.nu`, `yzx/desktop.nu` | Root help, palette inventory, and extern metadata now come from Rust. `yzx home_manager`, `yzx config`, `yzx why`, `yzx sponsor`, and `yzx keys` are now Rust-owned too. The remaining Nu command bodies should stay in Nu unless Rust becomes the single public owner for a whole relevant family |
| Launch and startup process orchestration | `core/launch_yazelix.nu`, `core/start_yazelix.nu`, `core/start_yazelix_inner.nu`, `utils/terminal_launcher.nu`, `shells/posix/*.sh` | This path is shell and process heavy. It is not a good Rust target unless a new deterministic subcore appears that deletes a real owner |
| Shell initializer generation and shellhook setup | `setup/initializers.nu`, `setup/environment.nu` | The typed runtime-env layer already moved to Rust. What remains is external-tool init generation, shell-specific text shaping, bridge sync, logging, and welcome-shellhook orchestration, so a smaller Rust insertion would only add bridge code without deleting the owner |
| Human rendering and front-door UX | `utils/doctor.nu`, `utils/config_report_rendering.nu`, `utils/ascii_art.nu`, `utils/upgrade_summary.nu` | The hard part here is user-facing prose and presentation, not typed decision logic |
| Runtime integration glue around live tools | `integrations/*.nu`, `zellij_wrappers/*.nu`, `utils/editor_launch_context.nu` | These files mostly adapt to Zellij, Yazi, and editor process behavior rather than model deterministic domain state |

These are "likely" survivors, not protected surfaces. A port is still fair game
if it removes the owner cleanly. The bar is just much higher here than in the
bridge and materialization lanes.

## Recommended Delete Order

1. Collapse the remaining `yzx_core` and `yzx_control` Nu bridge owners into one minimal transport layer
2. Keep wrapper-collapse follow-ups honest: terminal, Helix, Yazi, Zellij, runtime materialization, and extern lifecycle are already Rust-owned or wrapper-deleted lanes, not new "big Rust port" targets
3. Continue public command ownership only where the next cut deletes a real parser or command-body owner
4. Revisit stale specs only when a new inventory check finds a concrete removed owner still listed as live guidance

The first public metadata cut landed under `yazelix-ulb2.7`: root help, palette
inventory, and generated externs no longer probe the Nushell command tree. The
follow-up extern lifecycle cut deleted `nushell_externs.nu` by moving generated
extern bridge sync into `yzx_core yzx-command-metadata.sync-externs`. The next
public command cut only counts if it deletes or demotes another real public
owner, such as one of the surviving `core/yzx_*.nu` families still routed
directly from `rust_core/yazelix_core/src/bin/yzx.rs`.
