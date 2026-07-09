---
id: 019f2177-8327-73c2-9c51-255af800d44c
slug: tasks/nu-plugin-codedb-build-capture-logs
title: "Implement CodeDB optional build and proc-macro raw log capture"
type: task
status: completed
priority: high
tags: [nu_plugin, codedb, unsafe, build_capture]
---

## Overview

Implement package task `CDB034` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`: add optional approved-fixture build/proc-macro raw log capture behind the explicit unsafe execution gate.

## Scope

- Package path: `/home/flexnetos/Downloads/nu_plugin`
- CSV task: `CDB034`
- Depends on completed package task: `CDB033`
- PRD sections: `10.7`, `10.8`, `15.2`
- Allowed source surface from CSV: `crates/codedb-build-capture/**`; current package implementation path is `/home/flexnetos/Downloads/nu_plugin/crates/codedb_build_capture/**`
- Forbidden action: MCP dynamic execution
- Raw validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB034-build-capture.log`

## Readiness Gate

- [x] Selected `CDB034` from `execution/TASK_GRAPH.csv`
- [x] Read package readiness and stop gates
- [x] Read PRD sections `10.7`, `10.8`, and `15.2`
- [x] Identified validation gate: approved fixture captures logs or gaps
- [x] Identified target surface: `codedb_build_capture`
- [x] Confirmed dynamic execution must stay out of MCP

## Stop Conditions

- Do not execute build scripts or proc macros without `--unsafe-execute-build`
- Do not expose dynamic capture through MCP
- Do not run broad real-repo builds for validation; use an approved fixture
- Preserve raw stdout/stderr logs under `logs/CDB034-build-capture.log`

## Acceptance Criteria

- [x] Dynamic capture refuses without explicit unsafe approval
- [x] Approved fixture capture runs only through the unsafe fixture path
- [x] Raw stdout/stderr logs are preserved at the requested raw log path
- [x] Output rows include `unsafe_execution_approval`, `build_script_runs`, `proc_macro_invocations`, `raw_log_paths`, `capture_gaps`, and `validation_errors`
- [x] `/home/flexnetos/Downloads/nu_plugin/logs/CDB034-build-capture.log` records validation commands and results

## Evidence

- Extended `/home/flexnetos/Downloads/nu_plugin/crates/codedb_build_capture/src/lib.rs`
- Added `capture_approved_fixture_build` for explicit unsafe approved fixture execution
- Preserved CDB033 default refusal behavior when `unsafe_execute_build` is false
- The approved fixture path runs `cargo check --message-format=json`, writes raw stdout/stderr/status to the requested raw log path, and returns structured rows:
  - `unsafe_execution_approval`
  - `build_script_runs`
  - `proc_macro_invocations`
  - `raw_log_paths`
  - `capture_gaps`
  - `validation_errors`
- Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB034-build-capture.log`
- Passing commands:
  - `cargo fmt -p codedb-build-capture --check`
  - `cargo build -p codedb-build-capture`
  - `cargo test -p codedb-build-capture`
- Tests prove:
  - default dynamic capture refusal without unsafe approval
  - CDB033 approval scaffold remains non-executing
  - approved fixture capture writes a raw log containing the build script warning
  - approved fixture path still refuses without unsafe approval

## Next

Next CSV task by dependency order: `CDB035` (`Implement envctl export contract`) or `CDB036` (`Implement meta repo selection inputs`), both unblocked by completed `CDB029`.
