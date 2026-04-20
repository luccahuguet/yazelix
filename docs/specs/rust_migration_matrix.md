# Rust Migration Matrix

## Summary

The first helper-backed Rust slices are already in place.

That changes the migration question. The next useful question is no longer
"where could Rust help?" The next useful question is "which remaining Nushell
owners can Rust delete end-to-end?"

Current delete-first order:

1. collapse the Nu bridge layer around `yzx_core` and `yzx_control`
2. move one generator and materialization family to full Rust ownership, not
   just Rust planning
3. keep shell and process orchestration, public CLI UX, and text-heavy rendering
   in Nushell unless a port deletes those owners whole
4. treat any broader Rust public-CLI rewrite as a late move after the bridge and
   materialization owners are already much smaller

This is not a big-bang `yzx` rewrite plan. It is a delete-first owner-reduction
plan.

## Why

Yazelix already moved a large amount of typed core logic into Rust:

- `yzx_core` now owns config normalization, config-state computation and
  recording, runtime-contract evaluation, startup preflight evaluation,
  canonical runtime-env computation, materialization planning and repair
  evaluation, status and doctor report evaluation, install ownership
  evaluation, and Yazi/Zellij render-plan computation
- `yzx_control` now owns the public control-plane leaf parsing and execution for
  `yzx env`, `yzx run`, and `yzx update*`

The remaining product-side Nushell cost is concentrated in two places:

- Nu bridge owners that still shape requests, translate errors, and assemble
  per-command report shims around those Rust owners
- full generator and materialization families that still own real writes,
  orchestration, and template policy in Nushell

The migration mistake to avoid now is adding more Rust while keeping the same Nu
owner. A migration only counts if the Nu owner disappears or becomes clearly
thinner.

## Scope

- classify the main surviving product-side Nushell owners by deletion value
- identify which lanes are bridge collapse, which are true full-owner ports, and
  which are likely Nushell survivors
- define when a broader Rust public-CLI move is worth evaluating
- give later beads one concrete delete-first sequence instead of reopening the
  keep versus port question from scratch

## Delete-First Rules

Use these rules before starting any Rust lane:

- count a migration only if it deletes a Nushell owner end-to-end
- collapse bridge owners before porting new helper slices
- prefer one full-owner materialization lane over many small helper insertions
- keep Nushell where the hard part is shell execution, process orchestration,
  integration glue, or human-facing text
- do not justify a port with "the file is long" or "Rust is nearby"

## Current Live Rust Owners

`yzx_core` is already the typed owner for these helper commands:

- `config.normalize`
- `config-state.compute`
- `config-state.record`
- `runtime-contract.evaluate`
- `startup-launch-preflight.evaluate`
- `runtime-env.compute`
- `runtime-materialization.plan`
- `runtime-materialization.repair-evaluate`
- `runtime-materialization.apply`
- `status.compute`
- `doctor-config.evaluate`
- `doctor-helix.evaluate`
- `doctor-runtime.evaluate`
- `install-ownership.evaluate`
- `zellij-render-plan.compute`
- `yazi-render-plan.compute`

`yzx_control` is already the public leaf owner for:

- `yzx env`
- `yzx run`
- `yzx update*`

That means the remaining roadmap must be written around the Nu files that still
surround those Rust owners, not around deleted `yzx env.nu` and `yzx run.nu`
wrappers.

## Migration Matrix

