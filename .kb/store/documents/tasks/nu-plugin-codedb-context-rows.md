---
id: 019f2142-7151-7802-8684-75649cf7aa11
slug: tasks/nu-plugin-codedb-context-rows
title: "Implement CodeDB cfg feature target toolchain context"
type: task
status: completed
priority: high
tags: [nu_plugin, codedb, context, cargo]
---

## Overview

Implement package task `CDB021` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`: produce deterministic, keyed context rows for cfg/feature/target/toolchain facts.

## Scope

- Package path: `/home/flexnetos/Downloads/nu_plugin`
- CSV task: `CDB021`
- Depends on completed implementation row: `CDB019`
- Allowed source surface: `crates/codedb-cargo/**;crates/codedb-core/**`, represented on disk as `crates/codedb_cargo/**;crates/codedb_core/**`
- Raw validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB021-context.log`

## Stop Conditions

- Do not emit unkeyed context rows
- Do not omit feature/cfg/profile/edition from the context identity key
- Do not make context row ordering nondeterministic

## Acceptance Criteria

- [x] `codedb_contexts` rows include a stable context identity key
- [x] Feature sets are sorted and hashed deterministically
- [x] Target cfg values are sorted and hashed deterministically
- [x] Toolchain, cargo, rustc, host triple, target triple, profile, and edition facts are represented
- [x] `/home/flexnetos/Downloads/nu_plugin/logs/CDB021-context.log` records the validation commands and results

## Evidence

- Extended `/home/flexnetos/Downloads/nu_plugin/crates/codedb_cargo/src/lib.rs` with context row models for `codedb_contexts`, toolchain, cargo/rustc versions, host/target triples, target cfgs, feature sets, and profiles
- Implemented `build_context_rows()` with stable SHA-256 context/toolchain/feature/cfg identity hashes
- Added `sha2 = "0.10"` to `/home/flexnetos/Downloads/nu_plugin/crates/codedb_cargo/Cargo.toml`
- Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB021-context.log`
- Passing commands:
  - `cargo fmt -p codedb-cargo --check`
  - `cargo test -p codedb-cargo context_rows_are_keyed_and_deterministic`
  - `cargo test -p codedb-cargo`

## Next

Next CSV task by dependency order: `CDB022` (`Implement static Rust item inventory`).
