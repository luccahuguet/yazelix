---
id: 019f2152-5c0a-70c2-9cf1-f06e0acf13a0
slug: tasks/nu-plugin-codedb-static-include-edges
title: "Implement CodeDB static include path edge detection"
type: task
status: completed
priority: high
tags: [nu_plugin, codedb, rust_static, include_edges]
---

## Overview

Implement package task `CDB026` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`: detect static include/path edges without claiming dynamic file tracing.

## Scope

- Package path: `/home/flexnetos/Downloads/nu_plugin`
- CSV task: `CDB026`
- Depends on completed implementation row: `CDB022`
- Allowed source surface: `crates/codedb-rust-static/**`, represented on disk as `crates/codedb_rust_static/**`
- Raw validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB026-include.log`

## Crate vs In-House Decision

- Production crates: continue using `syn` for Rust syntax parsing and `sha2` for stable row IDs
- Dev-only crates: none
- In-house logic: detect literal `include!`, `include_str!`, `include_bytes!`, and `#[path = "..."]` edges from parsed syntax
- Rejected alternatives: runtime/dynamic file tracing, because CDB026 explicitly forbids dynamic file tracing claims
- Packaging impact: no new dependencies beyond the CDB022 parser/hash stack

## Stop Conditions

- Do not claim dynamic file tracing
- Do not resolve non-literal or computed include paths
- Do not execute code or macros

## Acceptance Criteria

- [x] Static include macro edges are captured for literal paths
- [x] Static `#[path = "..."]` module edges are captured
- [x] Nonliteral/computed paths are represented as unsupported static gaps
- [x] Fixture validation proves include/path rows are deterministic
- [x] `/home/flexnetos/Downloads/nu_plugin/logs/CDB026-include.log` records the validation commands and results

## Evidence

- Extended `/home/flexnetos/Downloads/nu_plugin/crates/codedb_rust_static/src/lib.rs` with `StaticIncludeInventory`, static include edge rows, and include gap rows
- Implemented `capture_static_include_edges()` using `syn` static parsing only
- Captures literal `include!`, `include_str!`, `include_bytes!`, and `#[path = "..."]` edges
- Emits gaps for nonliteral/computed include targets
- Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB026-include.log`
- Passing commands:
  - `cargo fmt -p codedb-rust-static --check`
  - `cargo test -p codedb-rust-static include_fixture_passes`
  - `cargo test -p codedb-rust-static`

## Next

Next CSV task by dependency order: `CDB027` (`Implement native/linker static/gap rows`).
