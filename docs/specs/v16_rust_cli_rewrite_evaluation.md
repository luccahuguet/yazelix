# v16 Rust CLI Rewrite Evaluation

## Summary

A broader Rust `yzx` rewrite is only justified in v16-or-later if Rust becomes the single public owner of the command tree for the control-plane families that still matter, deletes real Nushell public-owner seams, and removes the generated extern bridge as a source of truth for command discovery.

This is not a license to rewrite Yazelix just because Nushell remains large. The current v15 helper-backed Rust path already moved the high-value deterministic core behind `yzx_core`. A broader rewrite is worth doing only if it deletes the public command-registry and bridge layers that would otherwise remain permanent duplication.

## Why

The v15 Rust slices proved that narrow typed helpers buy real value:

- config normalization, config-state hashing, materialization planning, runtime dependency checks, and canonical runtime-env planning now live behind the Rust/Nushell bridge
- the launch/startup audit concluded that the remaining v15 launch path is mostly shell-bound orchestration and should stay out of the helper rewrite
- renderer/template ownership is explicitly locked to Nushell for v15.x

That leaves one larger future question: should Rust eventually own the public `yzx` CLI itself.

That question should only be answered with a concrete deletion budget and explicit ownership boundaries. Otherwise a broad rewrite would just add clap on top of the same Nushell tree and leave the same product split alive underneath.

## Scope

- define when a broader Rust public-CLI rewrite is worth doing
- define which command families are reasonable v16 Rust candidates and which should stay Nushell-owned
- define the minimum deletion budget that justifies the rewrite cost
- define the go/no-go gates for a v16 prototype
- record the crate-vs-in-house decision for this possible path

## Rust Dependency Gate

This section is the dependency gate for `yazelix-2ex.1.11`.

Production crates that are acceptable only if Rust becomes the public `yzx` owner:

- `clap`, but only if Rust replaces Nushell as the command parser, help owner, and completion source of truth
- the existing focused helper crates already used in `rust_core/` for serialization, parsing, hashing, and error modeling

Dev-only crates that are acceptable:

- existing command/fixture test crates such as `assert_cmd`
- an optional snapshot-style crate for help/completion parity only if plain assertions become too noisy to maintain

Logic to keep in-house:

- command-family ownership and dispatch policy
- update-owner and distribution policy
- launch/request planning records
- status/doctor summary models that are specific to Yazelix contracts

Rejected by default:

- `tokio`, async runtime frameworks, or process-control convenience frameworks
- shell-discovery convenience crates
- TUI/prompt frameworks such as `dialoguer`
- broad terminal abstraction crates

Packaging impact:

- a broader rewrite would move `yzx` from a Nushell-owned public command to a shipped Rust binary under `bin/`
- that cost is justified only if it deletes the public Nushell command registry, public Nushell wrapper parsing for migrated families, and the generated extern bridge timing/ownership seam

## Behavior

### Preconditions

A broader v16 Rust CLI path is worth evaluating only because the narrower v15.x helper work already landed first:

- `config.normalize`, `config_state.compute`, `runtime_materialization.plan`, `runtime_materialization.apply`, `runtime-contract.evaluate`, and `runtime-env.compute` already deleted real deterministic Nushell owners
- launch/startup Rust follow-up work already has a recorded stop condition in `launch_bootstrap_rust_migration.md`
- renderer/template ownership already stays Nushell-owned under `rust_migration_matrix.md`

If those preconditions were not true, a broader public rewrite would still be premature.

### Problems The Rewrite Must Solve

A broader Rust CLI is only worth doing if it solves most of these problems at once:

- one static source of truth for public command parsing, help, and shell completions
- one primary owner for backend-required and mixed control-plane command families
- less public-owner duplication between `core/yazelix.nu`, `yzx/*.nu`, and generated extern surfaces
- tighter shell-independent usage errors and machine-readable exit behavior for noninteractive command paths

It is not worth doing just to:

- reduce Nushell LOC
- move shell quoting or detached process execution into Rust
- re-port text-heavy user-facing command prose
- chase tiny startup microbenchmarks without deleting command-owner seams

### Command-Family Recommendation

The command-family split for a future v16 public rewrite should follow the current public surface audit in `yzx_command_surface_backend_coupling.md`.

