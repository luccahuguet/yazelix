---
id: 019f2158-61a7-70d2-99e8-b63936bc3a43
slug: tasks/nu-plugin-codedb-no-mutation-proof
title: "Implement CodeDB no-mutation proof"
type: task
status: completed
priority: high
tags: [nu_plugin, codedb, proof, no_mutation]
---

## Overview

Implement package task `CDB028` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`: prove CodeDB read-only operations do not mutate a Git repo by comparing before/after status and file hash manifests.

## Scope

- Package path: `/home/flexnetos/Downloads/nu_plugin`
- CSV task: `CDB028`
- Depends on completed implementation row: `CDB017`
- Allowed source surface: `crates/codedb-core/**`, represented on disk as `crates/codedb_core/**`
- Raw validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB028-no-mutation.log`

## Stop Conditions

- Do not mutate the target repo while proving no mutation
- Do not silently pass when Git is unavailable
- Do not treat a pre-existing dirty repo as a mutation introduced by the proof

## Acceptance Criteria

- [x] Before/after Git status is captured
- [x] Before/after file hash manifests are captured
- [x] Proof distinguishes clean repos from pre-existing dirty repos
- [x] Clean and dirty Git fixtures pass
- [x] `/home/flexnetos/Downloads/nu_plugin/logs/CDB028-no-mutation.log` records the validation commands and results

## Evidence

- Extended `/home/flexnetos/Downloads/nu_plugin/crates/codedb_core/src/lib.rs` with `NoMutationProof`, `GitRepoSnapshot`, and `prove_no_mutation()`
- Captures before/after Git status and file manifest hash while skipping `.git`
- Records pre-existing dirty state separately from mutation detected by the operation
- Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB028-no-mutation.log`
- Passing commands:
  - `cargo fmt -p codedb-core --check`
  - `cargo test -p codedb-core clean_git_fixture_proves_no_mutation`
  - `cargo test -p codedb-core dirty_git_fixture_proves_no_new_mutation`
  - `cargo test -p codedb-core`

## Next

Next CSV task by dependency order: `CDB029` (`Implement codedb CLI scan/export/schema`).
