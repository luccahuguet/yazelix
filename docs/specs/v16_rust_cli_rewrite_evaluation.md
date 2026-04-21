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

- the Nu bridge layer around `config_parser.nu`, `config_state.nu`,
  `runtime_env.nu`, and the per-command report bridges is
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
- deletion of another real public owner now that `nushell_externs.nu` has been
  deleted and `yzx_core yzx-command-metadata.sync-externs` owns generated
  extern bridge lifecycle

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
| `yzx status`, `yzx doctor` | `yzx status` and the public report half of `yzx doctor` are now on the Rust public path; the side-effecting `yzx doctor --fix` flow still ends in a private Nu helper | `yzx status` is already done, and `yazelix-5ewl.3` landed the honest report-only doctor owner cut by deleting the public Nu doctor surface and the shared report bridge. `yazelix-5ewl.4` then closed the Clap follow-up as a no-go because the surviving private fix helper plus the still-large internal-family rule engine are not a meaningful parser-deletion budget yet. |
| `yzx config` | `yzx_control` | Good Rust-owned public-family cut once the config surface loader and reset semantics are both owned directly in Rust and the public Nu wrapper disappears |
| `yzx keys` | `yzx_control` | Good Rust-owned help/discoverability cut only if the alias family and the table-style human output survive without turning into a flat wrapper or a broad formatting-crate dependency |
| `yzx edit`, `yzx import` | Nushell | Keep Nushell-owned unless a later separate ownership argument appears |
| `yzx why`, `yzx sponsor` | `yzx_control` | Good tiny Rust-owned leaf cuts when the goal is deleting the last trivial support-family Nu owner without changing user-visible copy or sponsor fallback behavior |
| `yzx menu`, `yzx popup`, `yzx screen`, `yzx tutor`, `yzx whats_new` | Nushell | Keep Nushell-owned |
| `yzx desktop`, maintainer surfaces, package and distribution commands | Nushell, Nix, POSIX | Keep Nushell, Nix, and POSIX-owned unless a separate distribution-policy rewrite justifies moving them. `yzx home_manager` left this bucket once the install-ownership report became the real owner and the public family moved to Rust |
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
- `nushell/scripts/core/yzx_session.nu` for `yzx restart`
- `nushell/scripts/yzx/launch.nu`, `enter.nu`, `desktop.nu`, `menu.nu`,
  `popup.nu`, `edit.nu`, `tutor.nu`, `screen.nu`,
  `whats_new.nu`, `import.nu`, and `dev.nu` as explicit
  internal helper modules, not as the public root registry

Explicit no-go for the first mixed family:

- `yzx launch`, `yzx enter`, and `yzx restart` still depend on
  `yzx/launch.nu`, `yzx/enter.nu`, `core/yzx_session.nu`,
  `core/launch_yazelix.nu`, `core/start_yazelix.nu`,
  `core/start_yazelix_inner.nu`,
  `utils/runtime_env.nu`, `utils/startup_profile.nu`,
  `utils/terminal_launcher.nu`, and `shells/posix/*.sh`
- `yzx status` and the public report path of `yzx doctor` are now on the
  Rust control-plane path through `yzx_control`, but `yzx doctor --fix` still
  survives as a private Nushell helper in `utils/doctor_fix.nu`

### 2026-04-21 Doctor Report-Owner Cut

`yazelix-5ewl.3.1` and `yazelix-5ewl.3.2` landed the honest report-only Rust
owner cut for `yzx doctor`.

What changed:

- `yzx_control` now owns the public `yzx doctor` route, `--json`, summary
  counts, human rendering, and live Zellij pane-orchestrator report checks
- `core/yzx_doctor.nu` is deleted
- `utils/doctor_report_bridge.nu` is deleted
- `utils/doctor.nu` is deleted
- the surviving Nushell work is a private fix helper in
  `utils/doctor_fix.nu`
- install-ownership request shaping lives in a focused
  `utils/install_ownership.nu` helper instead of a mixed doctor bridge

Why this counts:

- Rust is now the single public owner for the read-only doctor/report surface
- the deleted Nu files were the real public and bridge owners, not tiny stubs
- the surviving Nu fix helper is private and explicitly side-effecting instead
  of doubling as the report owner

Surviving private Nushell owners after the cut:

- `nushell/scripts/utils/doctor_fix.nu` for the side-effecting repair flow
- `nushell/scripts/utils/install_ownership.nu` for focused install-ownership
  request shaping outside the doctor report owner

Decision:

- the public report-owner cut is now landed
- do not treat this as automatic Clap approval yet
- `yazelix-5ewl.4` is the follow-up bead that decides whether the remaining
  private fix helper plus the broader internal-family routing surface are now
  small enough for a meaningful Clap decision

### 2026-04-21 Post-`yazelix-5ewl.4` Clap Re-evaluation

`yazelix-5ewl.4` closes the current Clap-prep lane as a no-go for now.

Why:

- the public Rust root is already small and schema-driven
- the surviving internal-Nu families are `desktop`, `dev`, `edit`, `enter`,
  `import`, `launch`, `menu`, `popup`, `restart`, `screen`, `tutor`, and
  `whats_new`
- most of those families are still intentionally Nushell-owned because they are
  shell-heavy, process-heavy, or mostly product UX
- adding `clap` now would mostly replace a small amount of family
  classification logic while keeping or re-encoding the same internal-family
  table and family-specific fallback behavior

Decision:

- do not open a Clap implementation lane now
- reopen only if a future delete-first cut removes another surviving internal
  family owner and materially shrinks `INTERNAL_NU_FAMILIES`

### 2026-04-21 Workspace Owner Cut

`yazelix-ql71` landed the next honest public-family deletion cut after the
Clap no-go.

What changed:

- `yzx_control` now owns the public `yzx cwd` and `yzx reveal` routes
- `core/yzx_workspace.nu` is deleted
- `cwd` and `reveal` are removed from `INTERNAL_NU_FAMILIES`
- the public workspace commands now talk directly to the pane orchestrator and
  the configured `ya` CLI instead of routing through a shared Nushell workspace
  owner

Why this counts:

- it deletes two surviving public root families at once
- it keeps pane-orchestrator session truth as the live workspace/sidebar source
- it does not reintroduce Nushell-side session-state reconstruction through a
  Rust shim over `integrations/zellij.nu` or `integrations/yazi.nu`

### Required Deletion Budget

A broader rewrite is not approved unless it deletes all of these public-owner
seams:

- the surviving internal Nu owner for at least one more command family still
  routed directly from `rust_core/yazelix_core/src/bin/yzx.rs`
- no regression to a Nushell command-discovery or generated-extern lifecycle
  wrapper after the `nushell_externs.nu` deletion
- public Nushell wrapper parsing for at least one remaining family beyond the
  already migrated `yzx_control` leaves

It should also delete or materially shrink at least one of these still-real
owner clusters:

- the public `launch` / `enter` / `restart` family
- the public `status` / `doctor` family
- the surviving bridge and report owner cluster around `config_parser.nu`,
  `install_ownership.nu`, and `doctor_fix.nu`

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
3. The deleted `nushell_externs.nu` path stays deleted, with generated extern
   bridge sync remaining Rust-owned instead of returning to steady-state
   command discovery
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
