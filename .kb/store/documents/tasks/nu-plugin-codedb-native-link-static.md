---
id: 019f2155-ec3f-7090-8a42-b1fcdd9313c6
slug: tasks/nu-plugin-codedb-native-link-static
title: "Implement CodeDB native linker static and gap rows"
type: task
status: completed
priority: high
tags: [nu_plugin, codedb, rust_static, native_link]
---

## Overview

Implement package task `CDB027` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`: convert statically visible build-script linker/native Cargo instructions into native/link rows and explicit gaps.

## Scope

- Package path: `/home/flexnetos/Downloads/nu_plugin`
- CSV task: `CDB027`
- Depends on completed implementation row: `CDB025`
- Allowed source surface: `crates/codedb-rust-static/**`, represented on disk as `crates/codedb_rust_static/**`
- Raw validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB027-native.log`

## Stop Conditions

- Do not execute build scripts
- Do not invoke native linkers or inspect host linker state dynamically
- Do not claim native library availability beyond static Cargo instruction rows

## Acceptance Criteria

- [x] `rustc-link-lib` instructions become native library rows
- [x] `rustc-link-search` instructions become link search path rows
- [x] `rustc-link-arg` instructions become link arg rows
- [x] Dynamic linker/native availability is represented as capture gaps
- [x] `/home/flexnetos/Downloads/nu_plugin/logs/CDB027-native.log` records the validation commands and results

## Evidence

- Extended `/home/flexnetos/Downloads/nu_plugin/crates/codedb_rust_static/src/lib.rs` with `NativeLinkInventory`, native library rows, link arg rows, link search path rows, and native/link capture gaps
- Implemented `capture_native_link_static()` as a projection from static build-script Cargo instruction rows
- Added gaps for linker tool, library availability, and link result truth
- Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB027-native.log`
- Passing commands:
  - `cargo fmt -p codedb-rust-static --check`
  - `cargo test -p codedb-rust-static native_link_fixture_emits_static_rows_and_gaps`
  - `cargo test -p codedb-rust-static`

## Next

Next CSV task by dependency order: `CDB028` (`Implement no-mutation proof`).
