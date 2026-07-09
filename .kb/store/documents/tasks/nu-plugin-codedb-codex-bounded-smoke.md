---
id: 019f21e5-eb3d-7ce3-b0bc-286ebefadc3e
slug: tasks/nu-plugin-codedb-codex-bounded-smoke
title: "Add CodeDB Codex bounded bridge smoke"
type: task
status: completed
priority: medium
tags: [codedb, codex, mcp, cdb062]
---

# Overview

Add a Codex-facing smoke test proving CodeDB CLI and MCP bridge samples stay bounded, read-only, raw-source-free, and free of auth/session hacks.

This task maps source-package task `CDB062` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`.

## Source Task

- Task id: `CDB062`
- Title: `Add Codex bounded CLI/MCP invocation smoke`
- Phase: `codex`
- Depends on: `CDB032`, `CDB060`
- Blocks: none
- Target surface: Codex CLI/MCP bridge
- Allowed files: `tests/**`, `docs/CODEX_BRIDGE.md`, `examples/codex/**`
- Raw log: `logs/CDB062-codex-bounded.log`
- Forbidden: unbounded source reads
- Gate: Codex bridge smoke; MCP tool report; CLI output sample
- Acceptance signal: `Codex bridge safe`

## Changes

- Added `tests/test_codex_bounded_bridge.nu`.
- Added `examples/codex/codedb_bounded_smoke_report.json`.
- Updated `docs/CODEX_BRIDGE.md` with executable CDB062 proof details.
- Recorded evidence in `logs/CDB062-codex-bounded.log`.

## Acceptance Criteria

- [x] Codex MCP sample config keeps `--default-limit 50`.
- [x] Codex MCP sample config keeps `--max-bytes 65536`.
- [x] Codex MCP sample config contains no auth, token, browser-session, or secret env fields.
- [x] Codex-safe CLI doctor sample stays under 50 rows and 65536 bytes.
- [x] Codex-safe CLI scan sample stays under 50 rows and 65536 bytes.
- [x] CLI samples avoid raw source and secret-looking fixture values.
- [x] MCP crate tests pass and continue to cover row limits, byte limits, blocked raw-source tools, and metadata-only summaries.
- [x] No fixture `Cargo.lock` was generated.

## Verification

Commands run from `/home/flexnetos/Downloads/nu_plugin`:

```bash
CODEDB_TEST_CARGO_DIR=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin \
  nu --no-config-file tests/test_codex_bounded_bridge.nu

nu --no-config-file --ide-check 100 tests/test_codex_bounded_bridge.nu

CODEDB_TEST_CARGO_DIR=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin \
  nu --no-config-file -c 'source tests/test_codex_bounded_bridge.nu; main | to json'

rg -n "CDB062|bounded|Codex|raw source|auth|session|token|secret_like_values|codedb_bounded_smoke_report|defaultRowLimit|maxBytes|nu --no-config-file" \
  tests/test_codex_bounded_bridge.nu docs/CODEX_BRIDGE.md \
  examples/codex/codedb_bounded_smoke_report.json logs/CDB062-codex-bounded.log

find fixtures -name Cargo.lock -print | sort

nu --no-config-file tests/test_nushell_syntax_gate.nu

jq . examples/codex/codedb_bounded_smoke_report.json
```

Evidence:

- `tests/test_codex_bounded_bridge.nu` returned `codex_mcp_config: passed`.
- `codex_doctor_cli`: `2` rows, `603` bytes.
- `codex_scan_cli`: `6` rows, `1020` bytes.
- `codex_mcp_tests`: `passed`.
- `nu --ide-check` for the smoke exited 0 with hints only.
- Required marker search passed.
- Fixture `Cargo.lock` guard printed no paths.
- Package Nu syntax gate returned `status: passed` and `checked_files: 19`.
- `examples/codex/codedb_bounded_smoke_report.json` parsed successfully with `jq`.
