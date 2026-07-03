---
id: 019f21d6-fee9-7c50-a50a-23fd38b6ea7e
slug: tasks/nu-plugin-codedb-no-real-home-registration
title: "Add CodeDB no-real-HOME plugin registration test"
type: task
status: completed
priority: medium
tags: [codedb, nushell, safety, cdb057]
---

# Overview

Add a dedicated no-real-HOME plugin registration test that proves CodeDB plugin registration uses temp HOME/plugin config and leaves the operator's real Nushell plugin registry unchanged.

This task maps source-package task `CDB057` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`.

## Source Task

- Task id: `CDB057`
- Title: `Add no-real-HOME plugin registration test`
- Phase: `safety`
- Depends on: `CDB053`
- Blocks: none
- Target surface: HOME isolation
- Allowed files: `tests/**`
- Raw log: `logs/CDB057-no-real-home.log`
- Forbidden: mutating operator HOME
- Gate: HOME isolation test
- Acceptance signal: `HOME safety proven`

## Changes

- Added `tests/test_no_real_home_plugin_registration.nu`.
- Recorded evidence in `logs/CDB057-no-real-home.log`.

## Acceptance Criteria

- [x] Plugin registration runs under temporary `HOME`.
- [x] Plugin registration uses an isolated `--plugin-config`.
- [x] Test records before/after real-HOME registry hashes.
- [x] Test fails if real-HOME registry state changes.
- [x] Isolated plugin registration exposes CodeDB table rows.
- [x] No fixture `Cargo.lock` was generated.

## Verification

Commands run from `/home/flexnetos/Downloads/nu_plugin`:

```bash
CODEDB_TEST_CARGO_DIR=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin \
  nu --no-config-file tests/test_no_real_home_plugin_registration.nu

nu --no-config-file --ide-check 100 tests/test_no_real_home_plugin_registration.nu

CODEDB_TEST_CARGO_DIR=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin \
  nu --no-config-file -c 'source tests/test_no_real_home_plugin_registration.nu; main | to json'

rg -n "CDB057|real HOME|temp HOME|plugin config|before/after|HOME safety proven|real_home_registry_unchanged|Test lane|Defends" \
  tests/test_no_real_home_plugin_registration.nu logs/CDB057-no-real-home.log

find fixtures -name Cargo.lock -print | sort
```

Evidence:

- `tests/test_no_real_home_plugin_registration.nu` returned `status: passed`.
- `real_home_registry_unchanged`: `true`.
- `plugin_table_rows`: `8`.
- `real_before.file_count`: `0`.
- `real_after.file_count`: `0`.
- `real_before.snapshot_sha256`: `4f53cda18c2baa0c0354bb5f9a3ecbe5ed12ab4d8e11ba873c2f11161202b945`.
- `real_after.snapshot_sha256`: `4f53cda18c2baa0c0354bb5f9a3ecbe5ed12ab4d8e11ba873c2f11161202b945`.
- `nu --ide-check` for the test exited 0 with hints only.
- Required marker search passed.
- Fixture `Cargo.lock` guard printed no paths.
