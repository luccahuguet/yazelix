---
id: 019f217f-af05-7eb3-970b-482aa6a3b3eb
slug: tasks/nu-plugin-codedb-envctl-export
title: "Implement CodeDB envctl export contract"
type: task
status: completed
priority: high
tags: [codedb, nu_plugin, envctl, CDB035]
---

## Source Task

`/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv` row CDB035.

- Phase: exports
- Depends on: CDB029
- Target surface: code;docs
- Allowed files: `crates/codedb/**`; `docs/ENVCTL_EXPORT_CONTRACT.md`
- Forbidden: envctl reading redb internals
- Primary artifact: export manifests
- Validation gate: envctl export validates
- Raw log: `logs/CDB035-envctl-export.log`
- PRD sections: 16.5, 17
- Blocks: CDB063

## Envctl Context

The sibling repository `/home/flexnetos/FlexNetOS/src/envctl` is present and fetched at `98a7e36090ddb53b3af1b7db622a14559a29665d`, matching `origin/master`.

Relevant envctl contracts inspected:

- `docs/env-table-schema.md` defines canonical table rows with common columns such as `row_id`, `table_name`, `schema_version`, `owner`, `source_role`, `source_format`, `source_checksum`, `scope`, `sensitive`, `generated`, and `validation_status`.
- `scripts/generate-bootstrap.nu` emits generated-file manifest rows with `source_table`, `source_table_checksum`, `output_checksum`, `header_status`, `manual_edits_allowed`, `secret_policy`, and `validation_status`.
- No existing CodeDB-specific envctl consumer exists, so CodeDB must publish stable export rows and manifests; envctl must not inspect CodeDB redb internals.

## Authority Boundary

CodeDB is the more accurate source for file-to-datatable conversion and code facts. It owns blob semantics, source file inventory, Rust/crate static semantics, build/proc-macro capture evidence, capture gaps, validation errors, and table checksums. Envctl consumes CodeDB's exported datatables/manifests when it needs to materialize or reconcile files, but it does not derive CodeDB facts itself and does not read CodeDB storage internals.

## Acceptance Criteria

- [x] CodeDB exposes an envctl export surface in `crates/codedb/**`.
- [x] Export rows preserve CodeDB as the authoritative observation/datatable layer and describe envctl as a downstream materialization/reconciliation consumer.
- [x] Export rows follow envctl table conventions, including schema version, owner, source role, source format, source checksum/checksum provenance, sensitivity, generated/manual flags, and validation status.
- [x] Export manifest rows include table checksum provenance suitable for envctl-generated-file style validation.
- [x] The implementation documents that envctl consumes only exported rows/manifests and never reads CodeDB redb internals.
- [x] Validation proves JSON, NUON, and CSV export formats are parseable and include tool versions, database endpoint, capture status, table checksums, validation errors, cache/log directories, and release artifact rows.
- [x] `logs/CDB035-envctl-export.log` records the validation commands and results.

## Notes

- Keep this slice export-only. Do not add envctl writes, envctl runtime assumptions, or a redb schema dependency.
- If the envctl repository later adds a native CodeDB importer, it should target these stable exported rows rather than CodeDB storage internals.

## Completion Evidence

Implemented in the source package under `/home/flexnetos/Downloads/nu_plugin`:

- `crates/codedb/src/main.rs` now supports `codedb export envctl` plus specific tables for `codedb_tool_versions`, `codedb_database_endpoints`, `codedb_capture_status`, `codedb_table_checksums`, `codedb_validation_errors`, `codedb_cache_dirs`, `codedb_log_dirs`, `codedb_release_artifacts`, `codedb_source_root_hashes`, and `codedb_export_manifests`.
- `docs/ENVCTL_EXPORT_CONTRACT.md` now states the authority split: CodeDB owns accurate file/datatable/blob/Rust/crate facts; envctl consumes exports for materialization and reconciliation.

Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB035-envctl-export.log`.

Passing gates recorded there:

- `cargo fmt -p codedb --check`
- `cargo build -p codedb`
- `codedb export envctl --format json` parsed by Nushell; 23 rows
- `codedb export envctl --format nuon` parsed by Nushell; 23 rows
- `codedb export codedb_table_checksums --format csv` parsed by Nushell; 9 rows
- envctl sibling repo fetched and verified at `98a7e36090ddb53b3af1b7db622a14559a29665d`
