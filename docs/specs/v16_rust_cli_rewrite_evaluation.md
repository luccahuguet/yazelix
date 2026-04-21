# v16 Rust CLI Rewrite Evaluation

## Summary

A broader Rust `yzx` rewrite is only justified in v16-or-later if it deletes
the remaining public Nushell owner seams after the helper and control-plane work
already landed.

That means a broader rewrite must do more than add a Rust dispatcher. It must
become the single public owner of command metadata, help, and parsing for the
families it moves, while deleting the Nu registry and extern seams that would
otherwise remain permanent duplication.

This is not the next deletion lane. The current deletion lanes are:

- collapsing the Nu bridge layer around `yzx_core` and `yzx_control`
- deleting one full generator and materialization family end-to-end
- continuing the newly landed Rust command-metadata lane only when the next cut
  deletes another real public parser or command-body owner

## Why

The current branch already proved the value of narrow Rust ownership:

- `yzx_core` owns typed config, state, preflight, runtime-env, materialization,
  structured report evaluation, Yazi/Zellij materialization generation, and
  shared public `yzx` command metadata
- `yzx_control` owns `yzx env`, `yzx run`, and `yzx update*`
- the launch and startup Rust follow-up was explicitly stopped for v15.x because
  the surviving path is mostly shell and process orchestration

That leaves one later question: should Rust eventually own more of the public
`yzx` surface.

That question is only worth answering with a concrete deletion budget. Otherwise
the result is just a Rust root above the same Nu ownership tree.

## Scope

- define when a broader Rust public-CLI move is worth doing
- define which command families are realistic candidates and which should stay
  Nushell-owned
- define the minimum deletion budget that justifies the rewrite cost
- define the go and no-go gates for a later prototype
- record the crate-vs-in-house posture for this possible path

## Rust Dependency Gate

This section is the dependency gate for `yazelix-2ex.1.11`.

Production crates that are acceptable only if Rust becomes the single public
owner for the moved command families:

- `clap`, but only if Rust replaces Nushell as the public parser, help owner,
  and completion source of truth for those families
- the existing focused helper crates already used in `rust_core/` for parsing,
  serialization, hashing, and error modeling

Dev-only crates that are acceptable:

- existing command and fixture test crates such as `assert_cmd`
- an optional snapshot-style help test crate only if plain assertions become too
  noisy

Rejected by default:

- async runtime frameworks
- process-control convenience frameworks
- shell-discovery convenience crates
- TUI or prompt frameworks

Packaging impact:

- a broader rewrite would move more of `yzx` from a Nushell-owned public command
  surface to a shipped Rust binary under `bin/`
- that cost is only justified if it deletes the remaining public Nu registry and
  extern ownership too

## Behavior

### Preconditions

A broader v16 Rust CLI path is worth evaluating only if all of these are true
first:

- the Nu bridge layer around `config_parser.nu`, `runtime_contract_checker.nu`,
  `materialization_orchestrator.nu`, and the per-command report bridges is
  already much thinner or partly gone
- at least one real generator and materialization owner family has been deleted
  or materially reduced, so a broader Rust root would not sit above the same
  old Nu materialization core
- the landed Rust command-metadata slice proves Rust can own help, palette
  inventory, and extern content without shelling back into the Nushell command
  tree for discovery

If those preconditions are not true, a broader public rewrite is still
premature.

### Problems The Rewrite Must Solve

A broader Rust CLI is only worth doing if it solves most of these problems at
once:

- extend the current single source of truth for public command metadata into
  parsing ownership for at least one more moved command family
- one primary public owner for any command family whose inner typed logic is
  already Rust-owned
- deletion of at least one more surviving internal Nu command-family owner
  still routed directly from `rust_core/yazelix_core/src/bin/yzx.rs`
- deletion of `nushell_externs.nu` entirely, or keeping it only as startup
  compatibility glue with no command discovery authority

It is not worth doing just to:

- reduce Nushell LOC in the abstract
- move shell quoting or detached process execution into Rust
- re-port text-heavy user-facing prose
- wrap the same Nu command tree in a faster front door

### Command-Family Recommendation

