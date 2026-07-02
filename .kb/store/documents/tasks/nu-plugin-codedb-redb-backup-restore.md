---
id: 019f20f7-8637-76a2-b3cd-33d52801435e
slug: tasks/nu-plugin-codedb-redb-backup-restore
title: "Implement CodeDB redb backup and restore smoke"
type: task
status: completed
priority: high
tags: [nu_plugin, codedb, redb, backup, restore]
---

Implement the CDB016 slice from the `nu_plugin_codedb` execution package: add schema version/readback refinements, explicit lock and migration policy metadata, backup/export helpers, restore smoke helpers, and checksum evidence for cleanly closed redb files.

Acceptance criteria:
- backup/export API copies a cleanly closed redb store to a declared output path
- backup report includes checksum evidence
- restore smoke API recreates a store from backup and proves metadata readback
- lock, reader, migration, and corruption-validation policies are explicit metadata rows
- `cargo test -p codedb-store-redb` passes with raw log preserved at `logs/CDB016-redb-restore.log`

Evidence:
- `cargo test -p codedb-store-redb` passed
- raw log preserved at `/home/flexnetos/Downloads/nu_plugin/logs/CDB016-redb-restore.log`
- backup report includes byte count and SHA-256 checksum
- restore report verifies restored checksum and store metadata readback

Next package task by dependency order: `CDB017` (`Implement filesystem scanner`)
