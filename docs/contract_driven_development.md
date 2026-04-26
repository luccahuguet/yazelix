# Contract-Driven Development

## Summary

Yazelix uses contract-driven development to converge toward canonical form:

- one owner per behavior
- no missing layer
- no extra layer
- no duplicated owner
- no weak governed tests
- no surviving Nushell without an explicit reason to exist

This is a delete-first protocol, not a paperwork protocol. The goal is to make
behavior, ownership, and verification sharp enough that Yazelix can remove dead
code, collapse bridge layers, and keep only strong tests without losing real
functionality.

## Durable Inputs

Use these files together:

- [`AGENTS.md`](../AGENTS.md)
  - the short workflow and command-surface entrypoint
- [`docs/spec_driven_workflow.md`](./spec_driven_workflow.md)
  - when a change needs a spec and the minimum spec shape
- [`docs/specs/canonical_contract_item_schema.md`](./specs/canonical_contract_item_schema.md)
  - the canonical contract-item schema and ID policy
- [`docs/specs/config_runtime_control_plane_contract_item_pilot.md`](./specs/config_runtime_control_plane_contract_item_pilot.md)
  - the mixed-subsystem pilot that proved the schema on real Yazelix seams
- [`docs/specs/test_suite_governance.md`](./specs/test_suite_governance.md)
  - lane, strength, and test-retention policy
- [`docs/specs/spec_inventory.md`](./specs/spec_inventory.md)
  - which specs are live, planning, historical, or template-only

## Core Model

### Beads own planning state

Beads own:

- scope
- priority
- dependencies
- status
- execution history
- implementation sequencing

Beads do not replace specs or tests. They answer "what work is happening" and
"what blocks what."

### Specs own normative contract items

Specs own the supported behavior and boundary contract.

The durable unit is a contract item, not a whole file and not every sentence.
Only normative statements should get IDs:

- behavior
- invariants
- subsystem boundaries
- ownership rules
- supported failure modes
- explicit non-goals

Rationale, examples, implementation sketches, and historical notes should stay
plain prose unless they are themselves normative.

### Tests and validators defend live contracts

Tests and validators exist to defend a live contract item, a concrete
regression, or a maintained invariant.

That means:

- a broad file-level marker like `Defends: docs/specs/test_suite_governance.md`
  is governance, not enough by itself for long-term default-lane traceability
- touched default-lane tests should usually carry at least one concrete
  `Contract:` item ID once the item exists
- regression-only tests are allowed when they preserve bug memory that is
  narrower than the main spec contract
- validators should own cheap structural or source-of-truth checks when they
  are cheaper and sharper than a heavier behavior test

### Delete-first audits own simplification pressure

Before implementation, Yazelix should audit a subsystem in planning space:

1. question the requirement
2. name the retained behavior
3. identify the canonical surviving owner
4. identify bridges, wrappers, or duplicate owners that can be deleted
5. name the verification that will remain after deletion
6. create delete-first beads from the findings

The audit should not quietly assume that every current layer deserves to exist.
Use
[`docs/subsystem_canonicalization_audit_template.md`](./subsystem_canonicalization_audit_template.md)
so audits produce the same feature-preservation output instead of vague cleanup
notes.

## Mechanical Checks

The current ratchet is enforced by these validators:

- `yzx_repo_validator validate-specs`
  - validates spec traceability plus indexed contract-item structure
- `yzx_repo_validator validate-default-test-traceability`
  - validates governed Nu test metadata, nearby `Contract:` markers, and the
    default-suite file-level ratchet
- `yzx_repo_validator validate-rust-test-traceability`
  - validates governed Rust test metadata and nearby `Contract:` markers
- `yzx_repo_validator validate-package-rust-test-purity`
  - validates default/package-time Rust tests do not execute host-only tools
    such as `nix` or `home-manager`; Nix-dependent metadata checks belong in
    maintainer validators such as `validate-config-surface-contract`
- `yzx_repo_validator validate-pane-orchestrator-sync`
  - validates tracked pane-orchestrator wasm sync metadata against the current
    source so source edits cannot silently ship with stale plugin artifacts

