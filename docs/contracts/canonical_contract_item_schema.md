# Canonical Contract Item Schema

## Summary

Yazelix should use an indexed contract item as the durable unit of contract and
test traceability. The unit is a normative contract item, not every prose line.
Each live item should say what behavior or boundary exists, who owns it, and
how it is verified. Rationale, examples, and historical explanation should stay
as prose so the system stays sharp instead of bureaucratic.

## Why

Current Yazelix contracts and tests are partly traceable, but the traceability is
still too broad for delete-first architectural work. File-level references such
as "this test defends `test_suite_governance.md`" do not say which behavior is
protected, which owner is canonical, or what can safely be deleted.

That matters more now because Yazelix is actively collapsing Nushell owners,
burning bridge layers, and trying to keep only strong tests. Without a smaller
contract unit:

- duplicate owners can survive because the surviving owner is not named
- weak tests can linger because they "defend a file" without defending a real
  behavior
- delete-first work can accidentally drop the last executable defense of a live
  behavior
- planning docs can sprawl because examples, rationale, and historical notes
  look equally binding

External references support this direction, but Yazelix should keep the
implementation lightweight:

- NASA's systems-engineering guidance favors stable requirement identifiers and
  explicit verification mapping
- Google's testing guidance favors small, reliable, high-signal checks over
  broad end-to-end accumulation
- Rust's testing model already separates unit and integration concerns cleanly,
  so Yazelix does not need a heavyweight external requirements tool just to
  connect behavior to verification

Yazelix is still a maintainer-driven monorepo. Git-reviewed Markdown plus local
validators are enough if the schema is precise and enforced incrementally.

## Scope

This contract defines:

- the canonical contract item unit for Yazelix contracts
- the allowed item ID format
- the allowed item types and statuses
- the required versus optional item fields
- the verification modes a contract item may use
- when a governed test should reference a contract item versus a narrower
  regression or invariant
- migration rules for existing contracts and tests
- feature-preservation rules for delete-first work

## Behavior

### Contract Unit

The contract unit is a normative item, not a sentence and not a whole file.

Only normative statements should get contract IDs. In practice that means:

- supported user-visible behavior
- explicit invariants
- subsystem boundaries
- ownership rules
- failure-mode promises
- explicit non-goals that the project intends to preserve

The following should usually stay plain prose without their own IDs:

- rationale
- examples
- implementation sketches
- historical explanation
- migration history that is no longer binding

### Item ID Format

Every contract item ID should use this shape:

- `PREFIX-NNN`

Rules:

- `PREFIX` is `2-8` uppercase ASCII letters or digits
- `NNN` is a zero-padded decimal sequence of at least three digits
- the prefix should be stable for a subsystem or contract family
- numbers should increase monotonically within a prefix
- once assigned, an ID is never reused for a different meaning

Examples:

- `CFG-001`
- `LAUNCH-004`
- `YZX-012`
- `WS-007`
- `DOCTOR-003`

IDs do not need to be globally sequential across the whole repo. Prefix plus
number is the durable key.

### Allowed Item Types

Every indexed item should declare exactly one type:

- `behavior`: an observable supported behavior
- `invariant`: a state or truth that must remain true
- `boundary`: an integration or subsystem edge
- `ownership`: the canonical owner of a behavior, lifecycle, or source of truth
- `failure_mode`: a supported error or rejection contract
- `non_goal`: an explicitly unsupported or intentionally omitted behavior

### Allowed Item Statuses

Every indexed item should declare exactly one status:

- `live`: current maintained contract
- `planning`: agreed target or future contract, not yet the shipped truth
- `deprecated`: still present enough to preserve during removal work, but meant
  to disappear
- `historical`: design history only, not binding for new implementation
- `quarantine`: temporarily retained legacy behavior or traceability debt that
  must not expand and must carry an exit condition

Rules:

- default-lane tests should normally map to `live` items
- `deprecated` items may still be defended while a removal transition is active
- `planning` and `historical` items are not valid default-lane test targets
- `quarantine` is a temporary migration state, not a place to leave behavior
  indefinitely

### Required Item Fields

For `live`, `planning`, `deprecated`, and `quarantine` items, the contract should
record these fields:

- `ID`
- `Type`
- `Status`
- `Owner`
- `Statement`
- `Verification`

Optional fields:

- `Notes`
- `Source`
- `Related issue`
- `Deletion note`

Suggested Markdown shape:

```md
#### CFG-001
- Type: invariant
- Status: live
- Owner: Rust `config_normalize`
- Statement: The managed main Yazelix config surface is `yazelix.toml`
- Verification: automated `nushell/scripts/dev/test_yzx_core_commands.nu`; validator `yzx_repo_validator validate-contracts`
- Notes: Home Manager must render the same option/default contract
```