| Surface | Current owners | Delete-first read | Recommendation | Timing and beads |
| --- | --- | --- | --- | --- |
| Bridge transport and error shaping | `config_parser.nu` plus the small per-command report bridges | High-leverage deletion lane with relatively low semantic risk | Collapse now. Keep one minimal transport layer, not one policy-bearing Nu owner per Rust helper command. | Current lane: `yazelix-ulb2.5.3` |
| Config, state, env, and preflight shims | `config_state.nu`, `runtime_env.nu`, `runtime_contract_checker.nu` | Rust already owns the typed computation, but Nu still shapes too much request and classification logic | Move request shaping and machine classification fully into Rust where possible. Leave only execution and final user rendering in Nu. | Current lane: `yazelix-ulb2.5.3` |
| Status, doctor, and install report bridges | `status_report.nu`, `doctor.nu`'s inline config-doctor bridge, `doctor_helix_report.nu`, `doctor_runtime_report.nu`, `install_ownership_report.nu` | Still too many Nu owners for already structured Rust outputs | Keep collapsing these toward one shared report transport seam. Keep human rendering in Nu only where it still adds product value. | Reduced under `yazelix-ulb2.5.3`; keep shrinking the surviving owners |
| Runtime materialization orchestrator | `generated_runtime_state.nu`, `config_state.nu`, `atomic_writes.nu` | Biggest remaining mixed control-plane owner | Only port if Rust becomes the full owner of freshness, expected artifacts, managed writes, and recorded state. A plan-only port is not enough now. | Main deletion lane: `yazelix-ulb2.3` |
| Yazi materialization family | Rust owner: `yazi-materialization.generate`; surviving Nu wrapper: `setup/yazi_config_merger.nu` | The real Nu owner family is gone. Only a thin compatibility wrapper remains. | Keep the wrapper thin or delete it. Do not rebuild Yazi policy ownership in Nu. Dependency gate for the landed cut: in-house logic plus existing `serde` and `toml`; no new crates. | Landed under `yazelix-ulb2.3.1` |
| Zellij materialization family | Rust owner: `zellij-materialization.generate`; surviving Nu wrapper: `setup/zellij_config_merger.nu` | The real Nu owner family is gone. Rust owns base-config selection, semantic KDL extraction, layout rendering, plugin wasm sync, permission migration, popup-runner cleanup, and generation-state reuse. | Keep the wrapper thin or delete it. Do not rebuild Zellij policy ownership in Nu. Dependency gate for the landed cut: in-house logic plus existing `serde`, `serde_json`, `toml`, `sha2`, `thiserror`, and `lexopt`; no new crates. | Landed under `yazelix-ulb2.3.2` |
| Terminal, Helix, and initializer materialization | `utils/terminal_configs.nu`, `utils/terminal_renderers.nu`, `setup/helix_config_merger.nu`, `setup/initializers.nu`, `setup/environment.nu` | Meaningful deletion budget, but spread across several file families | Batch this with a real full-owner materialization move. Avoid isolated helper ports that leave the Nu writer layer intact. | Later `yazelix-ulb2.3` work |
| Launch and startup process orchestration | `core/launch_yazelix.nu`, `core/start_yazelix.nu`, `core/start_yazelix_inner.nu`, `utils/terminal_launcher.nu`, `shells/posix/*.sh` | Shell-bound and process-heavy, not the best next Rust target | Keep Nu and POSIX in v15.x. Reopen only if a new deterministic subcore appears that deletes a real owner. | No active deletion lane; historical stop note in `launch_bootstrap_rust_migration.md` |
| Public `yzx` root, help, and completion ownership | `core/yazelix.nu`, `yzx/*.nu`, `utils/nushell_externs.nu` | Only worth touching after the bridge and materialization owners are much smaller | Defer. A broader Rust root is only justified if it deletes the public registry owner and the extern metadata owner too. | Later planning only: `yazelix-2ex.1.11` |
| Workspace and session state | `rust_plugins/zellij_pane_orchestrator/`, `integrations/*.nu`, `zellij_wrappers/*.nu` | Already Rust where live session truth matters | Keep this separate from `rust_core`. Do not fold the pane-orchestrator track into the control-plane migration by habit. | Separate pane-orchestrator beads |
| Front-door UX and command-palette surfaces | `utils/ascii_art.nu`, `yzx/menu.nu`, `yzx/popup.nu`, `yzx/screen.nu`, `yzx/keys.nu`, `yzx/tutor.nu`, `utils/upgrade_summary.nu` | Mostly text-heavy or interactive UX | Keep Nushell unless a future port deletes an owner cleanly and improves the UX story at the same time. | Not a current Rust target |
| Distribution and host integration | `home_manager/`, `packaging/`, `shells/`, `yzx/desktop.nu`, `yzx/home_manager.nu` | Nix, POSIX, and UX-heavy by nature | Keep outside the current Rust migration. Rust may be packaged here, but it should not become the new owner by default. | Not a current Rust target |

## Full-Owner Materialization Rule

Generator and materialization work only counts as progress when the surviving
Nu owner disappears.

That means a Rust lane is successful only if it deletes or materially shrinks
the owner family that currently writes and coordinates the product files:

- `generated_runtime_state.nu`
- the Yazi generation family
- the Zellij generation family
- the terminal, Helix, and initializer generation family

If the end state is "Rust computes a plan, Nu still owns the same writer and
same orchestration policy," that is still transitional code, not the target
architecture.

## 2026-04-20 Generator Audit Outcome

`yazelix-ulb2.3` chose the next generator and materialization deletion lane,
and the first two full-owner cuts have landed:

