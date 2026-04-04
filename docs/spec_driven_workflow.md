# Spec-Driven Workflow

Yazelix should be spec-driven for changes that alter user-visible behavior or subsystem contracts.

This does not mean every tiny edit needs a design memo. It means that when a change affects how Yazelix behaves, there should be a short written contract before or during implementation.

## Why

Specs exist to reduce drift between:

- the intended behavior
- the implemented behavior
- the regression tests
- the Beads issue that tracks the work

In Yazelix, specs are especially valuable when a change crosses subsystem boundaries:

- workspace behavior
- runtime/config behavior
- integration behavior
- maintainer workflow contracts

## What Needs A Spec

Write a spec for changes like these:

- new user-visible features
- user-visible behavior changes
- bug fixes that clarify or redefine a contract
- cross-subsystem behavior changes
- integration behavior that should be considered supported
- changes that need non-obvious acceptance criteria or regression coverage

Examples:

- changing how new tabs inherit workspace roots
- defining what `yzx cwd` should synchronize
- changing Home Manager ownership boundaries
- adding a new `yzx` command with real behavior

## What Usually Does Not Need A Spec

You usually do not need a spec for:

- typo fixes
- copy edits
- comment-only cleanup
- purely local refactors with no behavioral change
- dependency bumps with no changed product contract
- test-only cleanup that does not change what is supported

If a refactor changes a boundary or supported behavior, it stops being "purely local" and should get a spec.

## Where Specs Live

Specs live under:

- [`docs/specs/`](/home/lucca/pjs/yazelix/docs/specs)

Each spec should be a small Markdown file with an underscore-based name that matches the repo convention.

Examples:

- `docs/specs/workspace_root_inheritance.md`
- `docs/specs/home_manager_ownership.md`
- `docs/specs/floating_tui_panes.md`

Use [`docs/specs/template.md`](/home/lucca/pjs/yazelix/docs/specs/template.md) as the starting point.

## Relationship To Beads

Beads still own planning state:

- priority
- dependencies
- status
- execution history

Specs own the behavior contract.

The practical convention is:

1. The bead describes the work.
2. The spec describes the intended behavior.
3. Tests or checks defend the important acceptance cases from the spec.

Each real spec should also include a small `Traceability` section so the links stay visible in the file itself:

- one `Bead` line
- one or more `Defended by` lines pointing to concrete tests or validator commands

Later traceability automation can tighten this further, but this is the current convention.

## Required Sections

Every spec should include:

1. Summary
2. Why
3. Scope
4. Behavior
5. Non-goals
6. Acceptance cases
7. Verification
8. Traceability
9. Open questions

Keep it short. If a spec is becoming a large essay, it probably covers too much scope.

## Acceptance Case Rules

Acceptance cases should be:

- observable
- user-facing or boundary-facing
- testable
- specific enough that failure is obvious

Good:

- "A new tab opened from an explicit project-root tab inherits that tab workspace root."
- "Once the linked bead is updated locally, the GitHub/Beads contract validator passes for the closed issue."

Bad:

- "The UX feels better."
- "The code is cleaner."
- "The system is more modular."

## Verification Rules

A spec should name the checks that prove the behavior.

That can include:

- unit tests
- integration tests
- CI checks
- one-shot reproducible scripts
- explicit manual verification when automation is not yet practical

The important thing is that the verification path is concrete.

## Style Rules

- Prefer short specs over comprehensive prose.
- Write in terms of behavior, not implementation preference.
- Separate "what should happen" from "how we might implement it".
- Name subsystem boundaries explicitly when relevant.
- Add non-goals so the spec does not sprawl.

## Recommended Flow

1. Open or claim the Bead.
2. Decide whether the change needs a spec.
3. If yes, create a spec from the template.
4. Implement against the spec.
5. Add or update verification for the acceptance cases.
6. Close the Bead once behavior and verification match.

## Current Scope

This workflow is intentionally small.

Yazelix is not trying to become a heavyweight process machine. The goal is:

- fewer fuzzy behavior changes
- clearer acceptance criteria
- stronger regression coverage

If the workflow starts feeling bureaucratic, the answer is to make the specs smaller and sharper, not to abandon explicit contracts.
