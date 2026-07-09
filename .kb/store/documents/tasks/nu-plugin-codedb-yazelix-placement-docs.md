---
id: 019f2191-63f3-7641-8065-6dcad836f5b6
slug: tasks/nu-plugin-codedb-yazelix-placement-docs
title: "Implement CodeDB Yazelix placement docs"
type: task
status: completed
priority: high
tags: [codedb, nu_plugin, yazelix, CDB038]
---

## Source Task

`/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv` row CDB038.

- Phase: integration
- Depends on: CDB031
- Target surface: Yazelix placement
- Allowed files: `docs/YAZELIX_PLACEMENT.md`
- Forbidden: claiming Yazelix plugin ownership
- Primary artifact: Yazelix placement docs
- Execution gate: host/runtime Nu distinction documented
- Raw log: `logs/CDB038-yazelix.log`
- PRD sections: 16.3
- Blocks: CDB049

## Acceptance Criteria

- [x] `docs/YAZELIX_PLACEMENT.md` states that CodeDB is not a Yazelix plugin.
- [x] The document distinguishes host Nu from Yazelix runtime Nu and explains why plugin registry/protocol state must be checked per runtime.
- [x] The document places CodeDB as CLI, Nu plugin, Codex sidecar, and possible status/report source without bypassing Yazelix pane/session ownership.
- [x] The document forbids editing tracked Yazelix runtime config as an install side effect.
- [x] `logs/CDB038-yazelix.log` records validation commands and results.

## Notes

- Keep this as a placement contract only. Do not edit Yazelix source, runtime config, or plugin registries for this task.
- CodeDB should be packaged/hosted by Yazelix surfaces, but Yazelix remains the owner of operator flow.

## Completion Evidence

Implemented in the source package under `/home/flexnetos/Downloads/nu_plugin`:

- `docs/YAZELIX_PLACEMENT.md` now states CodeDB is not a Yazelix plugin, not a Zellij plugin owner, not a replacement for Yazelix generated shell initializers, and not a second Nushell startup-config owner.
- The document distinguishes host Nushell from Yazelix runtime Nushell and ties validation to `codedb doctor --nu` and `codedb doctor --yazelix`.
- The document places CodeDB as a runtime Nu plugin, CLI executable, Codex sidecar, and possible status/report source while keeping Yazelix pane/session ownership intact.
- The document forbids tracked `nushell/config/config.nu` mutation and real-HOME registry smoke.

Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB038-yazelix.log`.

Passing gates recorded there:

- Required placement statements found.
- Host/runtime Nu distinction found.
- No tracked config mutation contract found.
- Placement surfaces found.
- Forbidden Yazelix ownership claims absent.
