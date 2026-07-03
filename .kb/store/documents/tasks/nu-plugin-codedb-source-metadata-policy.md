---
id: 019f2136-9ca2-7ea3-b9b9-5379e0cb2bc3
slug: tasks/nu-plugin-codedb-source-metadata-policy
title: "Implement CodeDB source metadata and secret policy"
type: task
status: completed
priority: high
tags: [nu_plugin, codedb, source, security]
---

Implement the CDB018 slice from the `nu_plugin_codedb` execution package: add metadata-only source blob capture in `codedb-core`, compute exact source metadata and hashes, classify text/binary/newlines/BOM/UTF-8 status, and ensure secret-looking material is never exported as raw source by default.

Acceptance criteria:
- source metadata capture records path, length, hash, UTF-8/text status, newline style, BOM status, and export policy
- secret-looking input produces policy metadata and validation evidence without raw value export
- default source capture mode is metadata-only
- implementation stays within CDB018 allowed files: `crates/codedb-core/**`
- raw validation log is preserved at `/home/flexnetos/Downloads/nu_plugin/logs/CDB018-source.log`

Evidence:
- `cargo test -p codedb-core source_metadata_is_metadata_only_by_default` passed
- `cargo test -p codedb-core secret_like_source_is_redacted_by_policy` passed
- `cargo test -p codedb-core` passed
- raw log preserved at `/home/flexnetos/Downloads/nu_plugin/logs/CDB018-source.log`
- implementation added metadata-only SHA-256 source capture and redacted policy rows in `/home/flexnetos/Downloads/nu_plugin/crates/codedb_core/src/lib.rs`

Next package task by dependency order: `CDB019` (`Implement cargo metadata capture`)
