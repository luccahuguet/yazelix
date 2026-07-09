---
id: 019f21ba-f862-76f0-8ce0-17817eb2aae0
slug: tasks/nu-plugin-codedb-runtime-tool-package
title: "Package CodeDB CLI and Nu plugin as runtime tools"
type: task
status: completed
priority: medium
tags: [codedb, nix, packaging, cdb050]
---

# Overview

Package the CodeDB CLI and Nushell plugin as runtime tools in the `/home/flexnetos/Downloads/nu_plugin` source pack.

This task maps source-package task `CDB050` from `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`.

## Source Task

- Task id: `CDB050`
- Title: `Package nu_plugin_codedb as runtime tool`
- Phase: `packaging`
- Depends on: `CDB049`, `CDB030`
- Blocks: `CDB051`, `CDB054`, `CDB061`
- Target surface: runtime libexec/toolbin
- Allowed files: `flake.nix`, `packaging/**`, `docs/CODEDB_YAZELIX_RUNTIME_TOOL.md`
- Raw log: `logs/CDB050-runtime-tool.log`
- Forbidden: global install only
- Gate: runtime package metadata and plugin/CLI smoke output
- Acceptance signal: `plugin/CLI packaged`

## Changes

- Added package-local `flake.nix`.
- Added `packaging/codedb_runtime_tool.nix`.
- Added `docs/CODEDB_YAZELIX_RUNTIME_TOOL.md`.
- Recorded evidence in `logs/CDB050-runtime-tool.log`.

## Package Contract

- `.#default` points to `codedb_runtime_tools`.
- `.#codedb_runtime_tools` installs `bin/codedb` and `bin/nu_plugin_codedb`.
- `.#codedb` and `.#nu_plugin_codedb` are compatibility package names pointing at the same runtime-tool output.
- The derivation writes `share/codedb/runtime-tool-metadata.json`.
- `passthru.runtimeToolMetadata` declares `YAZELIX_CODEDB_BIN=bin/codedb` and `YAZELIX_CODEDB_PLUGIN_BIN=bin/nu_plugin_codedb`.

## Acceptance Criteria

- [x] Nix flake exposes a runtime package for CodeDB CLI and Nu plugin.
- [x] Runtime package installs both `codedb` and `nu_plugin_codedb`.
- [x] Runtime metadata declares both commands and future Yazelix environment variable paths.
- [x] `codedb --version` passes from the package output.
- [x] Plugin binary is executable from the package output.
- [x] Package smoke check passes.
- [x] No global install, profile mutation, Home Manager switch, or Nu plugin registry mutation was performed.

## Verification

Commands run:

```bash
nix flake show /home/flexnetos/Downloads/nu_plugin --no-write-lock-file

nix fmt --no-write-lock-file

nix build /home/flexnetos/Downloads/nu_plugin#codedb_runtime_tools \
  --out-link /tmp/codedb-cdb050-result \
  --no-write-lock-file

/tmp/codedb-cdb050-result/bin/codedb --version
test -x /tmp/codedb-cdb050-result/bin/nu_plugin_codedb

nix build /home/flexnetos/Downloads/nu_plugin#checks.$(nix eval --raw --impure --expr builtins.currentSystem).codedb_runtime_tool_smoke \
  --out-link /tmp/codedb-cdb050-check \
  --no-write-lock-file

nix eval /home/flexnetos/Downloads/nu_plugin#codedb_runtime_tools.passthru.runtimeToolMetadata \
  --json \
  --no-write-lock-file
```

Evidence:

- `codedb --version` output: `0.1.0`.
- Smoke check output contains `codedb-version.txt` with `0.1.0`.
- Smoke check output contains `plugin-path.txt` ending in `bin/nu_plugin_codedb`.
- Passthru metadata contains commands `codedb` and `nu_plugin_codedb`.
- `nix fmt --no-write-lock-file` passes from the package root with the package-local formatter wrapper.

## Notes

Directly executing `nu_plugin_codedb --version` outside Nushell returns the expected `nu-plugin` protocol message that the plugin must be run from within Nushell. CDB050 treats plugin packaging as executable and metadata proof. Nu protocol execution belongs to CDB051/CDB052.
