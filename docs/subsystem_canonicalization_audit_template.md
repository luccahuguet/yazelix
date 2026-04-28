# Subsystem Canonicalization Audit Template

## Summary

Use this template before running a delete-first subsystem audit in Yazelix.

An audit is planning-only work. It exists to identify the canonical surviving
owner, the bridges that can be burned, the tests and contracts that matter, and the
deletions that are real versus illusory. An audit is not permission to start
deleting code, tests, or docs without naming the retained behavior and the
verification that survives afterward.

## Required Output

Every subsystem audit should produce these sections.

## 1. Subsystem Snapshot

- subsystem name
- purpose in one short paragraph
- user-visible entrypoints
- primary source paths
- upstream or external dependencies if they matter

## 2. Must-Not-Lose Behavior

List the supported behavior or invariants that cannot be dropped by cleanup.

Use one row per behavior:

| Behavior | Current contract or source | Current owner | Current verification | Candidate surviving owner |
| --- | --- | --- | --- | --- |
| example | `CRCP-001` | Rust `yzx_core config.normalize` | `test_yzx_generated_configs.nu` + `yzx_core_config_normalize.rs` | same |

Rules:

- if the behavior is already covered by contract items, reference the IDs
- if no contract item exists yet, name the closest live contract or regression
- if current verification is unknown, say that explicitly before recommending
  deletion

## 3. Canonical Owner Map

For the audited subsystem, name the current owner or split boundary for:

- user-visible behavior
- typed or deterministic logic
- shell or process orchestration
- generated-state writes
- live session or plugin state
- final human-facing rendering

For each split owner, say whether the split is:

- intentional
- temporary bridge debt
- accidental duplication

## 4. Survivor Reasons

For each surviving Nushell, Rust, plugin, Lua, or POSIX layer, explain why it
still deserves to exist.

Use one of these reason classes:

- `canonical_owner`
- `irreducible_shell_boundary`
- `transport_only`
- `external_tool_adapter`
- `temporary_bridge_debt`
- `historical_debt`

If a layer cannot justify one of those reasons, it is a deletion candidate.

## 5. Delete-First Findings

Classify findings into these buckets:

### Delete Now

Code, docs, or tests that can be removed immediately without losing retained
behavior.

### Bridge Layer To Collapse

Temporary transport or compatibility seams that should stop being owners.

### Full-Owner Migration

Real remaining owner cuts where one language or runtime should take over a
behavior end to end.

### Likely Survivors

Layers that should probably remain because they are the right canonical fit.

### No-Go Deletions

Things that look deletable but currently are not, and the exact stop condition
that prevents deletion.

## 6. Quality Findings

Record:

- duplicate owners
- missing layer problems
- extra layer problems
- DRY opportunities
- weak or orphan tests
- only-known-executable-defense tests
- contract gaps
- docs drift

Do not stop at "this feels messy." Name the concrete seam and the retained
behavior it threatens.

## 7. Deletion Classes And Follow-Up Beads

Every follow-up bead created from the audit should name:

- retained behavior
- deletion class
- candidate surviving owner
- verification that must still pass
- explicit stop condition if the deletion turns out to be a no-go

Allowed deletion classes:

- `delete_now`
- `bridge_collapse`
- `full_owner_migration`
- `test_demote_or_delete`
- `spec_rewrite_or_retire`
- `no_go_record`

## Feature-Preservation Gate

An audit is invalid if it recommends deletion without all of these:

- the retained behavior is named
- the current verification is named or explicitly unknown
- the candidate surviving owner is named
- the stop condition is named when deletion is not yet honest

Additional hard rules:

- no code, test, or doc deletion should happen inside the audit itself
- do not delete the only known executable defense of a live behavior without a
  replacement or explicit contract retirement
- do not recommend wrappers as the default answer when an owner seam can be
  collapsed instead
- do not rely on local-only host fixes, ambient caches, or one-off environment
  recovery as the subsystem answer
- do not take ownership of user-managed external config files as a side effect
- preserve the Home Manager and `yazelix_default.toml` parity contract when the
  subsystem touches configuration
- preserve the shell-boundary rule: no new interpolated inline shell-script
  bodies just to call `bash -lc` or similar

## Recommended Audit Flow

1. Read the live contracts, current planning notes, and the relevant code together
2. Fill the must-not-lose behavior table before discussing deletions
3. Draw the current owner map across Nu, Rust, plugins, POSIX shell, and any
   other runtime involved
4. Classify each layer as canonical owner, survivor, bridge debt, or no-go
5. Record weak tests and contract gaps before proposing code removal
6. Create follow-up beads only after the deletion class and stop condition are
   clear

## Deliverable Shape

A completed audit should be easy to scan and should include:

- one short subsystem summary
- one must-not-lose table
- one owner map
- one delete-first findings section
- one quality findings section
- one follow-up bead list

If the audit grows into a long essay, split the subsystem or sharpen the scope.
