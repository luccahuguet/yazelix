---
id: 019f21cd-487c-75f0-9d09-5981ddf51b0a
slug: tasks/nu-plugin-codedb-yazelix-init-bridge
title: "Generate CodeDB Yazelix init and extern bridge artifact"
type: task
status: completed
priority: medium
tags: [codedb, yazelix, generated-state, cdb054]
---

# Overview

Generate CodeDB's Yazelix Nushell init/extern bridge artifacts as declared generated state, with provenance and checksums.

This task maps source-package task `CDB054` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`.

## Source Task

- Task id: `CDB054`
- Title: `Generate CodeDB extern/init bridge artifact`
- Phase: `yazelix-init`
- Depends on: `CDB050`, `CDB052`
- Blocks: `CDB055`, `CDB056`, `CDB059`
- Target surface: generated initializers
- Allowed files: `crates/codedb/**`, `templates/nushell/**`, `docs/CODEDB_YAZELIX_INIT_CONTRACT.md`
- Raw log: `logs/CDB054-init-bridge.log`
- Forbidden: editing tracked `config.nu`
- Gate: generated init/extern checksums
- Acceptance signal: `init bridge generated`

## Changes

- Added `codedb generate-yazelix-bridge --out-dir <path>`.
- Added `templates/nushell/codedb_init.nu`.
- Added `templates/nushell/codedb_extern.nu`.
- Added `docs/CODEDB_YAZELIX_INIT_CONTRACT.md`.
- Recorded evidence in `logs/CDB054-init-bridge.log`.

## Acceptance Criteria

- [x] Generator writes `codedb_init.nu`.
- [x] Generator writes `codedb_extern.nu`.
- [x] Generator writes `codedb_bridge_manifest.json`.
- [x] Generator emits artifact rows with SHA-256 checksums and provenance.
- [x] Generated files do not run `plugin add` or mutate plugin registries.
- [x] No tracked Yazelix `nushell/config/config.nu` edit is made.
- [x] Generated Nu files pass `nu --no-config-file --ide-check 100`.

## Verification

Commands run from `/home/flexnetos/Downloads/nu_plugin`:

```bash
PATH=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin:$PATH cargo fmt --check

PATH=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin:$PATH cargo test -p codedb

PATH=/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin:$PATH \
  cargo run -p codedb -- generate-yazelix-bridge \
  --out-dir /tmp/codedb-cdb054.FeH4Rr \
  --format json

nu --no-config-file --ide-check 100 /tmp/codedb-cdb054.FeH4Rr/codedb_init.nu
nu --no-config-file --ide-check 100 /tmp/codedb-cdb054.FeH4Rr/codedb_extern.nu
```

Evidence:

- `codedb_init` sha256: `d769c2bdca562fc53979cb3bad0fd7940f12c2f55fdcc65bf6b79f35f3987797`.
- `codedb_extern` sha256: `203233735d97fa16eb19d4b96efb2d5f44cb9114163724266e6d35275b41e7e0`.
- `codedb_bridge_manifest` sha256: `2364c2aeeab041cf30ab7d963b3235b9a8c97da20d9b74d78d2969f026e4bc07`.
- Generated files: `codedb_init.nu`, `codedb_extern.nu`, `codedb_bridge_manifest.json`.
- `codedb_init.nu` syntax check exited 0 with hints only.
- `codedb_extern.nu` syntax check exited 0.
