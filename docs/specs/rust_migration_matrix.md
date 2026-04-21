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
  evaluation, Yazi/Zellij render-plan computation, Yazi/Zellij
  materialization generation, and shared `yzx` command metadata for root help,
  command-palette inventory, and generated Nushell externs
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
- `runtime-materialization.materialize`
- `runtime-materialization.repair`
- `status.compute`
- `doctor-config.evaluate`
- `doctor-helix.evaluate`
- `doctor-runtime.evaluate`
- `install-ownership.evaluate`
- `zellij-render-plan.compute`
- `yazi-render-plan.compute`
- `yazi-materialization.generate`
- `zellij-materialization.generate`
- `yzx-command-metadata.list`
- `yzx-command-metadata.externs`
- `yzx-command-metadata.help`

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
| Bridge transport and error shaping | `yzx_core_bridge.nu` plus the small per-command report bridges | High-leverage deletion lane with relatively low semantic risk. The generic helper transport no longer lives inside `config_parser.nu`. | Keep one minimal transport layer, not one policy-bearing Nu owner per Rust helper command. | Current lane: `yazelix-057w` |
| Config, state, env, and preflight shims | `config_parser.nu`, `config_state.nu`, `runtime_env.nu`, `runtime_contract_checker.nu` | Rust already owns the typed computation. `config_parser.nu` is now config-normalize specific, while the shared transport moved into `yzx_core_bridge.nu`. | Keep shrinking request shaping and machine classification where Rust can become the single typed owner. Leave only execution and final user rendering in Nu. | Current lane: `yazelix-057w` |
| Doctor and install report bridges | `doctor.nu`'s inline config-doctor bridge, `doctor_helix_report.nu`, `doctor_runtime_report.nu`, `install_ownership_report.nu` | Still too many Nu owners for already structured Rust outputs; `yzx status` has already moved to the Rust public owner and `status_report.nu` is gone | Keep collapsing these toward one shared report transport seam. Keep human rendering in Nu only where it still adds product value. | Current lane: `yazelix-osco` |
| Runtime materialization lifecycle | Rust owners: `runtime-materialization.plan`, `runtime-materialization.materialize`, `runtime-materialization.repair`; surviving Nu bridge: `core/materialization_orchestrator.nu` | Landed full-owner cut; the remaining Nu file is startup and doctor glue, not the lifecycle owner | Keep the bridge thin or delete it later. Do not recreate a second Nu lifecycle owner. | Landed under `yazelix-ulb2.9` |
| Yazi materialization family | Rust owner: `yazi-materialization.generate`; remaining Nu use is dev-only direct invocation | The real Nu owner family is gone, and the surviving setup wrapper is deleted. | Keep Rust as the single Yazi materialization owner. Do not recreate a public or product-side compatibility wrapper. Dependency gate for the landed cut: in-house logic plus existing `serde` and `toml`; no new crates. | Landed under `yazelix-ulb2.3.1`; wrapper deletion landed under `yazelix-vf0u.1` |
| Zellij materialization family | Rust owner: `zellij-materialization.generate`; remaining Nu use is direct product or dev invocation | The real Nu owner family is gone, the setup wrapper is deleted, and Rust still owns base-config selection, semantic KDL extraction, layout rendering, plugin wasm sync, permission migration, popup-runner cleanup, and generation-state reuse. | Keep Rust as the single Zellij materialization owner. Do not recreate a public or product-side compatibility wrapper. Dependency gate for the landed cut: in-house logic plus existing `serde`, `serde_json`, `toml`, `sha2`, `thiserror`, and `lexopt`; no new crates. | Landed under `yazelix-ulb2.3.2`; wrapper deletion landed under `yazelix-vf0u.2` |
| Terminal launch-time compatibility seam | Standalone wrapper deleted; surviving Nu helpers live in `core/launch_yazelix.nu` | Rust already owns generated terminal writes in `terminal_materialization.rs` plus Ghostty config/shader generation in `ghostty_materialization.rs`. The surviving Nu seam is launch-time supported-terminal filtering, user-facing summary text, and the Ghostty reroll bridge. | Treat this as a landed delete-wrapper lane, not a fresh full-owner port. Keep only irreducible launch-time compatibility logic in `launch_yazelix.nu` and do not recreate a separate terminal materialization owner. Dependency gate: no new crates; keep using the existing Rust owners and in-house helper logic. | Landed under `yazelix-ulb2.10.3` |
| Helix shared path and launch compatibility seam | Standalone wrapper deleted; shared Helix path truth now lives in `utils/common.nu`; adjacent consumers are `doctor_helix_report.nu`, `yzx/edit.nu`, `yzx/import.nu`, and `shells/posix/yazelix_hx.sh` | Rust already owns Helix template merge policy, reveal-binding enforcement, generated-file writes, import-notice state, and now the doctor-side expected contract build in `helix_materialization.rs`. The surviving Nu/POSIX seam is path truth and launch/editor/import orchestration, not a second generator owner. | Treat this as a landed wrapper-deletion lane. Keep Helix path truth shared and non-owning, and keep the POSIX launcher talking to the Rust helper directly. Dependency gate: no new crates; keep using existing `serde` and `toml` plus in-house helper extraction where needed. | Landed under `yazelix-ulb2.10.2` |
| Shell initializer generation and shellhook environment setup | `setup/initializers.nu`, `setup/environment.nu` | The deterministic runtime-env subcore already moved to Rust. What remains is external-tool init generation, shell-specific text normalization, bridge sync, startup profiling, log cleanup, executable-bit repair, and welcome-shellhook orchestration. | Keep Nushell-owned in v15.x. Reopen only if a future port can delete the surviving shellhook owner end-to-end instead of inserting one more text or bridge helper. | Decision locked by `yazelix-iwzn` |
| Launch and startup process orchestration | `core/launch_yazelix.nu`, `core/start_yazelix.nu`, `core/start_yazelix_inner.nu`, `utils/terminal_launcher.nu`, `shells/posix/*.sh` | Shell-bound and process-heavy, not the best next Rust target | Keep Nu and POSIX in v15.x. Reopen only if a new deterministic subcore appears that deletes a real owner. | No active deletion lane; historical stop note in `launch_bootstrap_rust_migration.md` |
| Public `yzx` root, help, completion, and palette inventory | Rust metadata owner: `command_metadata.rs`; surviving Nu command bodies: `core/yazelix.nu`, `core/yzx_*.nu`, `yzx/*.nu`; compatibility wrapper: `utils/nushell_externs.nu` | First metadata slice has landed: root help, generated externs, and menu catalog no longer probe the Nushell command tree. The old mixed `core/yazelix.nu` owner lump is now split into explicit internal families. | Keep shrinking only when the next cut deletes a real public parser or command-body owner. Do not rebuild a parallel Nu registry. | Follow-up lanes: `yazelix-2jkb.2` landed, `yazelix-2jkb.3` next |
| Workspace and session state | `rust_plugins/zellij_pane_orchestrator/`, `integrations/*.nu`, `zellij_wrappers/*.nu` | Already Rust where live session truth matters | Keep this separate from `rust_core`. Do not fold the pane-orchestrator track into the control-plane migration by habit. | Separate pane-orchestrator beads |