| Family | Current owner | v16 recommendation |
| --- | --- | --- |
| Root, help, version, completion, and palette inventory | Rust metadata owner plus thin Nu wrappers | Metadata/help/extern/menu ownership has started. Keep moving only if the next cut deletes a real public parser or command-body owner. |
| `yzx env`, `yzx run`, and `yzx update*` | `yzx_control` | Use these as the first root-transition family. They already have Rust-owned parsing and execution, so the next cut can delete more of the public Nu root without dragging shell-bound launch or report-rendering work into Rust. |
| `yzx launch`, `yzx enter`, `yzx restart` | Public Nu commands over Nu and POSIX orchestration | Possible later only if Rust would own request parsing and command-family metadata while Nu or POSIX still own the shell-heavy execution path |
| `yzx status`, `yzx doctor` | `yzx status` is already on the Rust public path; `yzx doctor` is still a public Nu command over Rust findings plus Nu rendering, fix, and live-session checks | `yzx status` is already done. `yzx doctor` is a no-go for the next public Rust cut after `yazelix-osco.2`: keep it Nushell-owned until the surviving `core/yzx_doctor.nu`, `utils/doctor.nu`, and `doctor_report_bridge.nu` owners either disappear or split into a real machine-only family that deletes the public Nu doctor surface. |
| `yzx config`, `yzx edit`, `yzx import` | Nushell | Keep Nushell-owned unless a later separate ownership argument appears |
| `yzx menu`, `yzx popup`, `yzx screen`, `yzx keys`, `yzx tutor`, `yzx whats_new`, info commands | Nushell | Keep Nushell-owned |
| `yzx desktop`, `yzx home_manager`, maintainer surfaces, package and distribution commands | Nushell, Nix, POSIX | Keep Nushell, Nix, and POSIX-owned unless a separate distribution-policy rewrite justifies moving them |
| Live workspace and session state | Rust pane orchestrator | Keep separate from the public CLI rewrite; do not fold plugin and session truth into `rust_core` by default |

The default v16 shape is therefore not "move every command." It is "move the
public command metadata and selected control-plane families only if that becomes
a cleaner single-owner model."

### 2026-04-20 Mixed-Family Choice

`yazelix-qsb5.3` chooses the already migrated control-plane family as the first
mixed public family for a Rust-owned `yzx` root:

- `yzx env`
- `yzx run`
- `yzx update*`

Why this family wins:

- `shells/posix/yzx_cli.sh` already dispatches `env`, `run`, and `update`
  directly to `yzx_control`
- `nushell/scripts/core/yazelix.nu` no longer exports `yzx update*`, so there
  is already real public Nu-owner deletion instead of a plan to add more
  wrappers
- `command_metadata.rs` already owns the family's help, extern, and menu
  catalog metadata
- execution is already typed Rust, not shell-heavy orchestration or
  renderer-heavy UX

That means `yazelix-qsb5.2` should start by making Rust the single public
root/help/completion owner for this migrated control-plane surface. The exact
remaining public-owner deletion is:

- delete the public command-registry role of `nushell/scripts/core/yazelix.nu`
  for the `env` / `run` / `update*` surface
- stop relying on the fallback `use ... core/yazelix.nu *; yzx ...` path for
  those commands
- keep remaining Nushell command families reachable only through explicit
  internal helper entrypoints until they have their own deletion story

`yazelix-qsb5.2` landed that first root cut with this owner shape:

- `shells/posix/yzx_cli.sh` is now bootstrap-only and no longer acts as a
  command-family router
- `rust_core/yazelix_core/src/bin/yzx.rs` is the public root parser and
  dispatcher
- `yzx env`, `yzx run`, and `yzx update*` stay on the Rust-only path through
  `yzx_control`
- surviving Nushell-owned families are now routed directly from
  `rust_core/yazelix_core/src/bin/yzx.rs` to their concrete Nu modules

That means these old public-owner roles are gone:

- the generic root fallback `use nushell/scripts/core/yazelix.nu *; yzx ...`
- the command-routing role previously carried by the shell case tree in
  `shells/posix/yzx_cli.sh`

Surviving internal Nu helper owners after the cut:

- `nushell/scripts/core/yazelix.nu` only for root help/version plus re-exported
  internal families
- `nushell/scripts/core/yzx_support.nu` for `yzx why` and `yzx sponsor`
- `nushell/scripts/core/yzx_workspace.nu` for `yzx cwd` and `yzx reveal`
- `nushell/scripts/core/yzx_session.nu` for `yzx restart`
- `nushell/scripts/core/yzx_doctor.nu` for `yzx doctor`
- `nushell/scripts/yzx/launch.nu`, `enter.nu`, `desktop.nu`, `menu.nu`,
  `popup.nu`, `config.nu`, `edit.nu`, `keys.nu`, `tutor.nu`, `screen.nu`,
  `whats_new.nu`, `home_manager.nu`, `import.nu`, and `dev.nu` as explicit
  internal helper modules, not as the public root registry

Explicit no-go for the first mixed family:

- `yzx launch`, `yzx enter`, and `yzx restart` still depend on
  `yzx/launch.nu`, `yzx/enter.nu`, `core/yzx_session.nu`,
  `core/launch_yazelix.nu`, `core/start_yazelix.nu`,
  `core/start_yazelix_inner.nu`,
  `utils/runtime_env.nu`, `utils/startup_profile.nu`,
  `utils/terminal_launcher.nu`, and `shells/posix/*.sh`
- `yzx status` is already on the public Rust control-plane path through
  `yzx_control`, but `yzx doctor` still carries meaningful Nushell bridge,
  rendering, and repair ownership in `core/yzx_doctor.nu`, `utils/doctor.nu`,
  and `doctor_report_bridge.nu`

### 2026-04-21 Doctor Re-evaluation

