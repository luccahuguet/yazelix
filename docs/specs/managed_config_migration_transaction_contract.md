# Managed Config Migration Transaction Contract

## Summary

Yazelix should treat managed config migrations as a small staged transaction over a narrow set of Yazelix-owned config surfaces, not as a sequence of ad hoc direct writes. The contract should guarantee that a migration either commits a coherent new managed config set or leaves the previously valid managed config set intact.

This is not a promise that every legacy config shape can be rewritten automatically. It is a promise that deterministic rewrites, relocations, and related managed-surface changes do not leave users in a half-applied state.

## Why

Yazelix already has the ingredients of a migration system:

- a shared migration registry and preview/apply engine
- startup preflight that can auto-apply safe rewrites
- backup-first write paths for `yzx config migrate --apply`
- legacy root-level relocation into `user_configs`

But the current write model is still direct-write plus backup/copy-back recovery:

- it writes final files in place
- it treats backup files as rollback aids rather than as part of an explicit transaction boundary
- relocation and migration are still adjacent behaviors rather than one clear staged operation

That is good enough for many happy-path cases, but it leaves the repo with fuzzy answers to important questions:

- which files participate in one migration transaction?
- when is a migration considered valid enough to commit?
- what exactly is the rollback source of truth?
- what should startup do if it discovers an interrupted migration attempt?
- how should `yzx config migrate --apply`, `yzx doctor --fix`, and startup preflight share one model instead of slowly drifting apart?

Yazelix does not need a generic database. It does need one explicit, narrow transaction contract for the config surfaces it owns.

## Scope

- define the staged transaction model for Yazelix-owned managed config migrations
- define the participating managed surfaces and distinguish them from input-only legacy surfaces
- define prepare, validate, commit, rollback, and recovery phases
- define where staged artifacts and rollback artifacts live
- define the caller contract shared by:
  - `yzx config migrate --apply`
  - `yzx doctor --fix`
  - entrypoint config-migration preflight
- define what “success” means when manual-only migration items still remain

## Behavior

### Design Goals

- Safety over cleverness:
  - it is better to leave the previous valid config set in place than to partially apply a migration
- Narrow ownership:
  - this transaction contract applies only to Yazelix-owned managed config surfaces
- Small surface area:
  - do not build a generic transaction framework for arbitrary files
- Shared semantics:
  - migration apply, doctor fix, and startup preflight should all share the same transaction model even if they choose different caller behavior afterward

### Participating Surfaces

The transaction contract should be defined around the canonical managed config surfaces:

- main managed config:
  - `~/.config/yazelix/user_configs/yazelix.toml`
- pack sidecar:
  - `~/.config/yazelix/user_configs/yazelix_packs.toml`

These are the only required first-class transaction targets.

The following may participate as input-only or cleanup surfaces, but they are not independent committed outputs:

- legacy root-level main config:
  - `~/.config/yazelix/yazelix.toml`
- legacy root-level pack sidecar:
  - `~/.config/yazelix/yazelix_packs.toml`

The following are explicitly not part of the migration transaction itself:

- imported native upstream configs such as `~/.config/zellij/config.kdl`
- generated runtime state under `~/.local/share/yazelix`
- shell hooks
- desktop entries
- arbitrary host-owned or external files

### Transaction Boundary

The transaction boundary is the coherent set of managed config surfaces that Yazelix will read after migration:

- main config alone
- or main config plus pack sidecar

The contract should never commit one of those surfaces in a way that leaves the pair semantically inconsistent.

That means:

- if a migration splits `[packs]` out of `yazelix.toml`, the transaction owns both resulting managed files
- if a migration only rewrites the main config and the pack sidecar is unchanged, the transaction may stage and commit only the main config
- if no canonical managed surface would change, there is no transaction to commit

### Prepare Phase

The caller must first compute the intended final managed config set entirely in memory:

- resolve canonical target paths
- resolve whether legacy root-level config surfaces are being relocated
- compute the final `yazelix.toml` contents
- compute the final `yazelix_packs.toml` contents when the pack sidecar participates