## Current Bridge-Collapse Budget

`yazelix-057w` is the live delete-first lane for the remaining `yzx_core` /
`yzx_control` bridge. The budget is:

- collapse now:
  - generic `yzx_core` helper transport and error shaping should live in one
    minimal bridge owner, not inside `config_parser.nu`
  - default `require_yazelix_runtime_dir` plus default-error-surface request
    boilerplate should not be re-owned in each report or preflight shim
- keep in Nushell for now:
  - config-specific normalize/report behavior in `config_parser.nu`
  - startup/launch and doctor-specific request assembly in
    `runtime_contract_checker.nu` where it still feeds shell/process orchestration
    or user-facing rendering
- defer to other lanes:
  - doctor/install report family collapse belongs to `yazelix-osco`
  - compatibility-wrapper deletion for `setup/yazi_config_merger.nu` and
    `setup/zellij_config_merger.nu` landed under `yazelix-vf0u`
  - launch/startup orchestration and materialization-orchestrator cleanup are
    separate lanes, not success criteria for `057w`
| Front-door UX and command-palette surfaces | `utils/ascii_art.nu`, `yzx/menu.nu`, `yzx/popup.nu`, `yzx/screen.nu`, `yzx/keys.nu`, `yzx/tutor.nu`, `utils/upgrade_summary.nu` | Mostly text-heavy or interactive UX | Keep Nushell unless a future port deletes an owner cleanly and improves the UX story at the same time. | Not a current Rust target |
| Distribution and host integration | `home_manager/`, `packaging/`, `shells/`, `yzx/desktop.nu`, `yzx/home_manager.nu` | Nix, POSIX, and UX-heavy by nature | Keep outside the current Rust migration. Rust may be packaged here, but it should not become the new owner by default. | Not a current Rust target |

