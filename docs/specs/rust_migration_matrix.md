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
- the remaining shell/process-heavy surfaces and deterministic test buckets that
  still need explicit deletion budgets after the larger generator/materialization
  cuts landed

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
- `yzx-command-metadata.sync-externs`
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
| Bridge transport and error shaping | `yzx_core_bridge.nu` plus the small per-command report bridges | High-leverage deletion lane with relatively low semantic risk. The generic helper transport no longer lives inside `config_parser.nu`. | Keep one minimal transport layer, not one policy-bearing Nu owner per Rust helper command. | Landed under `yazelix-057w` |
| Config, state, env, and preflight shims | dev-only `config_normalize_test_helpers.nu`, `runtime_env.nu`, plus direct preflight calls in `core/start_yazelix.nu`, `core/launch_yazelix.nu`, and `yzx/launch.nu` | Rust already owns the typed computation. `runtime_contract_checker.nu`, `config_state.nu`, and `config_parser.nu` are gone, `runtime_env.nu` is now the explicit shell-exec seam, and the startup/launch owners now call the Rust preflight helpers directly. | Keep shrinking request shaping and machine classification where Rust can become the single typed owner. Leave only execution and final user rendering in Nu, and keep the normalize helper path dev-only. | Landed under `yazelix-0ksx`, `yazelix-ekfc`, and `yazelix-sq0g.4` |
| Doctor and install report bridges | public report owner in `yzx_control`; private Nu repair helper in `doctor_fix.nu`; caller-local desktop/restart UX | The public doctor owner, shared report bridge, and install-ownership Nu bridge are gone. Rust now owns report aggregation, summary/rendering, JSON, live Zellij plugin-health checks, install-ownership evaluation, and env-derived install-ownership request construction. | Keep the private fix helper narrow. Do not recreate a shared doctor-report bridge, install-ownership bridge, or public Nu doctor owner. | Public doctor cut landed under `yazelix-5ewl.3`; install-ownership bridge collapse landed under `yazelix-87zd` |
| Product-side full-config owner seam | former product callers now using `integration-facts.compute`, `transient-pane-facts.compute`, `startup-facts.compute`, and `runtime-env.compute`; surviving test-only normalize probe under `nushell/scripts/dev` | The real product-side `parse_yazelix_config` reads are gone, and `config_parser.nu` is deleted. The only surviving normalize shim is dev-only test support around helper resolution. | Keep the narrower Rust-owned fact helpers as the only retained product-side config facts. Do not recreate a second generic full-config bridge. | Budget defined under `yazelix-sq0g.1`; product-side cuts landed under `yazelix-jkk3`, `yazelix-sq0g.2`, `yazelix-sq0g.3`, and `yazelix-sq0g.4` |
| Runtime materialization lifecycle | Rust owners: `runtime-materialization.plan`, `runtime-materialization.materialize`, `runtime-materialization.repair`, including `--from-env` request construction; caller-local Nu progress/error rendering in startup and doctor | Landed full-owner cut; the shared Nu bridge is gone and Rust now owns env-derived request construction | Keep Rust as the lifecycle and request-construction owner. Do not recreate a shared Nu materialization bridge. | Full-owner cut landed under `yazelix-ulb2.9`; shared bridge deletion landed under `yazelix-q0o9.2` |
| Yazi materialization family | Rust owner: `yazi-materialization.generate`; remaining Nu use is dev-only direct invocation | The real Nu owner family is gone, and the surviving setup wrapper is deleted. | Keep Rust as the single Yazi materialization owner. Do not recreate a public or product-side compatibility wrapper. Dependency gate for the landed cut: in-house logic plus existing `serde` and `toml`; no new crates. | Landed under `yazelix-ulb2.3.1`; wrapper deletion landed under `yazelix-vf0u.1` |
| Zellij materialization family | Rust owner: `zellij-materialization.generate`; remaining Nu use is direct product or dev invocation | The real Nu owner family is gone, the setup wrapper is deleted, and Rust still owns base-config selection, semantic KDL extraction, layout rendering, plugin wasm sync, permission migration, popup-runner cleanup, and generation-state reuse. | Keep Rust as the single Zellij materialization owner. Do not recreate a public or product-side compatibility wrapper. Dependency gate for the landed cut: in-house logic plus existing `serde`, `serde_json`, `toml`, `sha2`, `thiserror`, and `lexopt`; no new crates. | Landed under `yazelix-ulb2.3.2`; wrapper deletion landed under `yazelix-vf0u.2` |
| Terminal launch-time compatibility seam | Standalone wrapper deleted; surviving Nu helpers live in `core/launch_yazelix.nu` | Rust already owns generated terminal writes in `terminal_materialization.rs` plus Ghostty config/shader generation in `ghostty_materialization.rs`. The surviving Nu seam is launch-time supported-terminal filtering, user-facing summary text, and the Ghostty reroll bridge. | Treat this as a landed delete-wrapper lane, not a fresh full-owner port. Keep only irreducible launch-time compatibility logic in `launch_yazelix.nu` and do not recreate a separate terminal materialization owner. Dependency gate: no new crates; keep using the existing Rust owners and in-house helper logic. | Landed under `yazelix-ulb2.10.3` |
| Helix shared path and launch compatibility seam | Standalone wrapper deleted; shared Helix path truth now lives in `utils/common.nu`; adjacent consumers are `yzx/edit.nu`, `yzx/import.nu`, and `shells/posix/yazelix_hx.sh` | Rust already owns Helix template merge policy, reveal-binding enforcement, generated-file writes, import-notice state, and now the doctor-side expected contract build in `helix_materialization.rs`. The surviving Nu/POSIX seam is path truth and launch/editor/import orchestration, not a second generator owner. | Treat this as a landed wrapper-deletion lane. Keep Helix path truth shared and non-owning, and keep the POSIX launcher talking to the Rust helper directly. Dependency gate: no new crates; keep using existing `serde` and `toml` plus in-house helper extraction where needed. | Landed under `yazelix-ulb2.10.2` |
| Shell initializer generation and shellhook environment setup | `setup/initializers.nu`, `setup/environment.nu` | The deterministic runtime-env subcore already moved to Rust. What remains is external-tool init generation, shell-specific text normalization, bridge sync, startup profiling, log cleanup, executable-bit repair, and welcome-shellhook orchestration. | Keep Nushell-owned in v15.x. Reopen only if a future port can delete the surviving shellhook owner end-to-end instead of inserting one more text or bridge helper. | Decision locked by `yazelix-iwzn` |
| Launch and startup process orchestration | `core/launch_yazelix.nu`, `core/start_yazelix.nu`, `core/start_yazelix_inner.nu`, `utils/terminal_launcher.nu`, `shells/posix/*.sh` | Shell-bound and process-heavy, not the best next Rust target | Keep Nu and POSIX in v15.x. Reopen only if a new deterministic subcore appears that deletes a real owner. | No active deletion lane; historical stop note in `launch_bootstrap_rust_migration.md` |
| Public `yzx` root, help, completion, and palette inventory | Rust metadata owner: `command_metadata.rs`; generated extern lifecycle owner: `yzx_core yzx-command-metadata.sync-externs`; surviving Nu command bodies: `core/yzx_*.nu`, `yzx/*.nu` | Metadata/help/menu ownership has landed, `nushell_externs.nu` is deleted, and `yazelix-f7hz` deleted `core/yazelix.nu`. Generated extern sync is now a Rust-owned cache lifecycle, with `setup/environment.nu` only keeping the shellhook profile boundary. | Keep shrinking only when the next cut deletes a real public parser or command-body owner. Do not rebuild a parallel Nu registry or generated-extern wrapper. | Follow-up lane: `yazelix-2jkb.3` remains the next public-family decision |
| Workspace and session state | `rust_plugins/zellij_pane_orchestrator/`, `integrations/*.nu`, `zellij_wrappers/*.nu` | Already Rust where live session truth matters | Keep this separate from `rust_core`. Do not fold the pane-orchestrator track into the control-plane migration by habit. | Separate pane-orchestrator beads |

