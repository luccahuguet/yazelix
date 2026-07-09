---
id: 019f21ca-4799-79d0-9f2c-6c8274277ea6
slug: tasks/nu-plugin-codedb-temp-home-plugin-registry
title: "Implement CodeDB temp-HOME plugin registry smoke test"
type: task
status: completed
priority: medium
tags: [codedb, nushell, registry, cdb053]
---

# Overview

Add a temp-HOME Nu plugin registry smoke test that runs `plugin add`, then `plugin use`, without touching the user's persistent registry.

This task maps source-package task `CDB053` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`.

## Source Task

- Task id: `CDB053`
- Title: `Implement temp-HOME plugin registry smoke test`
- Phase: `nu-plugin`
- Depends on: `CDB051`
- Blocks: `CDB057`
- Target surface: temp HOME plugin registry
- Allowed files: `tests/**`, `examples/nushell/**`
- Raw log: `logs/CDB053-plugin-registry.log`
- Forbidden: using real HOME
- Gate: temp HOME artifact and test log
- Acceptance signal: `registry path proven safely`

## Changes

- Added `tests/test_plugin_registry.nu`.
- Added `examples/nushell/plugin_registry_smoke.nu`.
- Recorded evidence in `logs/CDB053-plugin-registry.log`.

## Acceptance Criteria

- [x] `plugin add` writes only to a temp plugin registry path.
- [x] `plugin use` loads from the temp registry and runs `codedb tables`.
- [x] Test output is table-shaped and includes CodeDB table rows.
- [x] Test compares likely real HOME Nushell plugin registry files before and after.
- [x] No default user registry or tracked Yazelix runtime config is mutated.

## Verification

Commands run from `/home/flexnetos/Downloads/nu_plugin`:

```bash
CODEDB_TEST_CARGO_DIR=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin \
  CODEDB_TEST_REPO_ROOT=/home/flexnetos/Downloads/nu_plugin \
  nu /home/flexnetos/Downloads/nu_plugin/tests/test_plugin_registry.nu

PATH=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin:$PATH cargo fmt --check
```

Evidence:

- Test output `status`: `passed`.
- Test output `row_count`: `8`.
- Test output `first_table`: `codedb_contexts`.
- Temp plugin registry: `/tmp/tmp.AeewBHO9Wa/plugins.msgpackz`.
- Temp plugin registry sha256: `1f6e2a9c5aa2905af0b40a048bcf74ab35da8722380a5aa7bddb8a121cafba92`.
- Nu 0.112 registered the plugin under `codedb`; `plugin use codedb` and the full plugin executable path work.
