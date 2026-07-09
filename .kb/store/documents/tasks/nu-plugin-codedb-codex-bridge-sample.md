---
id: 019f218d-ec7a-7921-8f48-98c8cb96472c
slug: tasks/nu-plugin-codedb-codex-bridge-sample
title: "Implement CodeDB Codex bridge docs and sample MCP config"
type: task
status: completed
priority: high
tags: [codedb, nu_plugin, codex, mcp, CDB037]
---

## Source Task

`/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv` row CDB037.

- Phase: integration
- Depends on: CDB032
- Target surface: Codex bridge
- Allowed files: `docs/CODEX_BRIDGE.md`; `examples/codex/**`
- Forbidden: Codex auth hacks
- Primary artifact: MCP config sample
- Execution gate: Codex bridge config parse proof
- Raw log: `logs/CDB037-codex-bridge.log`
- PRD sections: 16.1, 16.2

## Acceptance Criteria

- [x] `docs/CODEX_BRIDGE.md` documents the bounded CLI/MCP bridge for Codex.
- [x] A sample Codex MCP config exists under `examples/codex/**`.
- [x] The sample config launches the CodeDB MCP server without embedding auth/session hacks or raw secrets.
- [x] The sample config and docs preserve bounded-output defaults and read-only behavior.
- [x] Validation proves the sample config parses and references the expected command/args.
- [x] `logs/CDB037-codex-bridge.log` records validation commands and results.

## Notes

- Codex should consume CodeDB through bounded CLI/MCP datatables. Do not add browser auth tricks, session-token handling, or unbounded raw source reads.
- The sample is a bridge target, not an install side effect. It must be safe to copy into a Codex config only after the operator chooses concrete paths.

## Completion Evidence

Implemented in the source package under `/home/flexnetos/Downloads/nu_plugin`:

- `docs/CODEX_BRIDGE.md` now documents the Codex bounded CLI/MCP bridge, explicit repo selection, sample config path, and read-only/no-raw-source/no-auth-hack policy.
- `examples/codex/codedb_mcp_config.json` provides a valid JSON MCP config fragment using `codedb mcp serve --repo-path ... --store ... --default-limit 50 --max-bytes 65536`.
- The sample has no embedded auth/session-token/browser/cookie/API-key/password/bearer fields and contains no raw secrets.

Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB037-codex-bridge.log`.

Passing gates recorded there:

- JSON sample parsed by Nushell.
- MCP args begin with `mcp serve`.
- Bounded policy is true.
- `rawSourceDefault` is `disabled`.
- `--default-limit` and `--max-bytes` are present.
- Forbidden auth/session patterns are absent from the sample config.
- Docs reference the sample config, bridge command, read-only tools, raw-source-disabled default, and no browser/session/auth hacks.