`historical` items may omit `Verification` when they are design record only.

### Verification Field Semantics

`Verification` must classify how the item is checked. Allowed modes are:

- `automated`: a governed test or bundle
- `validator`: a structural validator or CI check
- `manual`: an explicit manual verification path
- `unverified`: intentionally unverified for now, with a reason and exit condition

Rules:

- `live` items should normally have at least one `automated`, `validator`, or
  `manual` entry
- `unverified` is allowed only when the reason is explicit and an owning issue
  or stop condition is named
- a `live` item with `unverified` must not silently stay that way forever; it
  is migration debt
- `planning` and `historical` items may be unverified without blocking the
  branch contract, because they are not the shipped truth

### Test Reference Rules

Governed tests should keep their semantic reason markers:

- `# Defends:` / `// Defends:`
- `# Regression:` / `// Regression:`
- `# Invariant:` / `// Invariant:`

In addition, the contract schema defines a new optional traceability marker:

- `# Contract: CFG-001`
- `// Contract: CFG-001`

Multiple IDs may be listed on one line, separated by commas.

Use `Contract:` when the test is defending a stable normative item that already
exists in a contract.

Use a narrower `Regression:` or `Invariant:` without `Contract:` only when at
least one of these is true:

1. the test preserves bug memory that is narrower than the stable contract
2. the test defends a source-of-truth invariant that is too small to deserve a
   permanent top-level contract item
3. the contract migration has not landed yet, and the test is carrying
   temporary traceability debt

Rules:

- new or touched default-lane tests should normally carry at least one
  `Contract:` ID when defending stable behavior
- regression-only tests are allowed, but they must still name the bug or
  invariant explicitly and should point to the related contract when that
  mapping debt is temporary
- a broad policy-only reference such as only defending
  `docs/contracts/test_suite_governance.md` is not sufficient for default-lane test
  traceability
- a regression test may carry both `Regression:` and `Contract:` when it is
  defending a stable contract and preserving bug memory at the same time

### Migration Rules

The migration should be incremental rather than a big-bang rewrite.

Immediate rules:

- new contracts that define live normative behavior should use contract items
- touched contracts should add IDs only for the live normative items relevant to the
  current change
- existing historical docs do not need blanket backfilling unless they still
  drive live planning or live validation
- existing governed tests may remain unmapped temporarily while the inventory
  and validator ratchet land
- when a contract is split or merged, surviving item IDs should stay stable where
  practical; retired IDs should not be reused

### Feature-Preservation Rules For Delete-First Work

Delete-first work must preserve supported behavior intentionally, not by habit.

Before code, aliases, wrappers, bridges, contracts, or tests are removed, the work
must name:

- which contract items are retained
- which contract items are retired or demoted
- the canonical surviving owner for the retained behavior
- the verification path that remains after deletion

Additional rules:

- no test should be deleted while it is the only known executable defense of a
  live behavior unless replacement verification or explicit contract retirement
  is recorded
- bridges and glue seams that survive should carry either an exit path or a
  no-go reason
- command aliases are only part of the contract when Yazelix explicitly intends
  to support them; compatibility should not survive by inertia
- `quarantine` items must always name the owner or condition that will either
  delete, remap, or retire them

## Non-goals

- indexing every line of every contract
- rewriting every historical contract in one pass
- deleting the current test suite wholesale
- choosing Rust over Nushell by process rule alone
- introducing a heavyweight external requirements database or enterprise
  workflow tool

## Acceptance Cases

1. A maintainer can write one live contract item with a stable ID, named owner,
   and explicit verification without turning the whole contract into a database dump
2. A touched default-lane test defending stable behavior can keep its semantic
   `Defends` or `Regression` marker and add a `Contract:` ID without needing a
   second parallel test taxonomy
3. A regression-only test is still allowed when it is narrower than the stable
   contract, but the traceability debt is explicit instead of hidden
4. Delete-first removal work can say which behaviors survive, which owner keeps
   them, and which tests or validators remain after deletion
5. Historical explanation and design rationale remain readable prose instead of
   being forced into one-to-one item IDs

## Verification

- unit tests: n/a
- integration tests: n/a
- CI checks: `yzx_repo_validator validate-contracts`
- manual verification:
  - review this schema against `docs/contract_driven_development.md`
  - review this schema against `docs/contracts/test_suite_governance.md`

## Traceability
- Defended by: `yzx_repo_validator validate-contracts`

## Open Questions

- Should `deprecated` and `quarantine` items both be acceptable default-lane
  `Contract:` targets, or should `quarantine` be limited to migration-only
  validators and explicit temporary exceptions?
- How strict should the first validator ratchet be for untouched historical
  tests that currently defend only broad policy docs?
