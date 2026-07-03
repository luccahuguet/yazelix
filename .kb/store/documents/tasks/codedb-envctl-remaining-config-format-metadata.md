---
id: 019f2225-3578-7cf2-9d84-15e8cdfa16a9
slug: tasks/codedb-envctl-remaining-config-format-metadata
title: "Add reproduction metadata for remaining Yazelix config formats"
type: task
status: completed
priority: medium
tags: [envctl, yazelix, reproduction, nix, lua, config]
---

## Overview

After the Nu/KDL parser upgrade, the live Yazelix envctl import still has 35 visible config files with `parse_status = not_parsed`. These files are loaded into `config_files` with file identity and hashes, but they do not yet emit table rows with reproduction metadata.

This task tracks the next fix-forward parser/metadata slice for remaining Yazelix config families. CodeDB remains the higher-fidelity file/blob/fact store; envctl should expose honest rows that can drive later reconstruction without pretending every format has a complete AST parser.

## Acceptance Criteria

- [x] Envctl catalog recognizes Nix files as `format = nix` and emits stable reproduction metadata rows.
- [x] Envctl catalog recognizes Lua files as `format = lua` and emits stable reproduction metadata rows.
- [x] Envctl catalog recognizes terminal `.conf` files as config text and emits stable reproduction metadata rows.
- [x] Envctl catalog records explicit Markdown/flake-lock reproduction metadata or documents why those files should remain file-identity-only.
- [x] Live Yazelix import either reduces `not_parsed` to zero for identified config/settings files or documents each remaining exception with a more specific blocking task.

## Evidence

Live audit after Nu/KDL upgrade on 2026-07-02:

- Import summary: 10 tables, 1,656 rows, 58 config files, 1,377 settings rows, 44 env vars, non-mutating.
- `nushell/config/config.nu` and `configs/zellij/layouts/flexnetos_agent_workspace.kdl` now parse as `ok`.
- Remaining `not_parsed` count: 35.
- Remaining families include:
  - `configs/terminal_emulators/kitty/kitty.conf`
  - `configs/terminal_emulators/wezterm/.wezterm.lua`
  - `configs/yazi/plugins/*/main.lua`
  - `flake.nix` and `flake.lock`
  - `home_manager/**/*.nix`
  - `packaging/**/*.nix`
  - `home_manager/README.md`
  - `nushell/scripts/README.md`

## Progress Log

### 2026-07-02

- Created before continuing parser implementation because these are visible config/settings files that remain loaded only as file identity rows.
- Implemented in envctl source branch `codex/codedb-yazelix-config-catalog`.
- Added conservative reproduction metadata for `nix`, `lua`, `terminal_conf`, and `markdown` formats.
- Treated `flake.lock` as JSON so it imports through the structured JSON parser instead of file-identity-only metadata.
- Live import proof against `/home/flexnetos/FlexNetOS/src/yazelix`:
  - Tables: 10
  - Rows: 2,307
  - Config files: 58
  - Settings rows: 2,028
  - Env vars: 44
  - Remaining `not_parsed` config files: 0
  - Selected rows parse as `ok`: `flake.lock`, `flake.nix`, `home_manager/module.nix`, `packaging/mk_runtime_tree.nix`, `configs/terminal_emulators/kitty/kitty.conf`, and `configs/terminal_emulators/wezterm/.wezterm.lua`.
- Render proof:
  - `/tmp/yazelix-envctl-catalog-render/catalog/tables/config_files.json`: 58 rows
  - `/tmp/yazelix-envctl-catalog-render/catalog/tables/settings.json`: 2,028 rows
  - `/tmp/yazelix-envctl-catalog-render/dashboard/mission-control.catalog.kdl`: 2,077 path+settings rows
- Verification passed:
  - `cargo fmt --check`
  - `cargo test -p envctl-engine catalog::tests`
  - `cargo test -p envctl --test cli_contract catalog_repo_root_imports_yazelix_config_without_manifest`
