---
id: 019f2145-30c3-7ea0-885b-acb63d28fbe7
slug: tasks/nu-plugin-codedb-rust-items
title: "Implement CodeDB static Rust item inventory"
type: task
status: completed
priority: high
tags: [nu_plugin, codedb, rust_static, parser]
---

## Overview

Implement package task `CDB022` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`: capture a static Rust item inventory without semantic overclaim.

## Scope

- Package path: `/home/flexnetos/Downloads/nu_plugin`
- CSV task: `CDB022`
- Depends on completed implementation rows: `CDB018`, `CDB021`
- Allowed source surface: `crates/codedb-rust-static/**`, represented on disk as `crates/codedb_rust_static/**`
- Raw validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB022-rust-items.log`

## Crate vs In-House Decision

- Production crates: `syn` for Rust syntax parsing; `sha2` for stable item IDs
- Dev-only crates: none
- In-house logic: mapping parsed syntax items into CodeDB row structs, deterministic sorting, conservative module path construction
- Rejected alternatives: regex-only Rust parsing, because it would violate the semantic-overclaim stop condition; rustc/rustdoc execution, because this task is static inventory only
- Packaging impact: adds parser/hash dependencies to `codedb-rust-static`; no build-script, proc-macro, or network execution is introduced

## Stop Conditions

- Do not claim semantic resolution, type checking, macro expansion, or compiler truth
- Do not execute build scripts or proc macros
- Do not parse Rust with regex-only logic

## Acceptance Criteria

- [x] Static item rows are parsed through `syn`
- [x] Rows include stable IDs, context ID, source path, module path, item kind, name, visibility, and capture confidence
- [x] Simple fixture captures modules/functions/structs/enums/traits deterministically
- [x] Unsupported semantic truth is not claimed
- [x] `/home/flexnetos/Downloads/nu_plugin/logs/CDB022-rust-items.log` records the validation commands and results

## Evidence

- Implemented static syntax item capture in `/home/flexnetos/Downloads/nu_plugin/crates/codedb_rust_static/src/lib.rs`
- Added `RustItemRow`, item kind/visibility/confidence enums, stable item IDs, conservative inline-module path collection, and `capture_rust_items()`
- Added `syn` and `sha2` dependencies in `/home/flexnetos/Downloads/nu_plugin/crates/codedb_rust_static/Cargo.toml`
- Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB022-rust-items.log`
- Passing commands:
  - `cargo fmt -p codedb-rust-static --check`
  - `cargo test -p codedb-rust-static simple_item_fixture_passes`
  - `cargo test -p codedb-rust-static`

## Next

Next CSV task by dependency order: `CDB023` (`Implement macro_rules static inventory`).