## Full-Owner Materialization Rule

Generator and materialization work only counts as progress when the surviving
Nu owner disappears.

That means a Rust lane is successful only if it deletes or materially shrinks
the owner family that currently writes and coordinates the product files.

Recent landed examples:

- `yazelix-ulb2.9` deleted `generated_runtime_state.nu` and moved the runtime
  materialization lifecycle into Rust
- `yazelix-ulb2.3.1` deleted the real Yazi generation family
- `yazelix-ulb2.3.2` deleted the real Zellij generation family

There is no single remaining large generated-config family here anymore. After
the Yazi, Zellij, runtime-materialization, terminal-write, and Helix-write
cuts, the surviving work is smaller wrapper-collapse follow-up.

If the end state is "Rust computes a plan, Nu still owns the same writer and
same orchestration policy," that is still transitional code, not the target
architecture.

## 2026-04-21 Terminal And Helix Deletion-Budget Decision

`yazelix-ulb2.10.1` re-audited the terminal and Helix backlog after the more
recent Rust ownership cuts.

Decision:

- split terminal and Helix into separate cleanup lanes

Why split:

- `terminal_renderers.nu` is already gone, so the old "terminal family" framing
  is stale
- Rust already owns generated terminal writes and Ghostty asset generation
- Rust already owns Helix merge policy, reveal enforcement, generated-file
  writes, and import-notice state
- the surviving Nushell seams are different:
  - terminal: wrapper routing plus launch-time Ghostty reroll behavior
  - Helix: compatibility wrapper plus path-helper duplication and doctor caller
    dependence

Exact remaining delete targets:

- terminal lane:
  - landed: `nushell/scripts/utils/terminal_configs.nu` is deleted
  - surviving narrow launch-time compatibility logic in `nushell/scripts/core/launch_yazelix.nu`
- Helix lane:
  - landed: `nushell/scripts/setup/helix_config_merger.nu` is deleted
  - surviving shared path helpers in `nushell/scripts/utils/common.nu`
  - surviving direct-Rust call sites in `nushell/scripts/utils/doctor_helix_report.nu`,
    `nushell/scripts/yzx/edit.nu`, `nushell/scripts/yzx/import.nu`, and
    `shells/posix/yazelix_hx.sh`

Rust dependency gate before implementation:

- production crates:
  - no new crates
  - keep using existing `serde` and `toml` already present behind the landed
    Helix and terminal owners
- dev-only crates:
  - no new crates
- build in-house:
  - path-helper extraction if Helix callers still need shared path truth
  - caller reroutes for terminal launch/reroll flow
  - any remaining user-facing summary rendering
- rejected alternatives:
  - new TOML merge crates for Helix
  - terminal abstraction crates or template engines for terminal cleanup
  - another Rust helper layer that leaves the same Nu wrappers intact
- packaging impact:
  - none expected beyond the already shipped `yzx_core` helper and existing
    runtime assets
  - no new Nix inputs or package surfaces are justified by these wrapper
    cleanup lanes

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
- `zellij_config_merger.nu` was an intermediate thin wrapper over
  `zellij-materialization.generate` and is now deleted

What landed in `yazelix-ulb2.3.1`:

- Rust now owns Yazi merge policy, legacy-override rejection, managed-file
  writes, bundled asset sync, placeholder rendering, and warm self-heal logic
- `yazi_bundled_assets.nu` and `yazi_user_overrides.nu` are deleted
- the former `yazi_config_merger.nu` compatibility wrapper is now deleted too,
  so Rust is the only remaining product-side Yazi materialization owner

What landed in `yazelix-ulb2.3.2`:

- Rust now owns Zellij base-config selection, semantic block extraction,
  top-level setting replacement, layout rendering, stable plugin wasm sync,
  permission-cache migration, popup-runner cleanup, and generation-state reuse
- `zellij_semantic_blocks.nu`, `zellij_base_config.nu`,
  `zellij_owned_settings.nu`, `zellij_plugin_paths.nu`,
  `zellij_generation_state.nu`, and `layout_generator.nu` are deleted
- the former `zellij_config_merger.nu` compatibility wrapper is now deleted
  too, so Rust is the only remaining product-side Zellij materialization owner

Why not terminal and Helix first:

- the code is fragmented across several unrelated generated outputs instead of
  one clear owner family