No final target path should be modified during prepare.

The prepare phase should also assign a transaction id and build a small transaction manifest describing:

- transaction id
- manifest schema version
- caller surface:
  - `config_migrate`
  - `doctor_fix`
  - `entrypoint_preflight`
- participating target files
- optional legacy input files being consumed or cleaned up
- planned backup paths
- planned staged paths

### Artifact Formats

The transaction system should not introduce a new user-authored config format or DSL. It should reuse the existing managed-config payload format and add only one small machine-owned manifest format.

Required format choices:

- canonical managed config payloads remain TOML:
  - `yazelix.toml`
  - `yazelix_packs.toml`
- staged final payloads are also TOML files
- rollback backups are raw TOML copies of the previous canonical targets
- the transaction manifest is a hidden JSON file

Rationale:

- TOML remains the source of truth for actual config content because that is already the canonical user-facing config format
- JSON is a better fit for the hidden transaction manifest because it is strictly machine-owned state, easy to version with a `schema_version`, and does not pretend to be another editable Yazelix config surface

The manifest should be intentionally small and recovery-oriented. It should record only the information needed to decide whether to roll back and how to do it safely, for example:

- `transaction_id`
- `schema_version`
- `caller`
- `phase`
- `targets`
- `staged_path`
- `backup_path`
- `existed_before`
- optional legacy input paths that are pending cleanup

For the first implementation, the transaction journal should be one manifest file plus staged/backed-up TOML payload files. Yazelix does not need SQLite, a database log, or a new mini-language here.

### Staging Location

Staged files and rollback artifacts should live in the same managed config directory tree as the canonical target files, not in `/tmp`.

Rationale:

- same-directory rename is the core atomic primitive we can rely on
- same-directory staging avoids cross-filesystem rename surprises
- the state stays near the config it protects

The contract should use hidden transaction artifacts under the managed config root, for example:

- hidden staged files beside the canonical targets, or
- one hidden transaction directory under `user_configs`

The important contract requirement is:

- staged files must live on the same filesystem as their final targets

### Validation Before Commit

Before commit, Yazelix should validate the staged final config set, not only the migration rule metadata.

Required validation boundary:

1. migration metadata is valid
2. staged main config parses as a valid TOML record
3. staged pack sidecar, when present, parses as a valid TOML record
4. the staged pair satisfies the current config-surface ownership rules:
   - no duplicate pack ownership across both files
   - no invalid sidecar shape
5. the staged final config set is valid enough for the active parser/contract to load it as the managed config set

Important distinction:

- a transaction may still commit even if manual-only migration items remain
- manual-only remaining work is a caller-policy problem, not a transaction-integrity failure

So:

- transaction validation asks “is this staged managed config set internally coherent and readable?”
- caller continuation asks “should startup continue, or should the user still fix manual items first?”

### Rollback Artifacts

Before commit, the transaction should prepare rollback artifacts for every canonical target that will be replaced or removed.

For the first implementation, the rollback source of truth should be explicit backup files for the canonical targets. That is already aligned with current Yazelix behavior and is much simpler than inventing a more abstract undo log.

If a new sidecar is being created rather than replaced:

- no backup file is required for that new target
- the manifest must still record that the target did not exist before the transaction

If legacy root-level files are being relocated:

- they should be treated as input/cleanup paths
- they should not be deleted until the canonical managed targets are successfully committed

### Commit Phase

The commit phase should use the narrow file-transaction pattern that existing systems already rely on:

- prepare and close staged files
- atomically replace each canonical target with a same-directory rename
- only after all canonical targets are in place, clean up legacy input files that were relocated
- only after the whole set is coherent, remove the transaction manifest

This is inspired by the same core primitives used by:

- POSIX `rename()`
- Git lockfiles
- SQLite rollback-journal commit markers

The contract should treat transaction completion as:

- canonical managed targets swapped into place
- legacy consumed inputs cleaned up if applicable
- transaction manifest removed

### Interrupted Recovery

