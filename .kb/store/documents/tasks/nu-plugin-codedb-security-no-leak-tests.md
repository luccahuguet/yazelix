---
id: 019f21a6-1945-7930-bca8-91ac33005d24
slug: tasks/nu-plugin-codedb-security-no-leak-tests
title: "Add CodeDB security no leak tests"
type: task
status: completed
priority: high
tags: [codedb, nu_plugin, tests, security, no_leak, CDB043]
---

## Source Task

`/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv` row CDB043.

- Phase: tests
- Depends on: CDB041; CDB032
- Blocks: CDB046
- Target surface: tests
- Allowed files: `tests/**`
- Forbidden: raw source MCP leak
- Primary artifact: test outputs
- Execution gate: MCP/source secret tests pass
- Raw log: `logs/CDB043-security-tests.log`
- PRD sections: 15, 19

## Acceptance Criteria

- [x] Security/no-leak test exists under `tests/**`.
- [x] Test runs MCP safety coverage for blocked raw/mutating/unsafe tools and repo-summary no-leak behavior.
- [x] Test proves default CLI scan output over the secret-looking fixture does not emit raw secret-looking values.
- [x] Test proves default CLI table/export outputs over the secret-looking fixture do not emit raw secret-looking values.
- [x] Test uses a temporary fixture copy and does not mutate source fixtures.
- [x] Validation records evidence in `logs/CDB043-security-tests.log`.

## Notes

- Added `tests/test_security_no_leak.nu`.
- The test runs `cargo test -p codedb-mcp --quiet` so the existing MCP blocked-tool
  and repo-summary leak guard tests remain part of the CSV task proof.
- The test checks the raw placeholder values from `fixtures/secret_like/src/lib.rs`
  are absent from `codedb scan`, `codedb export rust_items`,
  `codedb export codedb_table_checksums`, and `codedb export envctl` outputs.
- Like CDB042, the test copies the fixture to a temporary directory before
  invoking CodeDB because Cargo metadata may create a `Cargo.lock` in the scanned
  repo. The source fixture must remain clean.

## Completion Evidence

- Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB043-security-tests.log`.
- Command:
  `CODEDB_TEST_CARGO_DIR=/nix/store/xlli1a8m35h5kwavjajnp6nl90xmjgcx-cargo-1.94.0/bin nu tests/test_security_no_leak.nu`.
- Evidence:
  - `mcp_security_tests` passed.
  - `scan_summary`, `rust_items`, `table_checksums`, and `envctl_export` all reported `raw_secret_values = absent`.
  - MCP blocked raw-source policy markers include `raw_source_blob_read`, `full_file_dump`, and `unsafe_build_capture`.
  - lane marker present: `# Test lane: default`.
  - contract marker present: `# Defends: CodeDB default CLI and MCP surfaces do not emit raw secret-looking source values.`
  - `find fixtures -name Cargo.lock` returned none after validation.
