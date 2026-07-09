---
id: 019f21aa-a992-70b1-beeb-3732a47f7098
slug: tasks/nu-plugin-codedb-unsafe-capture-tests
title: "Add CodeDB unsafe capture tests"
type: task
status: completed
priority: high
tags: [codedb, nu_plugin, tests, unsafe_capture, CDB045]
---

## Source Task

`/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv` row CDB045.

- Phase: tests
- Depends on: CDB033; CDB034; CDB041
- Blocks: CDB046
- Target surface: tests
- Allowed files: `tests/**`
- Forbidden: ungated unsafe execution
- Primary artifact: test outputs
- Execution gate: unsafe capture gate tests pass
- Raw log: `logs/CDB045-unsafe-tests.log`
- PRD sections: 15.2, 19

## Acceptance Criteria

- [x] Unsafe capture test exists under `tests/**`.
- [x] Test proves the `codedb-build-capture` gate/refusal/raw-log crate tests pass.
- [x] Test proves the runner manifest reports unsafe capture default policy as `refuse_without_unsafe_flag`.
- [x] Test proves MCP dynamic execution is blocked in the runner proof row.
- [x] Test uses a temporary fixture copy and does not mutate source fixtures.
- [x] Validation records evidence in `logs/CDB045-unsafe-tests.log`.

## Notes

- Added `tests/test_unsafe_capture.nu`.
- The test runs `cargo test -p codedb-build-capture --quiet`, covering:
  refused capture without unsafe approval, approval scaffold behavior, approved
  fixture raw-log capture, and refusal when the approved fixture helper is called
  without the unsafe flag.
- The test also reads `codedb export runner_proof_manifest` over a temporary
  `fixtures/build_script` copy and asserts the `unsafe_capture_default` gate is
  satisfied with `default_policy = refuse_without_unsafe_flag` and
  `mcp_dynamic_execution = blocked`.

## Completion Evidence

- Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB045-unsafe-tests.log`.
- Command:
  `CODEDB_TEST_CARGO_DIR=/nix/store/xlli1a8m35h5kwavjajnp6nl90xmjgcx-cargo-1.94.0/bin nu tests/test_unsafe_capture.nu`.
- Evidence:
  - `build_capture_crate_tests` passed.
  - `runner_unsafe_gate` status was `satisfied`.
  - implementation markers include `UNSAFE_FLAG`, `capture_build_refuses_without_unsafe_flag`, `approved_fixture_capture_writes_raw_logs`, and `approved_fixture_capture_still_refuses_without_unsafe_flag`.
  - runner markers include `unsafe_capture_default` and `mcp_dynamic_execution = blocked`.
  - lane marker present: `# Test lane: default`.
  - contract marker present: `# Defends: dynamic build/proc-macro capture is gated and refuses without explicit unsafe approval.`
  - `find fixtures -name Cargo.lock` returned none after validation.
