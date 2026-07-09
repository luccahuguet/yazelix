---
id: 019f216a-a04f-7501-9d0c-0296b73367d0
slug: tasks/nu-plugin-codedb-doctor-checks
title: "Implement CodeDB doctor checks"
type: task
status: completed
priority: high
tags: [nu_plugin, codedb, doctor, integration]
---

## Overview

Implement package task `CDB031` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`: add non-interactive `codedb doctor` checks for Nu, Yazelix, Codex, meta, and envctl integration status.

## Scope

- Package path: `/home/flexnetos/Downloads/nu_plugin`
- CSV task: `CDB031`
- Depends on completed package tasks: `CDB029`, `CDB030`
- PRD sections: `14`, `16`
- Allowed source surface: `crates/codedb/**`
- Forbidden action: silent degraded states
- Raw validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB031-doctor.log`

## Readiness Gate

- [x] Selected `CDB031` from `execution/TASK_GRAPH.csv`
- [x] Read package readiness and stop gates
- [x] Read PRD sections `14` and `16`
- [x] Identified validation gate: doctor reports Nu/Yazelix/Codex status
- [x] Identified target surface and allowed files: `crates/codedb/**`
- [x] Confirmed no hidden prompts or registry mutations are needed

## Stop Conditions

- Do not silently hide missing tools or incompatible runtime state
- Do not mutate plugin registries, real `HOME`, or Yazelix config
- Do not assume host Nu and Yazelix runtime Nu are the same executable
- Preserve raw failure logs in `logs/CDB031-doctor.log`

## Acceptance Criteria

- [x] `codedb doctor --nu --format json` reports host Nu status clearly
- [x] `codedb doctor --yazelix --format json` reports Yazelix runtime Nu status clearly
- [x] `codedb doctor --codex --meta --envctl --format json` reports integration statuses clearly
- [x] Doctor rows include recommended actions for degraded states
- [x] `/home/flexnetos/Downloads/nu_plugin/logs/CDB031-doctor.log` records validation commands and results

## Evidence

- Added non-interactive `codedb doctor` handling in `/home/flexnetos/Downloads/nu_plugin/crates/codedb/src/main.rs`
- Implemented host Nu checks for path, version, plugin protocol compatibility, plugin binary path, registration status, and recommended registration command
- Implemented Yazelix runtime Nu checks through explicit environment variables without assuming host Nu and Yazelix Nu are the same executable
- Implemented Codex/meta/envctl integration status rows with degraded states and recommended actions when optional tools are absent
- Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB031-doctor.log`
- Passing commands:
  - `cargo fmt -p codedb --check`
  - `cargo build -p codedb`
  - `cargo test -p codedb`
  - `codedb doctor --nu --format json` produced 5 `doctor_checks` rows
  - `codedb doctor --yazelix --format json` produced 1 explicit degraded `doctor_checks` row when runtime Nu env vars were unset
  - `codedb doctor --codex --meta --envctl --format json` produced 4 integration `doctor_checks` rows
  - JSON outputs parsed through Nushell `open ... | where table == doctor_checks | length`

## Next

Next CSV task by dependency order: `CDB032` (`Implement bounded read-only MCP server`).
