---
id: 019f2199-e13c-7032-b528-9020b4c7c639
slug: tasks/nu-plugin-codedb-rust-workspace-skeleton
title: "Create CodeDB Rust workspace skeleton"
type: task
status: completed
priority: high
tags: [codedb, nu_plugin, rust, workspace, CDB013]
---

## Source Task

`/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv` row CDB013.

- Phase: code
- Depends on: CDB006; CDB068
- Blocks: CDB014; CDB041
- Target surface: workspace
- Allowed files: `Cargo.toml`; `crates/*`
- Forbidden: source overwrite in existing repos
- Primary artifact: `Cargo.toml`
- Execution gate: cargo metadata succeeds
- Raw log: `logs/CDB013-workspace.log`
- PRD sections: 9

## Acceptance Criteria

- [x] Package root `Cargo.toml` defines a Rust workspace.
- [x] Workspace members cover the CodeDB CLI, core, redb store, cargo capture, static Rust capture, MCP, fixtures, build capture, and Nushell plugin crates.
- [x] Each workspace member has a `Cargo.toml`.
- [x] `cargo metadata --format-version 1 --no-deps` succeeds.
- [x] Validation records evidence in `logs/CDB013-workspace.log`.

## Notes

- This task is being closed after later crate slices already created the workspace. The verification is current-state based and does not overwrite existing source.

## Completion Evidence

Implemented in the source package under `/home/flexnetos/Downloads/nu_plugin`:

- Root `Cargo.toml` declares a workspace with resolver `3`.
- Workspace members:
  - `crates/codedb`
  - `crates/codedb_build_capture`
  - `crates/codedb_cargo`
  - `crates/codedb_core`
  - `crates/codedb_fixtures`
  - `crates/codedb_mcp`
  - `crates/codedb_rust_static`
  - `crates/codedb_store_redb`
  - `crates/nu_plugin_codedb`

Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB013-workspace.log`.

Passing gates recorded there:

- Root workspace manifest printed.
- All nine crate manifests found.
- `cargo metadata --format-version 1 --no-deps` parsed by Nushell and returned 9 package names matching the expected workspace members.
- Representative workspace crates built: `codedb`, `nu_plugin_codedb`, `codedb-build-capture`, and `codedb-mcp`.
