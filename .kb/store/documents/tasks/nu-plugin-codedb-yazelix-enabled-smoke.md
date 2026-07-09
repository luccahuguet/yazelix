---
id: 019f21dc-1ca8-7c51-b6f9-5d0fd73ee3bb
slug: tasks/nu-plugin-codedb-yazelix-enabled-smoke
title: "Add CodeDB-enabled Yazelix launch smoke"
type: task
status: completed
priority: medium
tags: [codedb, yazelix, smoke, cdb059]
---

# Overview

Add a Yazelix-like launch smoke proving CodeDB bridge paths can be present while startup remains ready, light, and registry-free.

This task maps source-package task `CDB059` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`.

## Source Task

- Task id: `CDB059`
- Title: `Add Yazelix launch smoke with CodeDB enabled`
- Phase: `yazelix-smoke`
- Depends on: `CDB058`, `CDB054`
- Blocks: none
- Target surface: Yazelix runtime enabled mode
- Allowed files: `tests/**`, `docs/YAZELIX_PLACEMENT.md`
- Raw log: `logs/CDB059-yazelix-enabled.log`
- Forbidden: startup heavy import
- Gate: enabled smoke; launch smoke log
- Acceptance signal: `enabled mode safe`

## Changes

- Added `tests/test_yazelix_enabled_smoke.nu`.
- Updated `docs/YAZELIX_PLACEMENT.md` with enabled launch smoke expectations.
- Recorded evidence in `logs/CDB059-yazelix-enabled.log`.

## Acceptance Criteria

- [x] Smoke models a Yazelix-managed Nu launch with explicit CodeDB runtime paths.
- [x] Smoke builds `codedb` and `nu_plugin_codedb`.
- [x] Smoke generates the CodeDB bridge under a temporary Yazelix-like state directory.
- [x] Launch probe reaches `status: ready`.
- [x] CLI status is `available`.
- [x] plugin status is `available`.
- [x] `CODEDB_BIN` and `CODEDB_NU_PLUGIN_BIN` are exported to explicit built paths.
- [x] No `plugin add`, `plugin use`, or registry creation happens during launch.
- [x] No fixture `Cargo.lock` was generated.

## Verification

Commands run from `/home/flexnetos/Downloads/nu_plugin`:

```bash
CODEDB_TEST_CARGO_DIR=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin \
  nu --no-config-file tests/test_yazelix_enabled_smoke.nu

nu --no-config-file --ide-check 100 tests/test_yazelix_enabled_smoke.nu

CODEDB_TEST_CARGO_DIR=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin \
  nu --no-config-file -c 'source tests/test_yazelix_enabled_smoke.nu; main | to json'

rg -n "CDB059|enabled_mode_safe|CodeDB bridge paths|startup light|plugin add|plugin use|registry creation|CODEDB_CLI_STATUS|CODEDB_NU_PLUGIN_STATUS|tracked Yazelix config" \
  tests/test_yazelix_enabled_smoke.nu docs/YAZELIX_PLACEMENT.md logs/CDB059-yazelix-enabled.log

find fixtures -name Cargo.lock -print | sort

nu --no-config-file tests/test_nushell_syntax_gate.nu
```

Evidence:

- `tests/test_yazelix_enabled_smoke.nu` returned `status: passed`.
- `enabled_mode_safe`: `true`.
- `launch_status`: `ready`.
- `codedb_cli_status`: `available`.
- `codedb_plugin_status`: `available`.
- Generated bridge rows: `3`.
- `plugin_registry_created`: `false`.
- `nu --ide-check` for the smoke exited 0 with hints only.
- Required marker search passed.
- Fixture `Cargo.lock` guard printed no paths.
- Package Nu syntax gate returned `status: passed` and `checked_files: 16`.
