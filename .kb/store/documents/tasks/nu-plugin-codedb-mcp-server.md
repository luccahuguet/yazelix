---
id: 019f216e-e6f6-7952-b93c-bcd3f51138dc
slug: tasks/nu-plugin-codedb-mcp-server
title: "Implement CodeDB bounded read-only MCP server"
type: task
status: completed
priority: high
tags: [nu_plugin, codedb, mcp, codex]
---

## Overview

Implement package task `CDB032` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`: add a bounded, read-only MCP server surface for Codex-facing CodeDB table access.

## Scope

- Package path: `/home/flexnetos/Downloads/nu_plugin`
- CSV task: `CDB032`
- Depends on completed package task: `CDB029`
- PRD sections: `13.3`, `15.4`, `16.1`
- Allowed source surface from CSV: `crates/codedb-mcp/**`; current package crate path is `/home/flexnetos/Downloads/nu_plugin/crates/codedb_mcp/**`
- Forbidden action: unbounded raw source tools
- Raw validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB032-mcp.log`

## Readiness Gate

- [x] Selected `CDB032` from `execution/TASK_GRAPH.csv`
- [x] Read package readiness and stop gates
- [x] Read PRD sections `13.3`, `15.4`, and `16.1`
- [x] Identified validation gate: MCP page/limit/source guard tests pass
- [x] Identified target surface and actual package files: `crates/codedb_mcp/**`
- [x] Confirmed raw source/blob/full-file tools must not be exposed

## Stop Conditions

- Do not add raw source, full-file dump, patch, git mutation, or unsafe build-capture tools
- Do not expose unbounded table dumps
- Do not execute build scripts or proc macros
- Preserve raw failure logs in `logs/CDB032-mcp.log`

## Acceptance Criteria

- [x] Allowed MCP tool registry matches the PRD read-only tools
- [x] Blocked tool names are rejected by default
- [x] Table page requests enforce row and byte limits
- [x] MCP responses do not expose raw source blobs or full file contents
- [x] `/home/flexnetos/Downloads/nu_plugin/logs/CDB032-mcp.log` records validation commands and results

## Evidence

- Replaced `/home/flexnetos/Downloads/nu_plugin/crates/codedb_mcp/src/lib.rs` with a bounded read-only MCP request/response layer
- Added the PRD-approved allowlist:
  - `codedb_schema`
  - `codedb_list_tables`
  - `codedb_get_table_page`
  - `codedb_get_capture_gaps`
  - `codedb_get_validation_errors`
  - `codedb_get_repo_summary`
  - `codedb_get_cargo_summary`
  - `codedb_get_rust_item_summary`
  - `codedb_get_macro_summary`
  - `codedb_get_build_script_summary`
  - `codedb_get_no_mutation_proof`
- Added explicit default blocks for raw source/blob, full file dump, unsafe build capture, source overwrite, patch apply, git mutation, and unbounded table dump tools
- Enforced default row limit `50`, maximum row limit `200`, and default max bytes `65536`
- Added tests for allowlist/denylist behavior, row-limit paging, excessive-limit rejection, byte-limit truncation, and source leak guard
- Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB032-mcp.log`
- Passing commands:
  - `cargo fmt -p codedb-mcp --check`
  - `cargo build -p codedb-mcp`
  - `cargo test -p codedb-mcp`

## Next

Next CSV task by dependency order: `CDB033` (`Implement unsafe build capture gate scaffold`).
