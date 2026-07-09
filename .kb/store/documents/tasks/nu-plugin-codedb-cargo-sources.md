---
id: 019f2140-0023-77d2-94ab-bc481199fa2d
slug: tasks/nu-plugin-codedb-cargo-sources
title: "Implement CodeDB Cargo source provenance capture"
type: task
status: completed
priority: high
tags: [nu_plugin, codedb, cargo, provenance]
---

## Overview

Implement package task `CDB020` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`: capture Cargo source provenance rows for path, registry, and git-observable package/dependency facts.

## Scope

- Package path: `/home/flexnetos/Downloads/nu_plugin`
- CSV task: `CDB020`
- Depends on completed implementation row: `CDB019`
- Allowed source surface: `crates/codedb-cargo/**` in the package, represented on disk as `crates/codedb_cargo/**`
- Raw validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB020-cargo-sources.log`

## Stop Conditions

- Do not perform network mutation or fetching
- Do not parse Cargo manifests with regex-only logic
- Do not leave provenance source strings unclassified when Cargo exposes them

## Acceptance Criteria

- [x] Source provenance rows are derived from structured Cargo metadata capture
- [x] Registry source facts are captured and classified
- [x] Git source facts are captured and classified
- [x] Path source facts are captured and classified
- [x] `/home/flexnetos/Downloads/nu_plugin/logs/CDB020-cargo-sources.log` records the validation commands and results

## Evidence

- Extended `/home/flexnetos/Downloads/nu_plugin/crates/codedb_cargo/src/lib.rs` with `CargoSourceRow`, `CargoSourceKind`, and `CargoSourceObservation`
- Added source rows to `CargoMetadataCapture` derived from Cargo metadata package/dependency `source` fields
- Implemented `classify_cargo_source()` for path, registry, git, and unknown source facts
- Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB020-cargo-sources.log`
- Passing commands:
  - `cargo fmt -p codedb-cargo --check`
  - `cargo test -p codedb-cargo cargo_source_classifier_covers_registry_git_and_path`
  - `cargo test -p codedb-cargo cargo_metadata_fixture_capture_is_stable`
  - `cargo test -p codedb-cargo`

## Next

Next CSV task by dependency order: `CDB021` (`Implement cfg/feature/target/toolchain context`).
