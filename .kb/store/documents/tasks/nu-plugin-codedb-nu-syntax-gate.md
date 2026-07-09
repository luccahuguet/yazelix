---
id: 019f21d4-b0f0-7ca3-af56-86524430ee79
slug: tasks/nu-plugin-codedb-nu-syntax-gate
title: "Extend CodeDB Nushell syntax gate"
type: task
status: completed
priority: medium
tags: [codedb, nushell, syntax, cdb056]
---

# Overview

Extend CodeDB's Nushell syntax validation so Nu tests, templates, examples, and stub fixtures parse under temp-HOME isolation.

This task maps source-package task `CDB056` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`.

## Source Task

- Task id: `CDB056`
- Title: `Extend syntax validator path for CodeDB fixtures`
- Phase: `syntax`
- Depends on: `CDB054`
- Blocks: `CDB058`
- Target surface: Nu syntax fixtures
- Allowed files: `tests/**`, `fixtures/**`, `docs/CODEDB_NUSHELL_SYNTAX_GATE.md`
- Raw log: `logs/CDB056-nu-syntax.log`
- Forbidden: real HOME dependency
- Gate: `nu --no-config-file --ide-check`
- Acceptance signal: `Nu syntax gate proven`

## Changes

- Added `tests/test_nushell_syntax_gate.nu`.
- Added `fixtures/nushell_syntax/stub_initializer.nu`.
- Added `docs/CODEDB_NUSHELL_SYNTAX_GATE.md`.
- Recorded evidence in `logs/CDB056-nu-syntax.log`.

## Acceptance Criteria

- [x] Syntax validation uses `nu --no-config-file --ide-check`.
- [x] Validation runs under a temporary `HOME`.
- [x] Validation uses an isolated `--plugin-config`.
- [x] Syntax fixtures include a Yazelix-like stub initializer instead of real HOME/plugin state.
- [x] Package Nu tests, templates, examples, and stub fixtures are checked.
- [x] The syntax report returns structured per-file rows.
- [x] No fixture `Cargo.lock` was generated.

## Verification

Commands run from `/home/flexnetos/Downloads/nu_plugin`:

```bash
nu --no-config-file tests/test_nushell_syntax_gate.nu

nu --no-config-file --ide-check 100 tests/test_nushell_syntax_gate.nu

nu --no-config-file -c 'source tests/test_nushell_syntax_gate.nu; main | to json'

rg -n "CDB056|temp-HOME|temp HOME|stub_initializer|nu --no-config-file --ide-check|Nu syntax gate proven|real HOME|plugin registry|Generated bridge files are not source truth" \
  tests/test_nushell_syntax_gate.nu fixtures/nushell_syntax/stub_initializer.nu \
  docs/CODEDB_NUSHELL_SYNTAX_GATE.md logs/CDB056-nu-syntax.log

find fixtures -name Cargo.lock -print | sort
```

Evidence:

- `tests/test_nushell_syntax_gate.nu` returned `status: passed`.
- Checked files: `13`.
- The JSON report returned one row per checked Nu file.
- Every syntax row had `status: passed` and `exit_code: 0`.
- Every syntax row had empty stderr hash `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- `nu --ide-check` for the gate itself exited 0 with hints only.
- Required marker search passed.
- Fixture `Cargo.lock` guard printed no paths.
