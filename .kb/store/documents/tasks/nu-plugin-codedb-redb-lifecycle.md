---
id: 019f21e2-394c-78b3-8e9d-f700322bd40b
slug: tasks/nu-plugin-codedb-redb-lifecycle
title: "Add CodeDB redb lock lifecycle test"
type: task
status: completed
priority: medium
tags: [codedb, redb, storage, cdb061]
---

# Overview

Add redb lock contention and plugin-like lifecycle release coverage so CodeDB's embedded store behavior is documented and safe.

This task maps source-package task `CDB061` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`.

## Source Task

- Task id: `CDB061`
- Title: `Add redb lock/plugin-GC behavior test`
- Phase: `storage`
- Depends on: `CDB014`, `CDB050`
- Blocks: none
- Target surface: redb lifecycle
- Allowed files: `crates/codedb-store-redb/**`, `tests/**`
- Raw log: `logs/CDB061-redb-gc.log`
- Forbidden: store corruption
- Gate: redb lock/GC smoke; redb test log
- Acceptance signal: `redb lifecycle safe`

## Changes

- Added redb validation rows for `lock_contention_behavior` and `plugin_lifecycle_gc`.
- Added `lock_contention_blocks_until_writer_lifecycle_release` to `crates/codedb_store_redb/src/lib.rs`.
- Added `tests/test_redb_lifecycle.nu`.
- Recorded evidence in `logs/CDB061-redb-gc.log`.

## Acceptance Criteria

- [x] Store metadata documents single-writer lock behavior.
- [x] Store metadata documents plugin-like lifecycle release behavior.
- [x] A second writer is blocked while the first writer is alive.
- [x] The second writer acquires the lock after the first writer is dropped.
- [x] A read transaction can coexist with the first writer.
- [x] The store remains readable after handles are dropped.
- [x] Backup/restore tests still pass with the new lifecycle rows.
- [x] No fixture `Cargo.lock` was generated.

## Verification

Commands run from `/home/flexnetos/Downloads/nu_plugin`:

```bash
PATH=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin:$PATH cargo fmt --check

PATH=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin:$PATH \
  cargo test -p codedb-store-redb lock_contention_blocks_until_writer_lifecycle_release --quiet

PATH=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin:$PATH \
  cargo test -p codedb-store-redb --quiet

CODEDB_TEST_CARGO_DIR=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin \
  nu --no-config-file tests/test_redb_lifecycle.nu

nu --no-config-file --ide-check 100 tests/test_redb_lifecycle.nu

CODEDB_TEST_CARGO_DIR=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin \
  nu --no-config-file -c 'source tests/test_redb_lifecycle.nu; main | to json'

rg -n "CDB061|lock_contention|plugin_lifecycle_gc|single_writer_blocks_until_release|drop_releases_write_lock|redb lifecycle|redb_lock_contention|Test lane|Defends" \
  crates/codedb_store_redb/src/lib.rs tests/test_redb_lifecycle.nu logs/CDB061-redb-gc.log

find fixtures -name Cargo.lock -print | sort

nu --no-config-file tests/test_nushell_syntax_gate.nu
```

Evidence:

- Focused redb lifecycle test returned `1 passed`.
- Full `codedb-store-redb` crate tests returned `3 passed`.
- Nu wrapper returned `redb_lock_contention: passed`.
- Nu wrapper returned `redb_store_crate_tests: passed`.
- JSON report recorded `single_writer_blocks_until_release`.
- JSON report recorded `drop_releases_write_lock`.
- `nu --ide-check` for the wrapper exited 0 with hints only.
- Required marker search passed.
- Fixture `Cargo.lock` guard printed no paths.
- Package Nu syntax gate returned `status: passed` and `checked_files: 18`.
