---
id: 019f214b-c33d-72f3-aa32-ab55bc81534c
slug: tasks/nu-plugin-codedb-proc-macro-static
title: "Implement CodeDB proc-macro static detection and gaps"
type: task
status: completed
priority: high
tags: [nu_plugin, codedb, rust_static, proc_macro]
---

## Overview

Implement package task `CDB024` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`: statically detect proc-macro crates and proc-macro invocation sites, while emitting capture gaps for all dynamic execution facts.

## Scope

- Package path: `/home/flexnetos/Downloads/nu_plugin`
- CSV task: `CDB024`
- Depends on completed implementation row: `CDB022`
- Allowed source surface: `crates/codedb-rust-static/**`, represented on disk as `crates/codedb_rust_static/**`
- Raw validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB024-proc-macro.log`

## Crate vs In-House Decision

- Production crates: continue using `syn` for Rust syntax parsing and `sha2` for stable row IDs
- Dev-only crates: none
- In-house logic: detect static proc-macro crate exports, attribute/derive/function-like proc-macro usage shapes, and dynamic capture gaps
- Rejected alternatives: executing proc macros or build scripts, because CDB024 explicitly forbids proc-macro execution
- Packaging impact: no new dependencies beyond the CDB022 parser/hash stack

## Stop Conditions

- Do not execute proc macros
- Do not claim proc-macro output token streams, panics, env, or file access without unsafe execution
- Do not parse proc-macro syntax with regex-only logic

## Acceptance Criteria

- [x] Proc-macro crate export rows are detected statically
- [x] Proc-macro invocation rows are detected statically for attribute, derive, and function-like shapes
- [x] Capture gaps are emitted for dynamic proc-macro facts
- [x] Fixture validation proves static rows and gaps without executing proc macros
- [x] `/home/flexnetos/Downloads/nu_plugin/logs/CDB024-proc-macro.log` records the validation commands and results

## Evidence

- Extended `/home/flexnetos/Downloads/nu_plugin/crates/codedb_rust_static/src/lib.rs` with `ProcMacroInventory`, proc-macro crate export rows, invocation rows, and dynamic capture gap rows
- Implemented `capture_proc_macro_static()` using `syn` static parsing only
- Added gaps for output token stream, panic, environment, and file-access facts
- Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB024-proc-macro.log`
- Passing commands:
  - `cargo fmt -p codedb-rust-static --check`
  - `cargo test -p codedb-rust-static proc_macro_fixture_emits_static_rows_and_gaps`
  - `cargo test -p codedb-rust-static`

## Next

Next CSV task by dependency order: `CDB025` (`Implement build.rs static detection and gaps`).
