---
id: 019f2148-b12f-7b00-9e24-4c8054e9f5da
slug: tasks/nu-plugin-codedb-macro-rules
title: "Implement CodeDB macro_rules static inventory"
type: task
status: completed
priority: high
tags: [nu_plugin, codedb, rust_static, macros]
---

## Overview

Implement package task `CDB023` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`: capture `macro_rules!` definitions and macro invocations statically, while recording gaps for expansion and hygiene truth.

## Scope

- Package path: `/home/flexnetos/Downloads/nu_plugin`
- CSV task: `CDB023`
- Depends on completed implementation row: `CDB022`
- Allowed source surface: `crates/codedb-rust-static/**`, represented on disk as `crates/codedb_rust_static/**`
- Raw validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB023-macros.log`

## Crate vs In-House Decision

- Production crates: continue using `syn` for Rust syntax parsing; continue using `sha2` for stable row IDs
- Dev-only crates: none
- In-house logic: classify `macro_rules!` definitions, macro invocations, matcher/transcriber token summaries, and explicit expansion/hygiene gaps
- Rejected alternatives: regex-only macro scanning, because it is brittle; compiler macro expansion, because CDB023 is static-only and the task forbids claiming full hygiene truth
- Packaging impact: no new dependencies beyond the CDB022 parser/hash stack

## Stop Conditions

- Do not claim full macro expansion or hygiene truth
- Do not execute proc macros or build scripts
- Do not parse macros with regex-only logic

## Acceptance Criteria

- [x] `macro_rules!` definitions are captured as deterministic rows
- [x] Macro invocations are captured as deterministic rows
- [x] Macro matcher/transcriber summaries are represented for static inspection
- [x] Missing expansion/hygiene truth is represented as a capture gap
- [x] `/home/flexnetos/Downloads/nu_plugin/logs/CDB023-macros.log` records the validation commands and results

## Evidence

- Extended `/home/flexnetos/Downloads/nu_plugin/crates/codedb_rust_static/src/lib.rs` with `MacroInventory`, macro definition rows, macro invocation rows, and macro capture gap rows
- Implemented `capture_rust_macros()` using `syn` static parsing
- Added explicit expansion and hygiene gaps for `macro_rules!` definitions
- Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB023-macros.log`
- Passing commands:
  - `cargo fmt -p codedb-rust-static --check`
  - `cargo test -p codedb-rust-static macro_fixture_passes_with_gaps`
  - `cargo test -p codedb-rust-static`

## Next

Next CSV task by dependency order: `CDB024` (`Implement proc-macro static detection and gaps`).
