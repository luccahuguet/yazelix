---
id: 019f21d9-4f63-7c01-b203-22192ff20c84
slug: tasks/nu-plugin-codedb-yazelix-disabled-smoke
title: "Add CodeDB-disabled Yazelix launch smoke"
type: task
status: completed
priority: medium
tags: [codedb, yazelix, smoke, cdb058]
---

# Overview

Add a Yazelix-like launch smoke proving CodeDB is optional and a disabled/absent CodeDB bridge does not block Nu startup.

This task maps source-package task `CDB058` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`.

## Source Task

- Task id: `CDB058`
- Title: `Add Yazelix launch smoke with CodeDB disabled`
- Phase: `yazelix-smoke`
- Depends on: `CDB049`, `CDB056`
- Blocks: `CDB059`
- Target surface: Yazelix runtime disabled mode
- Allowed files: `tests/**`, `docs/YAZELIX_PLACEMENT.md`
- Raw log: `logs/CDB058-yazelix-disabled.log`
- Forbidden: making CodeDB required for launch
- Gate: disabled smoke; launch smoke log
- Acceptance signal: `disabled mode safe`

## Changes

- Added `tests/test_yazelix_disabled_smoke.nu`.
- Updated `docs/YAZELIX_PLACEMENT.md` with disabled launch smoke expectations.
- Recorded evidence in `logs/CDB058-yazelix-disabled.log`.

## Acceptance Criteria

- [x] Smoke models a Yazelix-managed Nu launch with `IN_YAZELIX_SHELL` and `YAZELIX_RUNTIME_DIR`.
- [x] Smoke generates the CodeDB bridge under a temporary Yazelix-like state directory.
- [x] Smoke runs with `YAZELIX_CODEDB_BIN` and `YAZELIX_CODEDB_PLUGIN_BIN` absent.
- [x] Launch probe reaches `status: ready`.
- [x] CLI status is `missing:YAZELIX_CODEDB_BIN`.
- [x] plugin status is `missing:YAZELIX_CODEDB_PLUGIN_BIN`.
- [x] No CodeDB binary/plugin path is exported in disabled mode.
- [x] No plugin registration is attempted.
- [x] No fixture `Cargo.lock` was generated.

## Verification

Commands run from `/home/flexnetos/Downloads/nu_plugin`:

```bash
CODEDB_TEST_CARGO_DIR=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin \
  nu --no-config-file tests/test_yazelix_disabled_smoke.nu

nu --no-config-file --ide-check 100 tests/test_yazelix_disabled_smoke.nu

CODEDB_TEST_CARGO_DIR=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin \
  nu --no-config-file -c 'source tests/test_yazelix_disabled_smoke.nu; main | to json'

rg -n "CDB058|disabled_mode_safe|Yazelix launch|CodeDB must be optional|missing:YAZELIX_CODEDB_BIN|missing:YAZELIX_CODEDB_PLUGIN_BIN|no plugin registration|tracked Yazelix runtime config" \
  tests/test_yazelix_disabled_smoke.nu docs/YAZELIX_PLACEMENT.md logs/CDB058-yazelix-disabled.log

find fixtures -name Cargo.lock -print | sort

nu --no-config-file tests/test_nushell_syntax_gate.nu
```

Evidence:

- `tests/test_yazelix_disabled_smoke.nu` returned `status: passed`.
- `disabled_mode_safe`: `true`.
- `launch_status`: `ready`.
- `codedb_cli_status`: `missing:YAZELIX_CODEDB_BIN`.
- `codedb_plugin_status`: `missing:YAZELIX_CODEDB_PLUGIN_BIN`.
- Generated bridge rows: `3`.
- `nu --ide-check` for the smoke exited 0 with hints only.
- Required marker search passed.
- Fixture `Cargo.lock` guard printed no paths.
- Package Nu syntax gate returned `status: passed` and `checked_files: 15`.