## 2026-04-21 Config, State, Env, And Preflight Deletion-Budget Decision

`yazelix-0ksx.1` and `yazelix-0ksx.2` re-audited the remaining
config/state/env/preflight bridge cluster after the generic `yzx_core` bridge
collapse.

Decision:

- delete `nushell/scripts/utils/runtime_contract_checker.nu`
- move startup preflight ownership into `core/start_yazelix.nu`
- move launch preflight and terminal-candidate ownership into
  `core/launch_yazelix.nu`
- move the one-off launch-script runtime-script check into `yzx/launch.nu`
- delete `config_state.nu` by moving the active-surface and materialized-state
  request construction into Rust
- keep `runtime_env.nu` only as the shell-bound env application and argv exec
  seam

Why this cut:

- `runtime_contract_checker.nu` no longer owned the doctor path after the
  doctor-report bridge collapse
- it mostly survived as a generic startup/launch veneer over
  `runtime-contract.evaluate` and `startup-launch-preflight.evaluate`
- deleting that file removed a whole per-helper Nu owner without forcing a fake
  port of the shell-bound `runtime_env.nu` seam

Defer to other lanes:

- doctor/install report transport belongs to `yazelix-osco`
- materialization-orchestrator cleanup belongs to `yazelix-q0o9`
- broader config/state/env bridge collapse beyond this file belongs to later
  follow-up after the current direct-owner cut

## 2026-04-21 Runtime Materialization Bridge Deletion-Budget Decision

`yazelix-q0o9.1` re-audited `core/materialization_orchestrator.nu` after Rust
became the lifecycle owner for runtime materialization.

Decision:

- `core/materialization_orchestrator.nu` can die, and `yazelix-q0o9.2` deletes
  it by moving environment-derived materialization request construction into
  Rust entrypoints
- do not delete it by copying `build_runtime_materialization_context` into
  `start_yazelix_inner.nu`, `doctor_fix.nu`, or another install-ownership bridge
- keep the existing startup profile boundary name
  `materialization_orchestrator/materialize_runtime_state` comparable even
  after the file disappears

Why it can die:

