---
id: 019f2173-94d2-7222-b9ec-fd10e7ccc53c
slug: tasks/nu-plugin-codedb-unsafe-build-gate
title: "Implement CodeDB unsafe build capture gate scaffold"
type: task
status: completed
priority: high
tags: [nu_plugin, codedb, unsafe, build_capture]
---

## Overview

Implement package task `CDB033` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`: add the unsafe build capture gate scaffold that refuses dynamic build/proc-macro capture unless an explicit unsafe approval flag is present.

## Scope

- Package path: `/home/flexnetos/Downloads/nu_plugin`
- CSV task: `CDB033`
- Depends on completed package tasks: `CDB025`, `CDB032`
- PRD sections: `15.2`, `10.8`
- Allowed source surface from CSV: `crates/codedb-build-capture/**`; current package uses underscore crate paths, so the implementation path is `/home/flexnetos/Downloads/nu_plugin/crates/codedb_build_capture/**`
- Forbidden action: default execution
- Raw validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB033-unsafe-gate.log`

## Readiness Gate

- [x] Selected `CDB033` from `execution/TASK_GRAPH.csv`
- [x] Read package readiness and stop gates
- [x] Read PRD section `15.2`
- [x] Identified validation gate: refuses without unsafe flag
- [x] Identified target surface: unsafe capture scaffold
- [x] Confirmed dynamic build/proc-macro execution must not run by default

## Stop Conditions

- Do not execute build scripts or proc macros by default
- Do not expose dynamic capture through MCP
- Do not claim raw log capture before an explicit unsafe approval path exists
- Preserve raw failure logs in `logs/CDB033-unsafe-gate.log`

## Acceptance Criteria

- [x] Capture build API refuses when `unsafe_execute_build` is false
- [x] Approval rows identify the unsafe flag, approver, repo path, and raw log path
- [x] Refusal rows identify the missing unsafe approval and required next action
- [x] Tests prove default refusal and explicit approval scaffold behavior
- [x] `/home/flexnetos/Downloads/nu_plugin/logs/CDB033-unsafe-gate.log` records validation commands and results

## Evidence

- Added workspace crate `/home/flexnetos/Downloads/nu_plugin/crates/codedb_build_capture`
- Registered package `codedb-build-capture` in `/home/flexnetos/Downloads/nu_plugin/Cargo.toml`
- Implemented `capture_build(BuildCaptureRequest)` with:
  - default refusal when `unsafe_execute_build` is false
  - explicit `--unsafe-execute-build` approval rows
  - validation error and capture gap rows for missing unsafe approval
  - scaffold-only approval path that records CDB034 as the dynamic execution owner
  - raw log path rows that do not write execution logs in CDB033
- Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB033-unsafe-gate.log`
- Passing commands:
  - `cargo metadata --format-version 1 --no-deps` includes `codedb-build-capture`
  - `cargo fmt -p codedb-build-capture --check`
  - `cargo build -p codedb-build-capture`
  - `cargo test -p codedb-build-capture`

## Next

Next CSV task by dependency order: `CDB034` (`Implement optional build/proc-macro raw log capture`).
