---
id: 019f219c-19c1-71c3-8d1c-9260fe440f97
slug: tasks/nu-plugin-codedb-fixture-matrix
title: "Create CodeDB fixture matrix"
type: task
status: completed
priority: high
tags: [codedb, nu_plugin, fixtures, CDB041]
---

## Source Task

`/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv` row CDB041.

- Phase: fixtures
- Depends on: CDB012; CDB013
- Blocks: CDB042; CDB043; CDB044; CDB045
- Target surface: fixtures
- Allowed files: `fixtures/**`
- Forbidden: real source mutation
- Primary artifact: fixture workspace
- Execution gate: fixtures present and documented
- Raw log: `logs/CDB041-fixtures.log`
- PRD sections: 18

## Acceptance Criteria

- [x] Fixture matrix exists under `fixtures/**`.
- [x] Fixtures cover the PRD fixture families needed by later scan/security/no-mutation/reproduction tasks.
- [x] Fixture intent and expected CodeDB observations are documented inside `fixtures/**`.
- [x] Fixture crates are self-contained and do not mutate real source repos.
- [x] Validation records evidence in `logs/CDB041-fixtures.log`.

## Notes

- This task creates fixture inputs only. Later tasks own scan, security, no-mutation, and reproduction assertions.
- Keep fixture content intentionally tiny so later tests can run quickly and reason about expected rows.
- The fixture README records the intended boundary: CodeDB owns richer table/blob/provenance/store semantics, while envctl is an edge integration for selected export/import materialization.

## Completion Evidence

- Added `fixtures/README.md` and `fixtures/fixture_matrix.csv` with 14 fixture families:
  `single_simple_crate`, `workspace_two_crates`, `feature_cfg`, `macro_rules`,
  `proc_macro_consumer`, `build_script`, `out_dir_generator`, `include_edges`,
  `non_rust_assets`, `native_link`, `secret_like`, `clean_repo`, `dirty_repo`,
  and `symlink`.
- Added tiny self-contained Cargo fixtures for all buildable crate families and
  portable text fixtures for dirty-repo and symlink boundaries.
- Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB041-fixtures.log`.
- Validation evidence:
  - `open fixtures/fixture_matrix.csv | length` returned `14`.
  - All expected fixture ids were listed from `fixtures/fixture_matrix.csv`.
  - `cargo metadata --format-version 1 --no-deps` succeeded for every buildable fixture manifest using `/nix/store/xlli1a8m35h5kwavjajnp6nl90xmjgcx-cargo-1.94.0/bin/cargo`.
  - No `Cargo.lock` files were produced under `fixtures/**`.
