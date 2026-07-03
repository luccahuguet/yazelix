---
id: 019f21f9-84cc-7cf0-b9f7-69114df2f654
slug: tasks/nu-plugin-codedb-csv-source-of-truth-repair
title: "Verify CodeDB CSV source-of-truth repair"
type: task
status: completed
priority: medium
---

## Source

- CSV task: `CDB068`
- Title: Repair TASK_GRAPH CSV source-of-truth file linkage
- Depends on: `CDB067`
- Blocks: `CDB013`
- Allowed files: `execution/TASK_GRAPH.csv`, `execution/TASK_GRAPH.md`, `execution/TASK_FILE_MAP.csv`, navigation/gate/checklist/manifest/log package-governance files
- Evidence log: `logs/CDB068-csv-source-of-truth-repair.log`
- Forbidden actions: rewrite PRD or implementation docs, mark Rust implementation tasks complete, create simulated build evidence

## Scope

Verify that `execution/TASK_GRAPH.csv` is the strict source of truth for task
execution, that current package artifact references are exact package-relative
paths, and that implementation starts only after the repair gate.

## Acceptance

- [x] `execution/TASK_GRAPH.csv` parses with the repaired CDB068 row
- [x] `execution/TASK_FILE_MAP.csv` covers every task row
- [x] source-of-truth/path-resolution columns are present
- [x] task IDs are unique and dependency references resolve
- [x] dependency graph is acyclic
- [x] `CDB013` depends on `CDB068`
- [x] completed rows have existing raw logs and exact current evidence paths
- [x] `manifests/CSV_SOURCE_OF_TRUTH_REPAIR.json` matches the current task count
- [x] resealed package checksums validate

## Evidence

- Current audit reported 69 `TASK_GRAPH.csv` rows and 69 `TASK_FILE_MAP.csv` rows
- Required repair columns were present: `source_truth`, `governing_docs`, `acceptance_refs`, `first_run_refs`, `stop_condition_refs`, `checklist_source_files`, `current_artifact_paths`, `future_artifact_paths`, `non_file_outputs`, `path_resolution_status`, `evidence_status`, `path_policy`, and `implementation_start_gate`
- Complete rows were checked for `complete_current_paths_exact`, `evidence_files_present`, existing raw logs, and exact non-glob current evidence paths
- `CDB013` currently depends on `CDB006;CDB068`
- `manifests/CSV_SOURCE_OF_TRUTH_REPAIR.json` reports `after.task_count = 69` and `after.new_task = CDB068`
- `sha256sum -c manifests/CHECKSUMS.sha256` passed after the CDB067 reseal

## Result

`CSV has exact current package links, evidence logs exist, and validation
passes`: the repaired CSV is the current package task source of truth and
implementation is gated behind the repair task.