On startup or before a new migration attempt, Yazelix should check for an unfinished managed-config transaction manifest.

Recovery rule:

- if a manifest exists, prefer rollback to the previous valid canonical config set

That means:

- restore canonical targets from rollback backups where needed
- remove staged files
- remove partially created canonical targets that were new in the interrupted transaction
- leave legacy input files untouched unless the commit had fully completed

This is intentionally conservative. It favors “last known valid config set” over trying to guess whether a partially committed migration should be finished forward.

### Caller Contract

All three caller families should share the same transaction engine:

- `yzx config migrate --apply`
- `yzx doctor --fix`
- entrypoint migration preflight

What differs between callers is not the commit model, but what they do after a valid transaction:

- `yzx config migrate --apply`
  - may commit deterministic rewrites and then report remaining manual-only items without blocking the process afterward
- `yzx doctor --fix`
  - may do the same, but present the result through doctor UX
- entrypoint preflight
  - may commit deterministic rewrites, but if manual-only items still remain, it may stop the entrypoint after commit and explain the remaining work

### Non-Goals Of The Contract

This contract should not try to guarantee:

- automatic migration for every legacy config shape
- filesystem-level multi-file atomicity stronger than the underlying rename-based model can provide
- transaction semantics for host-owned external config trees
- transaction semantics for generated runtime state outside the managed config surfaces

## Inspiration

This contract is intentionally narrow and borrows from well-tested patterns instead of inventing a database:

- POSIX and Linux `rename()` semantics:
  - same-filesystem rename is the core atomic replacement primitive
- Git lockfiles:
  - write staged content to a lock/temp file, then atomically replace the destination
- SQLite rollback journals:
  - keep explicit rollback state and use a small commit marker/journal lifecycle to recover from interruption

Yazelix does not need to import those systems wholesale. It should copy the smallest parts that fit the managed config problem.

## Non-goals

- creating a generic transaction framework for arbitrary repo files
- transactionally updating generated runtime configs or desktop entries in the same system
- promising that manual-only config migrations will disappear
- defining the Rust implementation yet

## Acceptance Cases

1. When a deterministic migration rewrites only `yazelix.toml`, Yazelix stages the new file, validates it, and either commits it coherently or restores the previous file.
2. When a deterministic migration moves `[packs]` out of `yazelix.toml` into `yazelix_packs.toml`, Yazelix stages and validates the pair together and does not leave one updated while the other still reflects the old ownership model.
3. When startup preflight auto-applies safe rewrites and then finds manual-only items still remaining, the new deterministic rewrites may stay committed, but the resulting managed config set is still coherent and readable.
4. When a migration transaction is interrupted after backups and staged files exist but before the manifest is cleared, the next startup or migration attempt rolls back to the previous valid canonical config set.
5. When legacy root-level managed config files are being relocated into `user_configs`, the relocation follows the same staged transaction model instead of moving files ad hoc before the rest of the migration logic runs.

## Verification

- manual review against:
  - [config_migration_engine.md](./config_migration_engine.md)
  - [stale_config_diagnostics.md](./stale_config_diagnostics.md)
  - [backend_capability_contract.md](./backend_capability_contract.md)
- current-code review:
  - `nushell/scripts/utils/config_migrations.nu`
  - `nushell/scripts/utils/entrypoint_config_migrations.nu`
  - `nushell/scripts/utils/config_surfaces.nu`
  - `nushell/scripts/yzx/menu.nu`
  - `nushell/scripts/utils/config_diagnostics.nu`
- CI/spec check:
  - `nu nushell/scripts/dev/validate_specs.nu`

## Traceability

- Bead: `yazelix-53zx.1`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`

## Open Questions

- Should `yzx config reset` adopt the same transaction engine in its first implementation pass, or should it remain adjacent until the migration path is proven?
- Should the interrupted-transaction manifest live beside the managed config files or in a dedicated hidden transaction subdirectory under `user_configs`?
- When the migration layer moves to Rust later, should Yazelix keep the same rollback-first recovery policy or add a more explicit forward-commit phase marker?
