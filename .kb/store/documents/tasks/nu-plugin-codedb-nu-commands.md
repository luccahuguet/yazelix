---
id: 019f2163-378f-7cf1-945e-1e55b3a62032
slug: tasks/nu-plugin-codedb-nu-commands
title: "Implement CodeDB Nushell plugin commands"
type: task
status: completed
priority: high
tags: [nu_plugin, codedb, nushell, commands]
---

## Overview

Implement package task `CDB030` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`: expose the `nu_plugin_codedb` Nushell command surface for read-only scan/export/status commands.

## Scope

- Package path: `/home/flexnetos/Downloads/nu_plugin`
- CSV task: `CDB030`
- Depends on completed package task: `CDB029`
- PRD sections: `13.1`, `14`
- Allowed source surface: `crates/nu_plugin_codedb/**`
- Forbidden action: hardcoding one Nu runtime
- Raw validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB030-nu-plugin.log`

## Readiness Gate

- [x] Selected `CDB030` from `execution/TASK_GRAPH.csv`
- [x] Read package readiness and stop gates
- [x] Read PRD sections `13.1` and `14`
- [x] Identified validation gate: Nu command smoke passes
- [x] Identified target surface and allowed files: `crates/nu_plugin_codedb/**`
- [x] Confirmed no raw secret path is needed for this task
- [x] Confirmed the plugin must not assume one Nu registry/runtime

## Stop Conditions

- Do not hardcode one Nu runtime or one plugin registry
- Do not mutate real `HOME`, real Nushell config, or the user plugin registry
- Do not execute build scripts or proc macros
- Do not expose unbounded raw source output by default
- Preserve raw failure logs in `logs/CDB030-nu-plugin.log`

## Acceptance Criteria

- [x] `nu_plugin_codedb` builds
- [x] `codedb scan <repo_path>` returns table-shaped rows through transient `nu --plugins`
- [x] Read-only table commands return table-shaped rows through transient `nu --plugins`
- [x] `codedb export <table>` returns table-shaped rows through transient `nu --plugins`
- [x] Unsupported plugin inputs fail fast with clear Nushell errors
- [x] `/home/flexnetos/Downloads/nu_plugin/logs/CDB030-nu-plugin.log` records validation commands and results

## Evidence

- Expanded `/home/flexnetos/Downloads/nu_plugin/crates/nu_plugin_codedb/src/main.rs` from status-only commands into the CDB030 read-only Nu command surface:
  - `codedb scan`
  - `codedb fs entries`
  - `codedb source files`
  - `codedb cargo packages`
  - `codedb cargo deps`
  - `codedb cargo sources`
  - `codedb rust items`
  - `codedb rust macros`
  - `codedb rust cfg`
  - `codedb build scripts`
  - `codedb export`
  - `codedb tables`, `codedb gaps`, `codedb validation errors`, `codedb schema`, `codedb doctor`
- Added plugin crate dependencies on `codedb-cargo` and `codedb-rust-static`
- Kept plugin execution transient for validation; no real HOME, plugin registry, or Yazelix Nushell config was mutated
- Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB030-nu-plugin.log`
- Passing commands:
  - `cargo fmt -p nu_plugin_codedb --check`
  - `cargo build -p nu_plugin_codedb`
  - `cargo test -p nu_plugin_codedb`
  - transient `nu --plugins ... -c "codedb tables | length"` returned `8`
  - transient `nu --plugins ... -c "codedb scan /home/flexnetos/Downloads/nu_plugin/crates/codedb_fixtures | length"` returned `5`
  - transient `nu --plugins ... -c "codedb fs entries --repo /home/flexnetos/Downloads/nu_plugin/crates/codedb_fixtures --limit 3 | length"` returned `3`
  - transient `nu --plugins ... -c "codedb rust items --repo /home/flexnetos/Downloads/nu_plugin/crates/codedb_fixtures --limit 5 | length"` returned `1`
  - transient `nu --plugins ... -c "codedb export rust_items --repo /home/flexnetos/Downloads/nu_plugin/crates/codedb_fixtures --limit 5 | length"` returned `1`
  - unsupported export table rejected with a clear Nushell error

## Next

Next CSV task by dependency order: `CDB031` (`Implement doctor checks`).
