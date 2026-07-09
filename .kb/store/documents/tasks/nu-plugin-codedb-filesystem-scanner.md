---
id: 019f2133-e342-7ab2-a882-16c07c047d82
slug: tasks/nu-plugin-codedb-filesystem-scanner
title: "Implement CodeDB filesystem scanner"
type: task
status: completed
priority: high
tags: [nu_plugin, codedb, filesystem, scanner]
---

Implement the CDB017 slice from the `nu_plugin_codedb` execution package: add read-only deterministic filesystem scanning to `codedb-core`, producing stable filesystem entry rows for fixture paths without mutating source trees.

Acceptance criteria:
- filesystem scanner walks a root path read-only
- scanner rows include relative path, kind, size, readonly flag, symlink status, and classification
- scanner output ordering is deterministic
- tests prove repeated scans of the same fixture return identical rows
- implementation stays within CDB017 allowed files: `crates/codedb-core/**`
- raw validation log is preserved at `/home/flexnetos/Downloads/nu_plugin/logs/CDB017-fs.log`

Evidence:
- `cargo test -p codedb-core filesystem_scan_rows_are_stable` passed
- `cargo test -p codedb-core` passed
- raw log preserved at `/home/flexnetos/Downloads/nu_plugin/logs/CDB017-fs.log`
- implementation added std-only read-only filesystem scanner rows in `/home/flexnetos/Downloads/nu_plugin/crates/codedb_core/src/lib.rs`

Next package task by dependency order: `CDB018` (`Implement exact source metadata and blob policy`)
