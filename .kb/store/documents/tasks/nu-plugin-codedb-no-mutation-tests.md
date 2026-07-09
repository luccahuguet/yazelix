---
id: 019f21a8-8059-72f2-951f-816e3a3137e4
slug: tasks/nu-plugin-codedb-no-mutation-tests
title: "Add CodeDB no-mutation tests"
type: task
status: completed
priority: high
tags: [codedb, nu_plugin, tests, no_mutation, CDB044]
---

## Source Task

`/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv` row CDB044.

- Phase: tests
- Depends on: CDB028; CDB041
- Blocks: CDB046
- Target surface: tests
- Allowed files: `tests/**`
- Forbidden: source mutation
- Primary artifact: test outputs
- Execution gate: clean/dirty no-mutation tests pass
- Raw log: `logs/CDB044-no-mutation-tests.log`
- PRD sections: 15.1, 19

## Acceptance Criteria

- [x] No-mutation test exists under `tests/**`.
- [x] Test proves a clean Git fixture stays clean after CodeDB runner proof.
- [x] Test proves a pre-existing dirty Git fixture remains dirty but unchanged after CodeDB runner proof.
- [x] Test verifies `proof_status = proven`, `mutation_detected = false`, and correct `pre_existing_dirty` values.
- [x] Test uses temporary fixture copies and does not mutate source fixtures.
- [x] Validation records evidence in `logs/CDB044-no-mutation-tests.log`.

## Notes

- Added `tests/test_no_mutation.nu`.
- The test copies `fixtures/clean_repo` to temporary repositories, warms Cargo
  metadata before Git initialization, commits a stable baseline, and then checks
  CodeDB `runner_proof_manifest` no-mutation rows.
- Dirty-state coverage modifies one tracked file and creates one untracked file
  before running CodeDB. The before/after porcelain Git status must match exactly.
- The source fixture lockfile guard prevents accidental `Cargo.lock` mutation in
  `fixtures/**`.

## Completion Evidence

- Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB044-no-mutation-tests.log`.
- Command:
  `CODEDB_TEST_CARGO_DIR=/nix/store/xlli1a8m35h5kwavjajnp6nl90xmjgcx-cargo-1.94.0/bin nu tests/test_no_mutation.nu`.
- Evidence:
  - `clean_repo`: `proof_status = proven`, `pre_existing_dirty = false`, `mutation_detected = false`.
  - `dirty_repo`: `proof_status = proven`, `pre_existing_dirty = true`, `mutation_detected = false`.
  - lane marker present: `# Test lane: default`.
  - contract marker present: `# Defends: CodeDB no-mutation proof preserves clean and pre-existing dirty Git states.`
  - `find fixtures -name Cargo.lock` returned none after validation.
