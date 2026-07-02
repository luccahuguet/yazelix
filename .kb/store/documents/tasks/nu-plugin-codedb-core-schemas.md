---
id: 019f20df-6c56-7090-b91d-922692efdd04
slug: tasks/nu-plugin-codedb-core-schemas
title: "Implement CodeDB core schema types"
type: task
status: completed
priority: high
tags: [nu_plugin, codedb, schemas, rust]
---

Implement the CDB014 slice from the `nu_plugin_codedb` execution package: define core schema and identity types in `codedb-core`, replace placeholder row helpers with structured schema models, and prepare the crate for later store and scan layers.

Acceptance criteria:
- `codedb-core` exposes durable schema and identity types for the first V1.1 slice
- placeholder status rows are replaced by schema-oriented types and helpers
- the crate compiles cleanly in the package workspace
- the implementation stays within the package's allowed CDB014 surface
- follow-on work for store and scan layers is left explicit in the task notes

Evidence:
- `cargo test -p codedb-core` passed
- `cargo build -p nu_plugin_codedb -p codedb` passed
- raw log preserved at `/home/flexnetos/Downloads/nu_plugin/logs/CDB014-core.log`
