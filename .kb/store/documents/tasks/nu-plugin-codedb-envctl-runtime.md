---
id: 019f21eb-16e5-70d0-8958-1a5f59ab4cef
slug: tasks/nu-plugin-codedb-envctl-runtime
title: "Add CodeDB envctl runtime integration rows"
type: task
status: completed
priority: medium
---

## Source

- CSV task: `CDB063`
- Title: Add envctl table rows for CodeDB runtime integration
- Depends on: `CDB035`, `CDB055`
- Allowed files: `docs/ENVCTL_EXPORT_CONTRACT.md`, `crates/codedb/**`
- Evidence log: `logs/CDB063-envctl-runtime.log`

## Scope

Add explicit runtime integration rows to the CodeDB envctl export contract so
envctl can consume CodeDB as the high-fidelity datatable and blob/Rust/crate fact
store without reading redb internals or deriving semantic facts independently.

## Acceptance

- [x] `codedb export envctl` emits `codedb_runtime_integration`
- [x] `codedb_table_checksums` includes `codedb_runtime_integration`
- [x] Documentation states that envctl consumes exports only
- [x] Validation evidence is recorded in `logs/CDB063-envctl-runtime.log`

## Evidence

- `cargo fmt --check` passed
- `cargo test -p codedb` passed
- `codedb export envctl --repo-path <temp fixture> --format json` emitted 4 `codedb_runtime_integration` rows
- The same export emitted 1 `codedb_table_checksums` row with `source_table = codedb_runtime_integration`
- `find fixtures -name Cargo.lock -print | sort` produced no output

## Result

`envctl integration complete`: envctl now has an explicit runtime integration
table that keeps CodeDB authoritative for datatable/blob/Rust/crate semantics
while limiting envctl to export consumption and requested file materialization.
