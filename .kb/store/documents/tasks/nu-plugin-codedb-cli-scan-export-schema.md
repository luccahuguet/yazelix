---
id: 019f215b-dda2-7641-8973-c5342c098ff9
slug: tasks/nu-plugin-codedb-cli-scan-export-schema
title: "Implement CodeDB CLI scan export schema"
type: task
status: completed
priority: high
tags: [nu_plugin, codedb, cli]
---

## Overview

Implement package task `CDB029` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`: expose non-interactive `codedb` CLI scan/export/schema commands with JSON, NUON, and CSV output validation.

## Scope

- Package path: `/home/flexnetos/Downloads/nu_plugin`
- CSV task: `CDB029`
- Depends on completed implementation rows: `CDB015`, `CDB017`, `CDB019`, `CDB022`
- Allowed source surface: `crates/codedb/**`
- Raw validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB029-cli.log`

## Stop Conditions

- Do not introduce interactive or hidden prompts
- Do not mutate the scanned repository
- Do not claim persisted redb scan state before store-backed scan wiring exists

## Acceptance Criteria

- [x] `codedb scan <repo_path> --format json|nuon|csv` returns machine-readable scan summary rows
- [x] `codedb export <table> --format json|nuon|csv` returns bounded table output for available tables
- [x] `codedb schema --format json|nuon|csv` returns schema rows
- [x] Unsupported commands/formats fail fast with clear errors
- [x] `/home/flexnetos/Downloads/nu_plugin/logs/CDB029-cli.log` records validation commands and results

## Evidence

- Replaced `/home/flexnetos/Downloads/nu_plugin/crates/codedb/src/main.rs` with a noninteractive CLI for `scan`, `export`, `schema`, `tables`, `gaps`, and `validation-errors`
- Added CLI crate dependencies on `codedb-cargo`, `codedb-rust-static`, and `serde_json`
- `scan` summarizes filesystem, static Rust item, Cargo package/dependency, and Cargo source rows
- `export` supports schema/tables/gaps/validation errors/filesystem/rust items/Cargo packages/dependencies/sources
- Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB029-cli.log`
- Passing commands:
  - `cargo fmt -p codedb --check`
  - `cargo build -p codedb`
  - `cargo run -q -p codedb -- schema --format json`
  - `cargo run -q -p codedb -- scan <fixture> --format nuon`
  - `cargo run -q -p codedb -- export rust_items --repo <fixture> --format csv`
  - JSON, NUON, and CSV parse validations through Nushell

## Next

Next CSV task by dependency order: `CDB030` (`Implement Nushell plugin commands`).
