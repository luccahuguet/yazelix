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

## Why

The current branch already proved the value of narrow Rust ownership:

- `yzx_core` owns typed config, state, preflight, runtime-env, materialization,
  and structured report evaluation
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
  `generated_runtime_state.nu`, and the per-command report bridges is already
  much thinner or partly gone
- at least one real generator and materialization owner family has been deleted
  or materially reduced, so a broader Rust root would not sit above the same
  old Nu materialization core
- Rust can own command metadata, help, and completion without shelling back into
  the Nushell command tree for discovery

If those preconditions are not true, a broader public rewrite is still
premature.

### Problems The Rewrite Must Solve

A broader Rust CLI is only worth doing if it solves most of these problems at
once:

- one source of truth for public command metadata, parsing, help, and shell
  completion
- one primary public owner for any command family whose inner typed logic is
  already Rust-owned
- deletion of the public command-registry role currently carried by
  `core/yazelix.nu`
- deletion of `nushell_externs.nu` as an authoritative command-metadata owner

It is not worth doing just to:

- reduce Nushell LOC in the abstract
- move shell quoting or detached process execution into Rust
- re-port text-heavy user-facing prose
- wrap the same Nu command tree in a faster front door

### Command-Family Recommendation

| Family | Current owner | v16 recommendation |
| --- | --- | --- |
| Root, help, version, and completion surface | `core/yazelix.nu`, `utils/nushell_externs.nu` | Move only if Rust becomes the single public owner of command metadata for the moved surface |
| `yzx env`, `yzx run`, and `yzx update*` | `yzx_control` | Already migrated. Treat these as precedent, not as future scope justification by themselves |
| `yzx launch`, `yzx enter`, `yzx restart` | Public Nu commands over Nu and POSIX orchestration | Possible later only if Rust would own request parsing and command-family metadata while Nu or POSIX still own the shell-heavy execution path |
| `yzx status`, `yzx doctor` | Public Nu commands over Rust findings plus Nu rendering | Possible later only if the remaining public wrappers disappear and the family gains one clear owner instead of one more layer |
| `yzx config`, `yzx edit`, `yzx import` | Nushell | Keep Nushell-owned unless a later separate ownership argument appears |
| `yzx menu`, `yzx popup`, `yzx screen`, `yzx keys`, `yzx tutor`, `yzx whats_new`, info commands | Nushell | Keep Nushell-owned |
| `yzx desktop`, `yzx home_manager`, maintainer surfaces, package and distribution commands | Nushell, Nix, POSIX | Keep Nushell, Nix, and POSIX-owned unless a separate distribution-policy rewrite justifies moving them |
| Live workspace and session state | Rust pane orchestrator | Keep separate from the public CLI rewrite; do not fold plugin and session truth into `rust_core` by default |

The default v16 shape is therefore not "move every command." It is "move the
public command metadata and selected control-plane families only if that becomes
a cleaner single-owner model."

### Required Deletion Budget

A broader rewrite is not approved unless it deletes all of these public-owner
seams:

- the public command-registry role of `core/yazelix.nu`
- `nushell_externs.nu` as an authoritative public command-metadata owner
- public Nushell wrapper parsing for at least one remaining family beyond the
  already migrated `yzx_control` leaves

It should also delete or materially shrink at least one of these still-real
owner clusters:

- the public `launch` / `enter` / `restart` family
- the public `status` / `doctor` family
- the surviving bridge and report owner cluster around `config_parser.nu` and
  the report shims

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
3. `nushell_externs.nu` can be deleted or reduced to compatibility-only glue
   instead of remaining part of steady-state command discovery
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
