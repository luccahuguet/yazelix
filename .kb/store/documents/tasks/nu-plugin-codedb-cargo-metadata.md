---
id: 019f213b-a33a-75b1-b468-21760ab2e502
slug: tasks/nu-plugin-codedb-cargo-metadata
title: "Implement CodeDB cargo metadata capture"
type: task
status: completed
priority: high
tags: [nu_plugin, codedb, cargo, metadata]
---

## Overview

Implement package task `CDB019` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`: capture Cargo workspace/package/target/dependency/resolve/feature facts from `cargo metadata --format-version 1`.

## Scope

- Package path: `/home/flexnetos/Downloads/nu_plugin`
- CSV task: `CDB019`
- Depends on completed implementation rows: `CDB014`, `CDB015`
- Allowed source surface: `crates/codedb-cargo/**` in the package, represented on disk as `crates/codedb_cargo/**`
- Raw validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB019-cargo.log`

## Stop Conditions

- Do not parse Cargo manifests with regex-only logic
- Do not mutate source repositories during capture
- Do not require network access for the fixture validation

## Acceptance Criteria

- [x] `codedb-cargo` invokes `cargo metadata --format-version 1` for structured Cargo reality
- [x] Captured rows cover at least workspace roots, packages, targets, dependencies, resolve nodes, and features
- [x] Fixture validation proves a simple workspace capture is deterministic
- [x] `/home/flexnetos/Downloads/nu_plugin/logs/CDB019-cargo.log` records the validation commands and results
- [x] Task evidence is committed through GitKB after validation

## Evidence

- Implemented `capture_cargo_metadata()` in `/home/flexnetos/Downloads/nu_plugin/crates/codedb_cargo/src/lib.rs`
- Added structured row models for workspace, packages, targets, dependencies, resolve nodes, and features
- Added `serde` and `serde_json` dependencies in `/home/flexnetos/Downloads/nu_plugin/crates/codedb_cargo/Cargo.toml`
- Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB019-cargo.log`
- Passing commands:
  - `cargo fmt -p codedb-cargo --check`
  - `cargo test -p codedb-cargo cargo_metadata_fixture_capture_is_stable`
  - `cargo test -p codedb-cargo`

## Next

Next CSV task by dependency order: `CDB020` (`Implement Cargo source provenance capture`).