`yazelix-osco.2` is a no-go for a public Rust `yzx doctor` cut in the current
shape.

What changed first:

- `yazelix-osco.1` deleted the old per-report shims
- one shared bridge now lives in `utils/doctor_report_bridge.nu`

Why that is still not enough:

- `core/yzx_doctor.nu` still owns the public command surface, including the
  `--json` and `--fix` split and the fix-only env gate
- `utils/doctor.nu` still owns the human renderer, summary logic, Zellij
  pane-orchestrator live health checks, and all current `--fix` behavior
- `utils/doctor_report_bridge.nu` still shapes several multi-source requests
  before the Rust helpers run, especially install ownership, runtime preflight,
  Helix doctor context, and config doctor inputs

Current surviving Nushell owners after the bridge collapse:

- `nushell/scripts/core/yzx_doctor.nu` at about `25` raw lines
- `nushell/scripts/utils/doctor.nu` at about `339` raw lines
- `nushell/scripts/utils/doctor_report_bridge.nu` at about `293` raw lines

Decision:

- do not start a public Rust or Clap-owned `yzx doctor` lane now
- a Rust front door here would still sit above a large surviving Nushell doctor
  tree instead of deleting it
- reopen only if Yazelix can either:
  - split out a real machine-only doctor family that deletes the public Nu
    doctor surface end-to-end
  - or delete most of `utils/doctor.nu` and `doctor_report_bridge.nu` so Rust
    can become the single public owner without re-porting prose and fix flows
    by habit

### Required Deletion Budget

A broader rewrite is not approved unless it deletes all of these public-owner
seams:

- the surviving internal Nu owner for at least one more command family still
  routed directly from `rust_core/yazelix_core/src/bin/yzx.rs`
- `nushell_externs.nu` entirely, or its remaining non-authoritative startup
  wrapper role if a later shell integration no longer needs it
- public Nushell wrapper parsing for at least one remaining family beyond the
  already migrated `yzx_control` leaves

It should also delete or materially shrink at least one of these still-real
owner clusters:

- the public `launch` / `enter` / `restart` family
- the public `status` / `doctor` family
- the surviving bridge and report owner cluster around `config_parser.nu`,
  `doctor_report_bridge.nu`, and `doctor.nu`

If a proposal keeps those surfaces and merely adds a Rust root dispatcher above
them, reject it.

### Allowed Transition Shape

If Rust becomes the public `yzx` owner later, surviving Nushell implementations
may still exist for intentionally kept surfaces, but only as internal helpers.

Allowed:

- a Rust `yzx` binary dispatching to dedicated internal Nushell entrypoints for
  families that are intentionally still Nushell-owned
- Rust-owned help and completion metadata for the public command surface that it
  owns

Not allowed:

- Rust shelling back into `use nushell/scripts/core/yazelix.nu *; yzx ...`
- keeping the existing Nushell command tree as a parallel public parser or help
  owner
- using `clap` only as a thin argv shim over the unchanged public Nushell tree

### Go / No-Go Gates

Go only if all of these are true:

1. The already migrated `yzx_control` leaves fit cleanly into the new public
   Rust root instead of becoming a second parallel control-plane surface
2. At least one remaining public command family has a narrow enough internal
   boundary that Rust can own the public contract without inheriting fuzzy
   shell-bound behavior
3. `nushell_externs.nu` stays compatibility-only or can be deleted instead of
   returning to steady-state command discovery
4. A parity harness exists for help output, exit behavior, and the migrated
   command families
5. The proposal names the exact Nu modules that stop being public owners

No-go signals:

- most moved commands still immediately hand off to the old public Nu tree
- the main reason to move a family is shell or detached-process execution
- the proposal cannot name a real family-level deletion
- the case for the rewrite is mostly "Rust is faster" without an owner
  simplification story

## Non-goals

- forcing all user-facing commands into Rust in one pass
- moving renderer and template ownership out of Nushell by default
- moving detached terminal execution or shell quoting into Rust just because the
  public CLI moved
- folding the Rust pane orchestrator into `rust_core`
- porting maintainer tooling or Nix and package ownership into Rust as part of
  the same decision

## Acceptance Cases

1. A maintainer can tell when a broader Rust public-CLI rewrite is actually
   justified instead of guessing from Nushell size
2. The proposal names which command families are valid candidates and which
   should remain Nushell-owned
3. The minimum deletion budget is explicit, including the public registry and
   extern seams
4. The document rejects Rust-root-above-Nu-tree proposals clearly
5. The current bridge-collapse and materialization-delete lanes remain the
   default until those gates are met

## Verification

- `nu nushell/scripts/dev/validate_specs.nu`
- manual review against:
  - `docs/specs/rust_migration_matrix.md`
  - `docs/specs/rust_nushell_bridge_contract.md`
  - `docs/specs/cross_language_runtime_ownership.md`
  - `docs/specs/v15_trimmed_runtime_contract.md`

## Traceability

- Bead: `yazelix-2ex.1.11`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