- Rust already owns plan, materialize, repair, repair directives, artifact
  checks, and generated-state writes
- Rust already has env-based request construction in `control_plane.rs`; the
  remaining Nu bridge exists mostly because the current `yzx_core`
  materialization commands still require a full request JSON payload
- the surviving Nu responsibilities are caller-local progress and error
  rendering, not shared lifecycle policy

Surviving Nu responsibilities after `q0o9.2`:

- startup keeps the profile wrapper and user-facing failure message before
  Zellij handoff
- doctor keeps `--fix` progress printing and repair error reporting
- doctor report keeps only the structured runtime-findings call and reads the
  Rust-computed layout path from the Rust materialization plan
- maintainer update canaries keep a stable command path to force repair, but
  should not import a product-side shared Nu bridge

Verification for `q0o9.2`:

- generated config canonical tests still materialize runtime state
- `yzx doctor --fix` still repairs missing managed generated layouts
- maintainer startup-profile regression still sees the materialization profile
  boundary
- update canaries still exercise forced generated-state repair
- syntax, spec validation, and whitespace checks pass
## 2026-04-22 Full-Config Product Owner Decision

`yazelix-sq0g.1` re-audited the remaining product-side
`parse_yazelix_config` seam after the earlier bridge-collapse work.

Decision:

- stop treating `config_parser.nu` as a product-side owner when callers only
  need popup geometry, popup argv, managed editor kind, sidebar enablement,
  Yazi command facts, or startup/session toggles
- route popup/menu callers through `transient-pane-facts.compute`
- route popup/editor wrappers through the narrower `runtime-env.compute` and
  `transient-pane-facts.compute` surfaces
- route startup/launch/setup callers through `runtime-env.compute` and
  `startup-facts.compute`
- leave shell/process execution, Zellij/editor/Yazi orchestration, and startup
  handoff in Nu and POSIX
- treat dev/test parser callers as out of scope for the product-side cut

Why this cut:

- product callers were reparsing the full managed config even when they only
  needed a small retained fact surface
- Rust already owned the canonical normalized config and the deterministic fact
  extraction was a better fit than duplicating smaller Nu helper layers
- the surviving Nu responsibilities are still UX, shell, process, and tool
  orchestration, not config ownership

Landed follow-up:

- `yazelix-sq0g.4` deletes `config_parser.nu` outright and leaves only the
  dev-only `config_normalize_test_helpers.nu` probe for helper-resolution and
  fail-fast test cases

| Front-door UX and command-palette surfaces | `setup/welcome.nu`, `utils/front_door_runtime.nu`, `yzx/menu.nu`, `yzx/edit.nu`, `yzx/import.nu` | Mostly shell presentation, popup, editor, or interactive UX after the renderer and info commands moved | Rust already owns `yzx screen`, `yzx tutor`, `yzx whats_new`, the renderer, and upgrade-summary shaping. Keep the surviving Nu floor only while it remains the honest shell/process boundary. | Narrow follow-on target, not a broad migration bucket |
| Distribution and host integration | `home_manager/`, `packaging/`, `shells/`, `yzx/desktop.nu` | Nix, POSIX, and UX-heavy by nature | Keep outside the current Rust migration by default, but allow targeted Rust ownership when a public family is already mostly backed by Rust-owned reports or control helpers. `yzx home_manager` has now crossed that threshold and is no longer a Nushell owner. | Partially migrated: `yzx home_manager` is Rust-owned; broader distribution and desktop paths stay outside the current Rust target |

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
  - surviving direct-Rust call sites in `nushell/scripts/yzx/edit.nu`,
    `nushell/scripts/yzx/import.nu`, and
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

- startup profile wrapping
- final Nu-facing progress and remediation rendering in the startup and doctor
  callers

What was deleted:

- `nushell/scripts/utils/generated_runtime_state.nu`
- `nushell/scripts/core/materialization_orchestrator.nu`
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

- delete or demote at least one more surviving concrete Nu command family under
  the Rust root, now that `core/yazelix.nu` is already gone
- keep `nushell_externs.nu` deleted by preserving the Rust-owned generated
  extern bridge lifecycle
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

The point of the first root cut was to delete the public registry role of
`core/yazelix.nu` for a surface that was already Rust-owned internally, not to
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

- `yzx_repo_validator validate-specs`
- manual review against `docs/specs/rust_nushell_bridge_contract.md`
- manual review against `docs/specs/cross_language_runtime_ownership.md`
- manual review against `docs/specs/v15_trimmed_runtime_contract.md`
- manual review against `docs/subsystem_code_inventory.md`

## Traceability

- Bead: `yazelix-kt5.1`
- Bead: `yazelix-ulb2.3`
- Bead: `yazelix-iwzn`
- Defended by: `yzx_repo_validator validate-specs`

## Open Questions

- After the bridge layer collapse, does `config_parser.nu` still deserve to
  survive as a named owner, or should helper transport become a smaller shared
  utility inside the remaining Nu owners
- After the Yazi and Zellij full-owner cuts, is the next better deletion lane
  the runtime materialization orchestrator itself or the fragmented
  terminal/Helix materialization family
