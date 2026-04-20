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

There are not many pure runtime-code deletes left with zero replacement work.
The easiest immediate deletes are stale planning surfaces and tiny bridge veneers
that no longer justify surviving as named owners.

| Surface | Approx size | Why delete now | Replacement or stop condition |
| --- | ---: | --- | --- |
| `docs/specs/yzx_env_run_rust_owner_transition.md` | doc only | Assumes public Nushell `yzx env` and `yzx run` owners still exist even though `yzx_control` already owns them | Delete it and point future planning at the current Rust control-plane owners |
| `docs/specs/yzx_command_surface_backend_coupling.md` | doc only | Treats deleted `yzx env.nu`, `yzx run.nu`, and `yzx packs.nu` surfaces as live planning anchors | Delete it and stop using it as live architecture guidance |
| `nushell/scripts/setup/yazi_config_merger.nu` | thin wrapper only | The full Yazi owner family is gone; this surviving file is now just a compatibility wrapper around Rust-owned Yazi materialization | Delete it when the direct Rust-owned surface becomes the only useful entrypoint |
| `nushell/scripts/setup/zellij_config_merger.nu` | thin wrapper only | The full Zellij owner family is gone; this surviving file is now just a compatibility wrapper around Rust-owned Zellij materialization | Delete it when the direct Rust-owned surface becomes the only useful entrypoint |

Most remaining product-side Nushell is not "delete now" code. It either bridges
to an existing Rust owner or still owns real behavior.

## Bridge Layer To Collapse

This is the first serious product-side Nushell deletion lane.

These files already sit beside Rust owners. The right move is not more wrappers.
The right move is one minimal transport layer plus Nushell-only human rendering
where that still adds product value.

| Bridge cluster | Current Nu owners | Approx size | Why it should collapse | Success metric |
| --- | --- | ---: | --- | --- |
| Helper transport and error surfaces | `nushell/scripts/utils/config_parser.nu` | `344` raw lines | `config_parser.nu` is the shared argv/JSON/error bridge for most `yzx_core` calls, which makes it a second owner for every helper command | One minimal helper transport remains, but per-domain policy and duplicate error shaping disappear |
| Config, state, env, and preflight shims | `config_state.nu`, `runtime_env.nu`, `runtime_contract_checker.nu` | about `405` raw lines | Rust already owns config-state, runtime-env, and runtime-contract computation, but Nu still shapes too much request and classification logic | Rust owns request shaping and machine classification; Nu keeps only execution and user-facing rendering |
| Status, doctor, and install report shims | `status_report.nu`, `doctor.nu`'s inline config-doctor bridge, `doctor_helix_report.nu`, `doctor_runtime_report.nu`, `install_ownership_report.nu` | about `404` raw lines after the recent bridge cuts | These surfaces still bounce structured Rust data through several Nu owners | Collapse to one report-transport seam and keep only the human rendering helpers that still add product value |
| Mixed bridge and orchestration seam | `generated_runtime_state.nu` | `224` raw lines | This file now straddles bridge logic and real product ownership, which makes it the key seam to either keep honestly or delete fully | Either keep it as an explicit Nushell owner or move freshness, expected artifacts, apply/write, and recorded state to Rust end-to-end |

The point of this lane is not to move one more JSON call into Rust. The point is
to stop keeping one Nushell owner per Rust helper command.

## Big Rust-Port Targets

This is where large product-side Nushell deletion is still available.

These lanes only count if Rust becomes the full owner and deletes the Nushell
owner end-to-end. A new Rust planner that still leaves the same Nu writer or
orchestrator in charge is not a real cut.

| Target lane | Current owners | Why it matters | What "done" means |
| --- | --- | --- | --- |
| Runtime materialization owner | `generated_runtime_state.nu`, `config_state.nu`, `atomic_writes.nu` | This is the central refresh and apply seam for generated state | Rust owns freshness, expected artifacts, managed writes, and recorded-state updates instead of Nu coordinating the same lifecycle |
| Terminal, Helix, and initializer materialization | `utils/terminal_configs.nu`, `utils/terminal_renderers.nu`, `setup/helix_config_merger.nu`, `setup/initializers.nu`, `setup/environment.nu` | Meaningful deletion budget remains here, but it is spread across several file families | Only port this as a real full-owner lane; micro-ports would strand the Nu writer layer and add more bridge code |

The main migration mistake to avoid is porting a planner while keeping the same
Nu orchestrator. The right unit of progress is deleting the owner, not moving a
subroutine.

Current audit outcome:

- Yazi is no longer a big-port target; `yazelix-ulb2.3.1` landed the full-owner Rust cut and deleted the real Nu owners
- Zellij is no longer a big-port target; `yazelix-ulb2.3.2` landed the full-owner Rust cut and deleted the semantic/base/settings/plugin/generation/layout Nu owners
- `setup/zellij_config_merger.nu` remains only as the command-surface wrapper around `zellij-materialization.generate`
- leave the fragmented terminal, Helix, and initializer family for later

## Likely Nushell Survivors

An aggressive Rust migration still leaves some surfaces that are cleaner in
Nushell today because they are shell-bound, process-bound, or mostly human UX.

| Surface family | Current owners | Why it is still a good Nushell fit today |
| --- | --- | --- |
| Public CLI UX for intentionally Nu-owned commands | `core/yazelix.nu`, `yzx/config.nu`, `yzx/edit.nu`, `yzx/import.nu`, `yzx/menu.nu`, `yzx/popup.nu`, `yzx/screen.nu`, `yzx/keys.nu`, `yzx/tutor.nu`, `yzx/whats_new.nu`, `yzx/desktop.nu`, `yzx/home_manager.nu` | Root help, palette inventory, and extern metadata now come from Rust. The remaining Nu command bodies should stay in Nu unless Rust becomes the single public owner for a whole relevant family |
| Launch and startup process orchestration | `core/launch_yazelix.nu`, `core/start_yazelix.nu`, `core/start_yazelix_inner.nu`, `utils/terminal_launcher.nu`, `shells/posix/*.sh` | This path is shell and process heavy. It is not a good Rust target unless a new deterministic subcore appears that deletes a real owner |
| Human rendering and front-door UX | `utils/doctor.nu`, `utils/config_report_rendering.nu`, `utils/ascii_art.nu`, `utils/upgrade_summary.nu` | The hard part here is user-facing prose and presentation, not typed decision logic |
| Runtime integration glue around live tools | `integrations/*.nu`, `zellij_wrappers/*.nu`, `utils/editor_launch_context.nu` | These files mostly adapt to Zellij, Yazi, and editor process behavior rather than model deterministic domain state |

These are "likely" survivors, not protected surfaces. A port is still fair game
if it removes the owner cleanly. The bar is just much higher here than in the
bridge and materialization lanes.

## Recommended Delete Order

1. Remove stale transition docs and tiny plan bridges
2. Collapse the `yzx_core` and `yzx_control` Nu bridge owners into one minimal transport layer
3. Choose one real full-owner materialization lane and finish it end-to-end
4. Continue public command ownership only where the next cut deletes a real parser or command-body owner

The first public metadata cut landed under `yazelix-ulb2.7`: root help, palette
inventory, and generated externs no longer probe the Nushell command tree. The
next cut only counts if it deletes or demotes another real public owner, such as
`core/yazelix.nu` for a command family, or removes the remaining
`nushell_externs.nu` compatibility wrapper entirely.
