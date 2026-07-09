---
id: 019f2197-53a0-72f0-9d65-3105db643f61
slug: tasks/nu-plugin-codedb-tooling-boundary-docs
title: "Implement CodeDB GitKB RTK Kache wild Fenix integration docs"
type: task
status: completed
priority: high
tags: [codedb, nu_plugin, gitkb, rtk, toolchain, CDB040]
---

## Source Task

`/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv` row CDB040.

- Phase: integration
- Depends on: CDB009
- Target surface: docs
- Allowed files: `docs/INTEGRATION_CONTRACTS.md`
- Forbidden: ownership confusion
- Primary artifact: integration docs
- Execution gate: facts/export boundaries clear
- Raw log: `logs/CDB040-tooling.log`
- PRD sections: 16.7, 16.8, 16.9

## Acceptance Criteria

- [x] `docs/INTEGRATION_CONTRACTS.md` clearly describes GitKB as durable explanation/handoff storage, not raw source or release truth by itself.
- [x] The document clearly describes RTK as a summarization/compression surface that must preserve raw failure logs.
- [x] The document clearly describes Kache, wild, and Fenix as captured facts/toolchain context, not CodeDB-owned installs.
- [x] The document states which rows/exports CodeDB should provide for these integrations.
- [x] Ownership boundaries are explicit and avoid confusing CodeDB, GitKB, RTK, Kache, wild, and Fenix responsibilities.
- [x] `logs/CDB040-tooling.log` records validation commands and results.

## Notes

- This is a documentation slice only. Do not add probes, installers, or local host assumptions.
- Facts may be exported later; this task names the boundary and expected row classes.

## Completion Evidence

Implemented in the source package under `/home/flexnetos/Downloads/nu_plugin`:

- `docs/INTEGRATION_CONTRACTS.md` now includes detailed GitKB, RTK, and Kache/wild/Fenix boundary sections.
- GitKB is documented as durable explanation/handoff storage that may link raw logs and proof artifacts but must not store raw source blobs or replace runner proof.
- RTK is documented as summarization/compression that must preserve raw failure logs and root-cause evidence.
- Kache, wild, and Fenix are documented as environment/toolchain facts, not CodeDB-owned installs.
- Expected row/export classes are named, including `capture_gaps`, `validation_errors`, `meta_repo_selection`, `runner_proof_manifest`, `raw_log_paths`, `kache_status`, wild linker facts, and Fenix toolchain facts.
- The ownership matrix names owner, CodeDB responsibility, and forbidden crossing for each surface.

Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB040-tooling.log`.

Passing gates recorded there:

- GitKB boundary terms found.
- RTK boundary terms found.
- Kache/wild/Fenix boundary terms found.
- Ownership matrix terms found.
- Forbidden ownership-confusion phrases absent.