- a first pass there would likely recreate the exact failure mode this bead was
  meant to prevent: many small helper ports without one real owner deletion

## 2026-04-21 Runtime Materialization Outcome

`yazelix-ulb2.9` closed the remaining runtime materialization owner seam.

What moved to Rust:

- materialization planning
- Yazi and Zellij generation sequencing
- recorded-state finalization
- repair flow, including noop versus regenerate decisions and missing-artifact
  recovery

What stayed in Nushell:

- `core/materialization_orchestrator.nu` as a thin startup and doctor bridge
- startup profile wrapping
- final Nu-facing progress and remediation rendering

What was deleted:

- `nushell/scripts/utils/generated_runtime_state.nu`
- the old private helper split around `runtime-materialization.apply` and
  `runtime-materialization.repair-evaluate`

## 2026-04-20 Initializer And Shellhook Audit Outcome

`yazelix-iwzn` audited `setup/initializers.nu` and `setup/environment.nu` after
the terminal materialization cut and the earlier `runtime-env.compute` move.

Conclusion:

- do not start a Rust initializer or shellhook port in v15.x
- `runtime-env.compute` already took the only obvious typed decision layer:
  canonical runtime environment planning
- `initializers.nu` is now mostly external-tool integration and generated
  shell-text ownership:
  - it runs `starship`, `zoxide`, `atuin`, `mise`, and `carapace`
  - it applies Nushell-specific text fixes such as the Starship right-prompt
    strip and PATH-preservation footer
  - it writes the per-shell initializer files and the aggregate
    `yazelix_init.*` files
- `environment.nu` is now mostly shellhook and startup orchestration:
  - shellhook profiling metadata
  - shell selection for initializer generation
  - state and log directory management
  - stale-log trimming
  - generated extern and user-hook bridge sync
  - executable-bit repair for runtime scripts
  - welcome-screen gating and display

Any smaller Rust insertion here would strand the same Nushell owners and create
exactly the kind of bridge growth this roadmap is trying to prevent.

The only acceptable future Rust move in this area would be a full owner cut
that deletes either `setup/initializers.nu` or `setup/environment.nu`
substantially end-to-end. There is no current evidence for such a cut.

## Public CLI Rule

A broader Rust public-CLI move is not the next deletion lane, even though the
first command-metadata owner slice has landed.

It becomes worth evaluating only after the bridge and materialization owners are
already much smaller. If revisited later, the required deletion budget is:

- delete or demote `core/yazelix.nu` as a public command-registry owner for at
  least one more command family
- delete `nushell_externs.nu` entirely or keep it as compatibility-only startup
  glue with no command discovery authority
- delete public Nushell wrapper parsing for at least one remaining command
  family beyond the already migrated `yzx_control` leaves

If a proposal keeps those surfaces and only adds a Rust dispatcher above them,
reject it.

`yazelix-qsb5.3` narrowed the first serious root-transition family already:

- start with the already migrated control-plane leaves:
  `yzx env`, `yzx run`, and `yzx update*`
- use `yazelix-qsb5.2` to make Rust the single public root/help/completion
  owner for that surface
- do not start by pulling `launch` / `enter` / `restart` or `status` /
  `doctor` into Rust, because those families still have larger surviving Nu
  owner clusters than the control-plane family does

The point of the first root cut is to delete the public registry role of
`core/yazelix.nu` for a surface that is already Rust-owned internally, not to
port more shell orchestration or doctor rendering into Rust by habit.

What `yazelix-qsb5.2` changed:

- `shells/posix/yzx_cli.sh` is now only the stable bootstrap wrapper
- the public root parser and dispatcher moved to `rust_core/yazelix_core/src/bin/yzx.rs`
- `env` / `run` / `update*` no longer depend on the old Nu root path at all
- remaining Nu-owned families are now routed directly from
  `rust_core/yazelix_core/src/bin/yzx.rs` to their concrete Nu modules

That deletes the old public root fallback `use ... core/yazelix.nu *; yzx ...`
without pretending the remaining Nushell command bodies are gone. They still
exist, but only as explicit internal helpers until later deletion lanes choose
them one family at a time.

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
- Bead: `yazelix-iwzn`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`

## Open Questions

- After the bridge layer collapse, does `config_parser.nu` still deserve to
  survive as a named owner, or should helper transport become a smaller shared
  utility inside the remaining Nu owners
- After the Yazi and Zellij full-owner cuts, is the next better deletion lane
  the runtime materialization orchestrator itself or the fragmented
  terminal/Helix materialization family
