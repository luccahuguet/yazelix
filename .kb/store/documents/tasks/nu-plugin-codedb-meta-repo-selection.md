---
id: 019f2189-dad4-7793-bf39-ec151b68b3ad
slug: tasks/nu-plugin-codedb-meta-repo-selection
title: "Implement CodeDB meta repo selection inputs"
type: task
status: completed
priority: high
tags: [codedb, nu_plugin, meta, CDB036]
---

## Source Task

`/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv` row CDB036.

- Phase: meta
- Depends on: CDB029
- Target surface: meta integration
- Allowed files: `codedb CLI`; `docs/META_INTEGRATION.md`
- Forbidden: meta mutation
- Primary artifact: repo selection inputs
- Execution gate: meta target can be passed explicitly
- Raw log: `logs/CDB036-meta.log`
- PRD sections: 16.4

## Acceptance Criteria

- [x] CodeDB CLI accepts explicit `--repo-id` and `--repo-path` selection inputs without mutating meta state.
- [x] Repo selection is visible as datatable rows suitable for downstream Nu/meta workflows.
- [x] Selection supports explicit repo paths and optional repo labels/names without requiring the `meta` binary.
- [x] `docs/META_INTEGRATION.md` documents the explicit-input contract and the no-mutation boundary.
- [x] `logs/CDB036-meta.log` records validation commands and results.

## Notes

- CodeDB remains the authoritative file-to-table conversion layer; this task only gives callers a precise way to choose which repo roots to scan/export.
- Do not add implicit meta discovery that depends on host state. If `meta` integration grows later, it should feed the same explicit selection rows.

## Completion Evidence

Implemented in the source package under `/home/flexnetos/Downloads/nu_plugin`:

- `crates/codedb/src/main.rs` now accepts `--repo-id`, `--repo-path`, and `--store` for explicit repo selection.
- `codedb scan --repo-id <id> --repo-path <path>` emits a `meta_repo_selection` row before scan summary rows.
- `codedb export meta_repo_selection --repo-id <id> --repo-path <path>` emits the selection row in JSON, NUON, or CSV.
- Conflicting positional and explicit repo paths fail with a clear error.
- `docs/META_INTEGRATION.md` documents the explicit-input/no-mutation contract.

Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB036-meta.log`.

Passing gates recorded there:

- `cargo fmt -p codedb --check`
- `cargo build -p codedb`
- explicit `scan --repo-id fixture-project --repo-path ... --store ... --format json` parsed by Nushell; 6 rows
- `export meta_repo_selection --format nuon` parsed by Nushell; 1 row
- `export meta_repo_selection --format csv` parsed by Nushell; 1 row
- conflicting positional and explicit repo paths refused
