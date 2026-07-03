---
id: 019f21d1-fd0f-7870-92e4-10d4839ad70c
slug: tasks/nu-plugin-codedb-init-provenance
title: "Verify CodeDB initializer provenance"
type: task
status: completed
priority: medium
tags: [codedb, yazelix, provenance, cdb055]
---

# Overview

Verify that CodeDB's generated Yazelix initializer bridge can prove its provenance from manifest rows and artifact checksums.

This task maps source-package task `CDB055` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`.

## Source Task

- Task id: `CDB055`
- Title: `Verify generated initializer checksums/provenance`
- Phase: `provenance`
- Depends on: `CDB054`
- Blocks: `CDB063`
- Target surface: generated initializers
- Allowed files: `tests/**`, `manifests/**`
- Raw log: `logs/CDB055-init-provenance.log`
- Forbidden: unchecked generated files
- Gate: manifest rows; checksum report
- Acceptance signal: `initializer provenance proven`

## Changes

- Added `tests/test_init_provenance.nu`.
- Added `manifests/INIT_PROVENANCE.json`.
- Recorded evidence in `logs/CDB055-init-provenance.log`.

CodeDB remains the high-fidelity typed store for source/file/table/provenance data. Envctl consumes that store as an export/materialization layer and can convert tables back to files or environment state when needed.

## Acceptance Criteria

- [x] Generated initializer bridge rows are parsed as structured data.
- [x] `codedb_init`, `codedb_extern`, and `codedb_bridge_manifest` rows are all present exactly once.
- [x] Each emitted row records generated state, template source truth, manual-edit prohibition, and plugin-registry non-mutation.
- [x] File SHA-256 values match the emitted rows.
- [x] The generated manifest agrees with emitted init/extern artifact checksums.
- [x] A committed provenance manifest records stable template artifact checksums.
- [x] No generated file is treated as source truth.

## Verification

Commands run from `/home/flexnetos/Downloads/nu_plugin`:

```bash
nu --no-config-file --ide-check 100 tests/test_init_provenance.nu

CODEDB_TEST_CARGO_DIR=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin \
  nu --no-config-file tests/test_init_provenance.nu

rg -n "CDB055|INIT_PROVENANCE|test_init_provenance|initializer provenance proven|No generated file is source truth|manual_edits_allowed|mutates_plugin_registry" \
  manifests/INIT_PROVENANCE.json tests/test_init_provenance.nu logs/CDB055-init-provenance.log

find fixtures -name Cargo.lock -print | sort
```

Evidence:

- `nu --ide-check` exited 0 with hints only.
- `tests/test_init_provenance.nu` returned `status: passed`.
- Generated row count: `3`.
- `codedb_init` sha256: `d769c2bdca562fc53979cb3bad0fd7940f12c2f55fdcc65bf6b79f35f3987797`.
- `codedb_extern` sha256: `203233735d97fa16eb19d4b96efb2d5f44cb9114163724266e6d35275b41e7e0`.
- Required provenance markers were present.
- Fixture `Cargo.lock` guard printed no paths.
