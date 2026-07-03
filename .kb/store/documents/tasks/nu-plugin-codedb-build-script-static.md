---
id: 019f214f-d575-76c3-9469-2d32e89e98fa
slug: tasks/nu-plugin-codedb-build-script-static
title: "Implement CodeDB build.rs static detection and gaps"
type: task
status: completed
priority: high
tags: [nu_plugin, codedb, rust_static, build_script]
---

## Overview

Implement package task `CDB025` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`: statically detect `build.rs`, observable Cargo instruction print sites, and dynamic build-script capture gaps.

## Scope

- Package path: `/home/flexnetos/Downloads/nu_plugin`
- CSV task: `CDB025`
- Depends on completed implementation row: `CDB022`
- Allowed source surface: `crates/codedb-rust-static/**`, represented on disk as `crates/codedb_rust_static/**`
- Raw validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB025-build-static.log`

## Crate vs In-House Decision

- Production crates: continue using `syn` for Rust syntax parsing and `sha2` for stable row IDs
- Dev-only crates: none
- In-house logic: classify build script files, statically inspect `println!` / `eprintln!` Cargo instruction string literals, and emit dynamic gaps
- Rejected alternatives: executing `build.rs`, because CDB025 explicitly forbids build-script execution
- Packaging impact: no new dependencies beyond the CDB022 parser/hash stack

## Stop Conditions

- Do not execute build scripts
- Do not claim runtime stdout/stderr/env/OUT_DIR artifacts without unsafe execution
- Do not parse build scripts with regex-only logic

## Acceptance Criteria

- [x] `build.rs` files are detected statically
- [x] Static Cargo instruction print sites are represented when string literals expose them
- [x] Capture gaps are emitted for dynamic build-script facts
- [x] Fixture validation proves static rows and gaps without executing `build.rs`
- [x] `/home/flexnetos/Downloads/nu_plugin/logs/CDB025-build-static.log` records the validation commands and results

## Evidence

- Extended `/home/flexnetos/Downloads/nu_plugin/crates/codedb_rust_static/src/lib.rs` with `BuildScriptInventory`, build script rows, static Cargo instruction rows, and dynamic capture gap rows
- Implemented `capture_build_script_static()` using `syn` static parsing only
- Added gaps for execution, environment, stdout, stderr, and OUT_DIR artifact facts
- Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB025-build-static.log`
- Passing commands:
  - `cargo fmt -p codedb-rust-static --check`
  - `cargo test -p codedb-rust-static build_script_fixture_emits_static_rows_and_gaps`
  - `cargo test -p codedb-rust-static`

## Next

Next CSV task by dependency order: `CDB026` (`Implement static include/path edge detection`).
