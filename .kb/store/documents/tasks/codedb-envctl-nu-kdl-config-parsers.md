---
id: 019f221c-6e0a-7b13-b287-1ae3cc0ce3c8
slug: tasks/codedb-envctl-nu-kdl-config-parsers
title: "Parse Yazelix Nu and KDL config rows for reproduction"
type: task
status: completed
priority: medium
tags: [envctl, yazelix, nushell, zellij, kdl, reproduction]
---

## Overview

The first live Yazelix config import proves Nu and KDL config files are visible in envctl `config_files`, but they currently report `parse_status = not_parsed`. That is acceptable for file identity and reproduction staging, but incomplete for the long-term goal where envctl can reproduce files from higher-fidelity table rows.

This task tracks a fix-forward parser upgrade for key Yazelix non-JSON/TOML config formats, starting with `nushell/config/config.nu` and Zellij layout/override KDL files.

## Evidence

- `nushell/config/config.nu` imported with `file_kind = yazelix_nushell_config`, `format = unknown`, `parse_status = not_parsed`.
- `configs/zellij/layouts/flexnetos_agent_workspace.kdl` imported with `file_kind = yazelix_runtime_config`, `format = kdl`, `parse_status = not_parsed`.
- Both files were visible through CodeDB plugin filesystem rows and envctl `config_files`, but not yet decomposed into structured settings rows.

## Acceptance Criteria

- [x] Envctl catalog recognizes `.nu` as Nushell config format.
- [x] Envctl catalog parses at least stable metadata rows for Nushell config files needed for reproduction.
- [x] Envctl catalog parses Zellij KDL layout/config rows or records structured reproduction metadata sufficient to regenerate the exact file.
- [x] Parser failures produce explicit validation rows instead of silent `not_parsed` for supported Yazelix config formats.

## Progress Log

### 2026-07-02

- Filed from the first live CodeDB/envctl Yazelix config import.
- Implemented in envctl source branch `codex/codedb-yazelix-config-catalog`.
- `.nu` files now infer `format = nushell` and produce stable reproduction metadata rows including source/use/plugin lines, env assignment count, line/byte counts, SHA-256, and `reproduction_policy = source_bytes_required`.
- `.kdl` files now produce stable reproduction metadata rows including node names, layout/tab/pane/plugin counts, line/byte counts, SHA-256, and `reproduction_policy = source_bytes_required`.
- Live import proof against `/home/flexnetos/FlexNetOS/src/yazelix`:
  - Tables: 10
  - Rows: 1,656
  - Config files: 58
  - Settings rows: 1,377
  - `nushell/config/config.nu`: `file_kind = yazelix_nushell_config`, `format = nushell`, `parse_status = ok`
  - `configs/zellij/layouts/flexnetos_agent_workspace.kdl`: `file_kind = yazelix_runtime_config`, `format = kdl`, `parse_status = ok`
- Verification passed:
  - `cargo fmt --check`
  - `cargo test -p envctl-engine catalog::tests`
  - `cargo test -p envctl --test cli_contract catalog_repo_root_imports_yazelix_config_without_manifest`
