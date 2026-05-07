# LOC Extraction Scorecard

This is the measurement policy for delete-first refactors and first-party child repository extractions. The scorecard exists so Yazelix can tell the difference between real main-repo simplification, churn that adds more validation/docs than it removes, and extraction that only moves code on paper.

## Baseline

Use `v16.3` as the first baseline for the post-child-repo extraction cycle.

The primary target is main-repo runtime ownership. Child-repo LOC, maintainer tooling, docs, tests, packaging, generated fixtures, and binary assets are reported separately so extraction does not look successful merely because code moved to another repository.

For each deletion or extraction bead, report:

- baseline ref, usually the previous completed bead commit or `v16.3` for the first pass
- candidate ref, usually `HEAD`
- raw diff insertions, deletions, and net line change
- `tokei` code LOC before and after
- category diff for runtime, maintainer, tests, docs, generated fixtures, packaging, and assets
- whether main-repo ownership actually decreased
- any child-repo LOC added outside the main repo
- any deferred deletion debt

If accepted product work raises a no-growth ceiling, refresh the relevant inventory and record the increase as budget debt unless the new owner directly replaces a larger deleted owner. Do not close an extraction bead on a repository split alone.

## Spartan Protocol

This protocol is intentionally stricter than the measurement policy. It exists because Yazelix can keep extracting child repos while the main project still grows through adapters, validators, fixtures, docs, compatibility shims, and generated clutter.

For extraction, cleanup, refactor, validator, generated-fixture, and command-surface beads:

- the default success metric is lower main-repo ownership, not total work performed
- a child repo split is incomplete until the main repo deletes code, stops owning a contract, or reduces runtime closure/storage for users who opt out
- a bead that claims cleanup/refactor/extraction should not increase main-repo runtime, maintainer, tests, generated, or packaging LOC
- any accepted growth above `100` main-repo code LOC must name a payback bead before the work is closed
- validators, generated fixtures, docs, wrappers, compatibility shims, and adapters are part of the cost, not free bookkeeping
- stale scaffolding created for an extraction should be deleted in the same bead unless there is a concrete risk that needs a separate follow-up
- when a budget family shrinks, lower the ceiling in the same commit so future work cannot spend the savings silently
- prefer rejecting a feature or extraction over adding a configurable abstraction that keeps both old and new owners alive

Use these labels in Beads when recording the result:

- `net_shrink`: main-repo ownership decreased and the budget ceiling was ratcheted down if applicable
- `flat`: behavior or ownership improved but main-repo LOC did not materially change
- `budget_debt`: accepted product behavior increased main-repo LOC and has a named payback bead
- `paper_extraction`: code moved out but main-repo ownership did not decrease enough; this is not complete extraction success

## Counted Surfaces

`runtime` covers shipped user behavior: `rust_core/yazelix_core`, `rust_plugins`, `nushell`, `configs`, `user_configs`, `shells`, and default config files.

`maintainer` covers repo-only tooling and gates: `rust_core/yazelix_maintainer`, `maintainer_shell.nix`, `.github`, maintainer workflow helpers, and local validator policy.

`tests` covers Rust and Nushell tests, fixtures whose purpose is test execution, and test support modules.

`docs` covers `README.md`, `CHANGELOG.md`, and `docs/`.

`generated` covers generated examples, metadata snapshots, schema outputs, and fixtures that mirror generated config rather than owning product behavior.

`packaging` covers flake, Nix packaging, Home Manager module wiring, overlays, and runtime package assembly.

`assets` covers images, shaders, wasm binaries, and other non-source payloads. Binary assets are counted by file/size, not text LOC.

## Exclusions

Exclude `.beads`, `.git`, `target`, build outputs, local caches, result symlinks, and wasm binaries from `tokei` code LOC.

Do not exclude tracked wasm/assets from ownership discussion. They should appear as binary or storage impact even when they are excluded from code LOC.

## Commands

Use the checked-in helper for normal before/after reports:

```bash
shells/posix/yazelix_loc_scorecard.sh v16.3 HEAD
```

For a focused raw diff:

```bash
git diff --shortstat v16.3..HEAD -- . ':(exclude).beads/*'
git diff --numstat v16.3..HEAD -- . ':(exclude).beads/*'
```

For direct code LOC snapshots:

```bash
tokei --exclude .beads --exclude target --exclude '*.wasm' .
```

For storage or closure impact, use Nix only when the change plausibly affects runtime closure size:

```bash
nix path-info -Sh .#yazelix
nix path-info -Sh .#runtime
nix path-info -Sh .#yazelix_bar
```

## Child Repos

Child-repo LOC is reported separately from main-repo LOC. A child extraction is successful only when at least one of these is true:

- main-repo code or generated clutter decreases
- the main repo stops owning a subsystem contract directly
- runtime closure size decreases for users who disable or do not consume the subsystem
- the extracted API is smaller and clearer than the old internal boundary

Creating a separate repository without reducing main-repo ownership is deferred deletion debt, not a completed simplification.

Adapters retained in this repo should be thin. If an extraction leaves a large adapter, generated mirror, compatibility layer, or validator here, record it under deferred deletion debt and create the owner bead immediately.

## Report Template

```text
Baseline: <ref>
Candidate: <ref>

Raw diff excluding .beads:
- files: <n>
- insertions: <n>
- deletions: <n>
- net: <+/-n>

Tokei code LOC excluding .beads, target, and wasm:
- baseline: <n>
- candidate: <n>
- delta: <+/-n>

Category diff:
- runtime: <+/-n>
- maintainer: <+/-n>
- tests: <+/-n>
- docs: <+/-n>
- generated: <+/-n>
- packaging: <+/-n>
- assets/binary: <files/bytes when relevant>

Main-repo ownership result:
- deleted:
- moved:
- still owned:
- deferred deletion debt:

Child-repo impact:
- repo:
- code LOC added:
- package/closure impact:
```
