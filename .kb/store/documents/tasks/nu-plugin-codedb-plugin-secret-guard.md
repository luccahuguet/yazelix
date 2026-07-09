---
id: 019f21df-0795-76d2-a1b3-9344b9b30522
slug: tasks/nu-plugin-codedb-plugin-secret-guard
title: "Add CodeDB plugin secret leak guard"
type: task
status: completed
priority: medium
tags: [codedb, nushell, security, cdb060]
---

# Overview

Add a plugin stderr/stdout and MCP default-output guard proving CodeDB does not leak secret-looking fixture values through default Nu plugin or MCP surfaces.

This task maps source-package task `CDB060` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`.

## Source Task

- Task id: `CDB060`
- Title: `Add plugin stderr/trace secret-leak guard`
- Phase: `security`
- Depends on: `CDB052`, `CDB032`
- Blocks: `CDB062`
- Target surface: plugin logs/MCP outputs
- Allowed files: `tests/**`, `docs/SECURITY_AND_SECRET_POLICY.md`
- Raw log: `logs/CDB060-plugin-secret-guard.log`
- Forbidden: raw source/secret leak
- Gate: stderr/log/MCP leak tests; redaction report; test log
- Acceptance signal: `plugin outputs safe`

## Changes

- Added `tests/test_plugin_secret_guard.nu`.
- Updated `docs/SECURITY_AND_SECRET_POLICY.md` with plugin stderr/trace guard policy.
- Recorded evidence in `logs/CDB060-plugin-secret-guard.log`.

## Acceptance Criteria

- [x] Nu plugin transport is invoked through `nu --plugins`.
- [x] Secret-like fixture is copied to a temporary directory before testing.
- [x] Plugin stdout and stderr are checked for known secret-looking fixture values.
- [x] Plugin scan/source/rust/validation surfaces avoid raw secret-looking values.
- [x] MCP default tests avoid raw secret-looking values.
- [x] Redaction report records labels, output hashes, and row counts without echoing raw secret-looking values.
- [x] No fixture `Cargo.lock` was generated.

## Verification

Commands run from `/home/flexnetos/Downloads/nu_plugin`:

```bash
CODEDB_TEST_CARGO_DIR=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin \
  nu --no-config-file tests/test_plugin_secret_guard.nu

nu --no-config-file --ide-check 100 tests/test_plugin_secret_guard.nu

CODEDB_TEST_CARGO_DIR=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin \
  nu --no-config-file -c 'source tests/test_plugin_secret_guard.nu; main | to json'

rg -n "CDB060|plugin stderr|trace guard|secret_like_values|plugin_transport|nu --plugins|MCP|redaction report|must not echo" \
  tests/test_plugin_secret_guard.nu docs/SECURITY_AND_SECRET_POLICY.md logs/CDB060-plugin-secret-guard.log

find fixtures -name Cargo.lock -print | sort

nu --no-config-file tests/test_nushell_syntax_gate.nu
```

Evidence:

- `tests/test_plugin_secret_guard.nu` returned `status: passed`.
- `plugin_scan`: `secret_like_values: absent`.
- `plugin_source_files`: `secret_like_values: absent`.
- `plugin_rust_items`: `secret_like_values: absent`.
- `plugin_validation_errors`: `secret_like_values: absent`.
- `mcp_tests`: `secret_like_values: absent`.
- `plugin_transport`: `status: passed`.
- Plugin row counts: `scan_rows: 5`, `source_file_rows: 1`, `rust_item_rows: 3`, `validation_error_rows: 0`.
- `nu --ide-check` for the guard exited 0 with hints only.
- Required marker search passed.
- Fixture `Cargo.lock` guard printed no paths.
- Package Nu syntax gate returned `status: passed` and `checked_files: 17`.
