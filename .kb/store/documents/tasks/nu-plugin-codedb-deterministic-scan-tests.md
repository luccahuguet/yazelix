---
id: 019f21a1-8d2b-71b2-8a0b-36cd0ff4695c
slug: tasks/nu-plugin-codedb-deterministic-scan-tests
title: "Add CodeDB deterministic scan tests"
type: task
status: completed
priority: high
tags: [codedb, nu_plugin, tests, determinism, CDB042]
---

## Source Task

`/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv` row CDB042.

- Phase: tests
- Depends on: CDB041; CDB029
- Blocks: CDB046
- Target surface: tests
- Allowed files: `tests/**`
- Forbidden: network dependency
- Primary artifact: test outputs
- Execution gate: repeat scan checksums stable
- Raw log: `logs/CDB042-determinism.log`
- PRD sections: 19

## Acceptance Criteria

- [x] Deterministic scan test exists under `tests/**`.
- [x] Test proves repeated `codedb scan` output is stable for an unchanged fixture.
- [x] Test proves repeated `codedb export codedb_table_checksums` output is stable for the same fixture.
- [x] Test avoids network dependencies and uses local fixture input.
- [x] Test does not mutate source fixtures.
- [x] Validation records evidence in `logs/CDB042-determinism.log`.

## Notes

- Added `tests/test_deterministic_scan.nu`.
- The test copies `fixtures/single_simple_crate` to a temporary directory before
  scanning. This isolates Cargo's expected `Cargo.lock` creation from canonical
  fixture inputs.
- The first temp scan is a warmup so measured repeated scans compare a stable
  fixture state after Cargo metadata has created its lockfile in the temp copy.
- During development, scanning the source fixture in place created
  `fixtures/single_simple_crate/Cargo.lock`; that generated file was removed,
  and the final validation proves no source fixture lockfile remains.

## Completion Evidence

- Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB042-determinism.log`.
- Command:
  `CODEDB_TEST_CARGO_DIR=/nix/store/xlli1a8m35h5kwavjajnp6nl90xmjgcx-cargo-1.94.0/bin nu tests/test_deterministic_scan.nu`.
- Evidence:
  - `scan_summary` repeated-output SHA-256 comparison passed.
  - `table_checksums` repeated-output SHA-256 comparison passed.
  - lane marker present: `# Test lane: default`.
  - contract marker present: `# Defends: repeated CodeDB scans of unchanged fixtures produce stable table outputs.`
  - `find fixtures -name Cargo.lock` returned none after validation.
