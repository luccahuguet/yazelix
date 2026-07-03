---
id: 019f21c4-ff63-7902-adfb-cf57b13d97a2
slug: tasks/nu-plugin-codedb-nu-protocol-compat
title: "Validate CodeDB host and Yazelix Nu protocol compatibility"
type: task
status: completed
priority: medium
tags: [codedb, nushell, protocol, cdb051]
---

# Overview

Validate that `codedb doctor --nu --yazelix` reports host Nu and Yazelix runtime Nu compatibility without assuming both runtimes are identical.

This task maps source-package task `CDB051` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`.

## Source Task

- Task id: `CDB051`
- Title: `Validate host Nu vs Yazelix runtime Nu protocol`
- Phase: `compat`
- Depends on: `CDB050`
- Blocks: `CDB052`, `CDB053`
- Target surface: host Nu + Yazelix Nu
- Allowed files: `crates/codedb/**`, `docs/NUSHELL_PLUGIN_COMPAT.md`
- Raw log: `logs/CDB051-nu-protocol.log`
- Forbidden: assuming protocol equality
- Gate: doctor output and protocol status row
- Acceptance signal: `Nu compatibility gate exists`

## Changes

- Updated `crates/codedb/src/main.rs` so Yazelix runtime Nu discovery prefers `YAZELIX_NU_BIN`.
- Updated `docs/NUSHELL_PLUGIN_COMPAT.md` with the Yazelix Nu variable lookup order and degraded behavior.
- Recorded validation in `logs/CDB051-nu-protocol.log`.

## Acceptance Criteria

- [x] `codedb doctor --nu` reports host Nu path/version/protocol rows.
- [x] `codedb doctor --yazelix` reports a clear degraded row when explicit Yazelix Nu env vars are missing.
- [x] `codedb doctor --nu --yazelix` reports separate `host_nu` and `yazelix_nu` components when `YAZELIX_NU_BIN` is set.
- [x] Protocol compatibility row is explicit and does not assume equality silently.
- [x] Doctor does not mutate or inspect the user's Nu plugin registry.

## Verification

Commands run from `/home/flexnetos/Downloads/nu_plugin`:

```bash
PATH=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin:$PATH cargo fmt --check

PATH=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin:$PATH cargo test -p codedb

env -u YAZELIX_NU_BIN -u YAZELIX_NU_PATH -u YAZELIX_RUNTIME_NU -u YZX_NU -u YAZELIX_TOOLBIN \
  PATH=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin:$PATH \
  cargo run -p codedb -- doctor --yazelix --format json

NU_BIN=$(command -v nu)
PATH=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin:$PATH \
  YAZELIX_NU_BIN="$NU_BIN" \
  cargo run -p codedb -- doctor --nu --yazelix --format json
```

Evidence:

- Missing runtime env output includes `component=yazelix_nu`, `check=runtime_nu_path`, `status=degraded`.
- Explicit `YAZELIX_NU_BIN` output includes both `host_nu` and `yazelix_nu`.
- Explicit `YAZELIX_NU_BIN` output includes `nu_path`, `nu_version`, `plugin_protocol_compatibility`, `plugin_binary_path`, and `plugin_registration_status` rows.
- Observed Nu version: `0.112.2`.
- Protocol row status: `available`.