Current migration-safe debt that is allowed but must shrink is tracked in
[`docs/contract_traceability_quarantine.toml`](./contract_traceability_quarantine.toml).
New file-level policy-only debt should fail unless it is explicitly quarantined
behind a real bead.

## Contract Item Rules

The canonical schema lives in
[`docs/specs/canonical_contract_item_schema.md`](./specs/canonical_contract_item_schema.md).
The short version:

- item IDs use `PREFIX-NNN`
- each item declares `Type`, `Status`, `Owner`, `Statement`, and
  `Verification`
- statuses are `live`, `planning`, `deprecated`, `historical`, or
  `quarantine`

Use `quarantine` for temporary traceability or ownership debt that must not
expand and must carry an exit bead. Do not use it as a long-term parking lot.

The config/runtime/control-plane pilot shows the intended level of sharpness:

- live items name the Rust owner, bridge boundary, failure contract, or parity
  invariant
- the surviving Nu bridge layer is called out as `quarantine` debt instead of
  being left implicit
- the pilot records weak-traceability debt without deleting tests or code

## When To Write Or Update A Spec

Write or update a spec when the change affects:

- user-visible behavior
- a subsystem boundary
- supported failure behavior
- ownership of a real behavior or source of truth
- a deletion that could remove or shrink a maintained surface

You usually do not need a spec for:

- typo fixes
- copy edits
- comment-only cleanup
- purely local refactors with no contract change

If delete-first work changes which layer owns a real behavior, it is not a
purely local refactor.

## How To Add Or Change A Contract

1. Open or claim the Bead
2. Decide whether the work changes a live behavior, boundary, or owner
3. If yes, write or update the spec
4. Add or update the relevant contract items
5. Implement only after the retained behavior, surviving owner, and
   verification path are explicit
6. Update tests or validators so the surviving contract stays defended
7. Close the Bead only when implementation and verification match the contract

## Test Outcomes: Keep, Demote, Quarantine, Delete

When reviewing governed tests, use these outcomes:

- `keep`
  - the test still defends a live contract, regression, or maintained
    invariant with acceptable strength for its lane
- `demote`
  - the test still has value, but it belongs in a cheaper, heavier, or more
    explicit lane
- `quarantine`
  - the test carries temporary traceability debt or protects a still-live
    transition, but it must not expand and must carry an exit bead
- `delete`
  - the test is weak, redundant, or orphaned and no longer defends a retained
    behavior

Delete-first does not mean "delete all tests first." Do not delete the only
known executable defense of a live behavior until replacement verification or
explicit contract retirement exists.

## What Good Looks Like

A good contract-driven change has these properties:

- the surviving behavior is named explicitly
- the surviving owner is singular or the split boundary is explicit
- the deleted layer is named, not hand-waved
- the test or validator path is concrete
- the spec distinguishes live contract from rationale and history
- future work can build on the new seam without reopening the same ambiguity

## What To Avoid

- indexing every prose sentence as if it were normative
- leaving bridge layers implicit just because they are small
- keeping a weak default-lane test because it "checks something"
- using compatibility wrappers as the default answer when owner collapse is
  possible
- deleting code or tests without naming the retained behavior and verification
- duplicating this whole protocol into `AGENTS.md`

## References

- NASA Systems Engineering Handbook appendix on requirements traceability and
  verification matrices
  - <https://www.nasa.gov/reference/system-engineering-handbook-appendix/>
- Google Testing Blog, Test Sizes
  - <https://testing.googleblog.com/2010/12/test-sizes.html>
- Google Testing Blog, Just Say No to More End-to-End Tests
  - <https://testing.googleblog.com/2015/04/just-say-no-to-more-end-to-end-tests.html>
- The Rust Programming Language, Test Organization
  - <https://doc.rust-lang.org/book/ch11-03-test-organization.html>
- Cargo Book, `cargo test`
  - <https://doc.rust-lang.org/cargo/commands/cargo-test.html>
- cargo-nextest
  - <https://nexte.st/>
- cargo-mutants
  - <https://docs.rs/crate/cargo-mutants/latest>
- cargo-mutants limitations
  - <https://mutants.rs/limitations.html>
- Rust Fuzz Book
  - <https://rust-fuzz.github.io/book/>
