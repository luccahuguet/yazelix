---
id: 019f20e2-8c3e-7dd3-92a3-57c3d322b97d
slug: tasks/nu-plugin-codedb-redb-store-init
title: "Implement CodeDB redb store init"
type: task
status: completed
priority: high
tags: [nu_plugin, codedb, redb, store]
---

Implement the CDB015 slice from the `nu_plugin_codedb` execution package: create and open the redb store, write schema/store/toolchain metadata rows, and expose a minimal init report that can be validated in tests.

Acceptance criteria:
- `codedb-store-redb` can create a redb database file on first use
- schema version and store metadata rows are written during init
- init/readback helpers prove the metadata rows are present
- the implementation stays within the package's allowed CDB015 surface
- store init tests pass in the package workspace

Evidence:
- `cargo test -p codedb-store-redb` passed
- raw log preserved at `/home/flexnetos/Downloads/nu_plugin/logs/CDB015-redb-init.log`
- the crate now writes and rereads schema, store, toolchain, and validation metadata rows

Next package task by dependency order: `CDB016` (`Implement redb schema version, locks, backup, restore`)
