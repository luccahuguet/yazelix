---
id: 019f21c7-100e-7061-b0d2-df05e751b55b
slug: tasks/nu-plugin-codedb-transient-nu-plugin-smoke
title: "Implement transient CodeDB nu --plugins smoke test"
type: task
status: completed
priority: medium
tags: [codedb, nushell, tests, cdb052]
---

# Overview

Add and run a transient `nu --plugins` smoke test for `nu_plugin_codedb` using an isolated HOME/XDG/plugin-config surface.

This task maps source-package task `CDB052` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`.

## Source Task

- Task id: `CDB052`
- Title: `Implement transient nu --plugins smoke test`
- Phase: `nu-plugin`
- Depends on: `CDB051`
- Blocks: `CDB054`, `CDB060`
- Target surface: transient Nu plugin load
- Allowed files: `tests/**`, `examples/nushell/**`
- Raw log: `logs/CDB052-transient-plugin.log`
- Forbidden: mutating plugin registry
- Gate: test log and Nu output
- Acceptance signal: `transient plugin load proven`

## Changes

- Added `tests/test_transient_plugin.nu`.
- Added `examples/nushell/transient_plugin_smoke.nu`.
- Recorded evidence in `logs/CDB052-transient-plugin.log`.

## Acceptance Criteria

- [x] Transient `nu --plugins <path>` smoke runs with temp HOME/XDG roots.
- [x] Smoke uses temp `--plugin-config`, not the real user registry.
- [x] Smoke runs `codedb tables` through the plugin and returns table-shaped output.
- [x] Test checks likely real HOME Nushell plugin registry files before and after.
- [x] No `plugin add` command is used.

## Verification

Commands run from `/home/flexnetos/Downloads/nu_plugin`:

```bash
CODEDB_TEST_CARGO_DIR=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin \
  CODEDB_TEST_REPO_ROOT=/home/flexnetos/Downloads/nu_plugin \
  nu /home/flexnetos/Downloads/nu_plugin/tests/test_transient_plugin.nu

PATH=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin:$PATH cargo fmt --check
```

Evidence:

- Test output `status`: `passed`.
- Test output `row_count`: `8`.
- Test output `first_table`: `codedb_contexts`.
- Test plugin path: `/home/flexnetos/Downloads/nu_plugin/target/debug/nu_plugin_codedb`.