- choose Yazi first, then land the full-owner Rust cut
- move Zellij second once the Yazi cut proves the full-owner pattern
- do not start with the fragmented terminal, Helix, and initializer family

Why Yazi first:

- it is the smallest coherent owner family still large enough to matter:
  about `739` raw Nu lines across `yazi_config_merger.nu`,
  `yazi_bundled_assets.nu`, and `yazi_user_overrides.nu`
- Rust already owns the typed Yazi render-plan semantics, so the next move can
  be a true owner transfer rather than a fresh helper insertion
- the outputs are bounded and product-shaped: `yazi.toml`, `theme.toml`,
  `keymap.toml`, `init.lua`, and bundled plugin or flavor assets
- it does not carry Zellij's extra KDL semantic-block merge, plugin wasm sync,
  generation fingerprinting, layout generation, or session-local command wiring

Why Zellij second:

- the deletion budget was bigger, but so was the surface area and product risk
- after the Yazi proof, Rust could take the whole owner family at once: config
  source selection, semantic KDL blocks, owned top-level settings, plugin
  artifact sync, layout generation, permission migration, popup-runner cleanup,
  and reuse fingerprints
- `zellij_config_merger.nu` is now a thin wrapper over `zellij-materialization.generate`

What landed in `yazelix-ulb2.3.1`:

- Rust now owns Yazi merge policy, legacy-override rejection, managed-file
  writes, bundled asset sync, placeholder rendering, and warm self-heal logic
- `yazi_bundled_assets.nu` and `yazi_user_overrides.nu` are deleted
- `yazi_config_merger.nu` is demoted to a thin compatibility wrapper over the
  Rust owner instead of a product-side policy owner

What landed in `yazelix-ulb2.3.2`:

- Rust now owns Zellij base-config selection, semantic block extraction,
  top-level setting replacement, layout rendering, stable plugin wasm sync,
  permission-cache migration, popup-runner cleanup, and generation-state reuse
- `zellij_semantic_blocks.nu`, `zellij_base_config.nu`,
  `zellij_owned_settings.nu`, `zellij_plugin_paths.nu`,
  `zellij_generation_state.nu`, and `layout_generator.nu` are deleted
- `zellij_config_merger.nu` is demoted to a thin compatibility wrapper over the
  Rust owner instead of a product-side policy owner

Why not terminal, Helix, and initializers first:

- the code is fragmented across several unrelated generated outputs instead of
  one clear owner family
- a first pass there would likely recreate the exact failure mode this bead was
  meant to prevent: many small helper ports without one real owner deletion

## Public CLI Rule

A broader Rust public-CLI move is not the next deletion lane.

It becomes worth evaluating only after the bridge and materialization owners are
already much smaller. If revisited later, the required deletion budget is:

- delete `core/yazelix.nu` as a public command-registry owner
- delete `nushell_externs.nu` as an authoritative command-metadata owner
- delete public Nushell wrapper parsing for at least one remaining command
  family beyond the already migrated `yzx_control` leaves

If a proposal keeps those surfaces and only adds a Rust dispatcher above them,
reject it.

## Non-goals

- treating helper insertion itself as success
- rewriting launch and startup orchestration just because Rust is available
- moving text-heavy user-facing UX into Rust without a clear ownership win
- folding the Rust pane orchestrator into `rust_core`
- porting maintainer tooling or package ownership into Rust by default

## Acceptance Cases

1. A maintainer can tell which remaining product-side Nu surfaces are bridge
   collapse work versus true full-owner Rust ports
2. The matrix names the real remaining owners instead of deleted `yzx env.nu`
   and `yzx run.nu` wrappers
3. The document makes it explicit that generator and materialization work only
   counts when it deletes the Nu owner end-to-end
4. The document makes it explicit that a broader Rust public-CLI move is late,
   not first

## Verification

- `nu nushell/scripts/dev/validate_specs.nu`
- manual review against `docs/specs/rust_nushell_bridge_contract.md`
- manual review against `docs/specs/cross_language_runtime_ownership.md`
- manual review against `docs/specs/v15_trimmed_runtime_contract.md`
- manual review against `docs/subsystem_code_inventory.md`

## Traceability

- Bead: `yazelix-kt5.1`
- Bead: `yazelix-ulb2.3`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`

## Open Questions

- After the bridge layer collapse, does `config_parser.nu` still deserve to
  survive as a named owner, or should helper transport become a smaller shared
  utility inside the remaining Nu owners
- After the Yazi and Zellij full-owner cuts, is the next better deletion lane
  the runtime materialization orchestrator itself or the fragmented
  terminal/Helix/initializer family
