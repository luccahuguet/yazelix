# Contract-Driven Development

Yazelix uses contracts to keep supported behavior, ownership, and verification explicit. This is a delete-first workflow, not a paperwork workflow.

## Source Of Truth

Use these durable inputs together:

- [AGENTS.md](../AGENTS.md) for agent workflow and command-surface policy
- [Documentation Architecture](./documentation_architecture.md) for the docs taxonomy
- [Contracts Inventory](./contracts/contracts_inventory.md) for the canonical contracts list
- [Test Suite Governance](./contracts/test_suite_governance.md) for governed test policy
- [Contract Item Schema](./contracts/canonical_contract_item_schema.md) for indexed contract items

## Ownership

Beads own planning:

- scope
- priority
- dependencies
- decision history
- implementation sequencing
- closure evidence

Contracts own current supported behavior:

- user-visible behavior
- subsystem ownership
- integration boundaries
- source-of-truth rules
- supported failure modes
- explicit non-goals
- verification paths

Tests and validators defend live contracts, concrete regressions, and maintained invariants.

## When To Write Or Update A Contract

Write or update a contract when a change affects:

- user-visible behavior
- a subsystem boundary
- supported failure behavior
- ownership of a real behavior or source of truth
- a deletion that could remove or shrink a maintained surface
- a validator or test-governance rule that should remain durable

Do not create a contract for:

- implementation research
- rejected alternatives
- prototype notes
- migration diaries
- backlog sequencing
- broad historical audits
- typo fixes or local refactors with no behavior change

Those belong in Beads, normal maintainer docs, or explicit historical notes.

## Contract Shape

Contracts live under `docs/contracts/`.

A canonical contract should be:

- current-tense
- normative
- scoped to a real supported boundary
- short enough to review
- defended by concrete tests, validators, or explicit manual gates

A canonical contract should not include:

- Bead traceability, unless the contract is specifically about Beads/GitHub planning architecture
- follow-up bead lists
- prototype outcomes
- evaluation logs
- historical implementation diaries
- stale `docs/specs/` links

## Contract Items

Use indexed contract items when tests or validators need stable, granular traceability.

The durable item schema is defined in [Contract Item Schema](./contracts/canonical_contract_item_schema.md).

The short version:

- item IDs use `PREFIX-NNN`
- each item declares `Type`, `Status`, `Owner`, `Statement`, and `Verification`
- live items must name a concrete verification path
- tests should reference item IDs when a broad file-level marker would be too vague

## Delete-First Flow

1. Open or claim the Bead
2. Question whether the behavior or document still deserves to exist
3. Delete obsolete planning, duplicate docs, weak tests, or dead owners first
4. Define the surviving behavior and owner in `docs/contracts/` only when the contract is durable
5. Implement the smallest code or docs change that satisfies the retained contract
6. Verify with focused tests or validators
7. Close the Bead with execution evidence

## Mechanical Checks

Use these validators as the current contract ratchet:

- `yzx_repo_validator validate-contracts`
  - validates canonical contract files, contract items, and contract-surface hygiene
- `yzx_repo_validator validate-default-test-traceability`
  - rejects governed Nu test files from the canonical suite
- `yzx_repo_validator validate-rust-test-traceability`
  - validates governed Rust test metadata and nearby `Contract:` markers
- `yzx_repo_validator validate-package-rust-test-purity`
  - keeps package-time Rust tests away from host-only tools such as `nix` or `home-manager`
- `yzx_repo_validator validate-pane-orchestrator-sync`
  - checks tracked pane-orchestrator wasm sync metadata
- `yzx_repo_validator validate-workspace-session-contract`
  - checks built-in layout metadata, workspace runtime assets, pane-orchestrator commands, and Yazi workspace entrypoints

Temporary traceability debt is tracked in [Contract Traceability Quarantine](./contract_traceability_quarantine.toml). Do not add new broad file-level debt unless it is explicitly quarantined behind a real Bead.