| Family | Current owner | v16 recommendation |
| --- | --- | --- |
| Root/help/version/completion surface | `core/yazelix.nu`, generated extern bridge | Move to Rust only if Rust becomes the single public `yzx` owner |
| Backend control plane: `yzx env`, `yzx run` | Nushell wrappers over the runtime/core bridge | First public-family candidate for Rust ownership |
| Mixed startup/control plane: `yzx launch`, `yzx enter`, `yzx restart` | Nushell wrappers plus startup/launch helpers | Move only after the family stays scoped to request/dispatch ownership and does not drag detached terminal execution into Rust |
| Mixed inspection: `yzx status`, `yzx doctor` | `core/yazelix.nu`, `doctor.nu`, runtime-contract helpers | Move only after machine-readable status/findings and prose rendering are split cleanly |
| Config/edit/import UX | `yzx/config.nu`, `yzx/edit.nu`, `yzx/import.nu` | Keep Nushell-owned in v16 unless a separate later reason appears |
| Popup/menu/screen/training/info UX | `yzx/popup.nu`, `yzx/menu.nu`, `yzx/screen.nu`, `yzx/keys.nu`, `yzx/tutor.nu`, `yzx/whats_new.nu`, `core/yazelix.nu` info commands | Keep Nushell-owned |
| Distribution/desktop/Home Manager/maintainer surfaces | `yzx/desktop.nu`, `yzx/home_manager.nu`, `yzx/dev.nu`, update commands in `core/yazelix.nu` | Keep Nushell/Nix-owned unless a separate distribution-policy rewrite justifies moving them |
| Live workspace/session state | Rust pane orchestrator | Keep separate from the public CLI rewrite; do not fold plugin/session truth into `rust_core` |

The default v16 shape is therefore not "move every command." It is "move the public command root and the control-plane families only if that becomes a cleaner single-owner model."

### Required Deletion Budget

A broader rewrite is not approved unless it deletes all of these public-owner seams:

- the `export use ../yzx/*.nu *` public command registry in `nushell/scripts/core/yazelix.nu`
- generated extern bridge regeneration as the authoritative source of public command discovery
- public Nushell wrapper parsing for the migrated command families
- public help/completion introspection that depends on loading the Nushell command tree

It must also delete at least one of these family-level owners:

- `nushell/scripts/yzx/env.nu` and `nushell/scripts/yzx/run.nu` as public command wrappers
- the public `launch` / `enter` / `restart` wrappers once their remaining ownership is narrow enough
- the public `status` / `doctor` command owner path once their responsibilities are split clearly enough

If a proposal keeps those surfaces and merely adds a Rust root dispatcher above them, the rewrite should be rejected.

### Allowed Transition Shape

If Rust becomes the public `yzx` owner in v16, surviving Nushell implementations may still exist for intentionally kept surfaces, but only as internal helpers.

Allowed:

- a Rust `yzx` binary dispatching to dedicated internal Nushell entrypoints for commands that are intentionally still Nushell-owned
- Rust-owned help/completion metadata and argument parsing for the whole public command surface

Not allowed:

- Rust shelling back into `use nushell/scripts/core/yazelix.nu *; yzx ...`
- keeping the existing Nushell command tree as a parallel public parser/help/completion owner
- using clap only as a thin argv shim over the unchanged public Nushell tree

### Go / No-Go Gates

Go only if all of the following are true:

1. `yzx env` and `yzx run` are ready to become first-class public Rust commands without changing their contract
2. at least one mixed family has a narrow enough internal boundary that Rust can own the public contract without inheriting fuzzy product semantics
3. the generated extern bridge can be deleted or reduced to a compatibility surface instead of remaining part of steady-state command discovery
4. a parity harness exists for help output, exit behavior, and the migrated command families
5. the proposal names the exact Nushell modules that disappear or stop being public owners

No-go signals:

- most user-facing commands would still immediately hand off to the old public Nushell tree unchanged
- detached terminal execution or shell-boundary logic becomes the main reason to move the command
- the proposal cannot name at least one real family-level deletion
- the case for the rewrite depends mostly on "Rust is faster" without a command-owner simplification story

### Benchmark And Maintainability Bar

The justification should be measured primarily by deletion and maintainability, with performance as supporting evidence.

Reasonable supporting wins:

- cold `yzx --help` and shell completion discovery avoid loading the Nushell command tree
- noninteractive control-plane commands have a tighter startup path
- one command-metadata source replaces the current help/extern duplication

Insufficient justification by itself:

- trying to beat interactive startup or terminal launch latency, which is still dominated by Nushell, generated-state work, terminal startup, and Zellij handoff

## Non-goals

- forcing all user-facing commands into Rust in one pass
- moving renderer/template ownership out of Nushell
- moving detached terminal execution or shell quoting into Rust just because the public CLI moved
- folding the Rust pane orchestrator into `rust_core`
- porting maintainer-only tooling or Nix/package ownership into Rust as part of the same decision

## Acceptance Cases

1. A maintainer can tell when a broader Rust public-CLI rewrite is actually justified instead of guessing from Nushell size
2. The proposal names which command families are valid v16 Rust candidates and which should remain Nushell-owned
3. The minimum deletion budget is explicit, including the public command registry and extern-bridge seams
4. The document states concrete go/no-go gates and rejects clap-as-thin-shim proposals
5. The current v15 helper-backed Rust path remains the default until those gates are met

## Verification

- `nu nushell/scripts/dev/validate_specs.nu`
- manual review against:
  - `docs/specs/rust_migration_matrix.md`
  - `docs/specs/rust_nushell_bridge_contract.md`
  - `docs/specs/launch_bootstrap_rust_migration.md`
  - `docs/specs/yzx_command_surface_backend_coupling.md`
  - `docs/specs/cross_language_runtime_ownership.md`
  - `docs/specs/v15_trimmed_runtime_contract.md`

## Traceability

- Bead: `yazelix-2ex.1.11`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
