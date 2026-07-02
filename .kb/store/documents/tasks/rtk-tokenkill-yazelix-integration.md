---
id: 019f20cb-0458-7720-a283-ee08a19b67be
slug: tasks/rtk-tokenkill-yazelix-integration
title: "Require RTK TokenKill in Yazelix agent sessions"
type: task
status: completed
priority: high
tags: [rtk, tokenkill, yazelix, meta, agent-commands]
---

# Overview

FlexNetOS RTK TokenKill is the missing runtime piece for Yazelix agent session execution. `FlexNetOS/rtk-tokenkill` must be physically present on disk, registered in the FlexNetOS meta project graph or inventory, and wired into Yazelix so any Codex/agent session launched inside Yazelix uses the approved RTK TokenKill integration instead of assuming an ambient optional `rtk` binary.

This task also keeps the `~/workspace` execution packet aligned with the implementation so future agents can pick up the same requirement from the packet CSVs and task note.

## Goals

- Locate or create the local `FlexNetOS/rtk-tokenkill` source checkout without unnecessary cloning
- Register the project in FlexNetOS meta using the existing meta inventory format
- Wire Yazelix agent command generation/runtime paths to invoke RTK TokenKill where required
- Ensure any Codex session launched inside Yazelix gets the RTK policy/instruction context, not only the initial `rtk codex` process wrapper
- Keep raw logs/fail-fast behavior intact; RTK summaries must not hide failures
- Update the `~/workspace/flexnetos_production_execution_pack` task packet with this requirement

## Acceptance Criteria

- [x] `FlexNetOS/rtk-tokenkill` exists locally at the expected workspace path and its branch/HEAD are recorded
- [x] FlexNetOS meta can discover/query `rtk-tokenkill` as a registered project
- [x] Yazelix agent command surfaces route through the approved RTK TokenKill integration for all agent commands
- [x] Yazelix verification proves generated/runtime config contains the RTK TokenKill wiring without PATH-only local fixes
- [x] `~/workspace/flexnetos_production_execution_pack/TASK_FILE_MAP.csv` references the RTK TokenKill task note
- [x] `~/workspace/flexnetos_production_execution_pack/execution_artifacts/revised_task_table.csv` contains the concrete RTK TokenKill integration task row
- [x] Yazelix-generated Codex/agent session context explicitly requires shell commands to use `rtk` by default, with raw-output exemptions only for verification/root-cause evidence
- [x] Runtime/session verification proves the generated Yazelix session context includes the RTK requirement

## Context

- User requirement: "FlexNetOS/rtk-tokenkill must be present on disk, registered in meta, and properly wired into yazelix for all agent commands"
- Packet note: `/home/flexnetos/workspace/flexnetos_production_execution_pack/execution_artifacts/rtk_tokenkill_integration_task.md`
- Packet task row: `T238` in `/home/flexnetos/workspace/flexnetos_production_execution_pack/execution_artifacts/revised_task_table.csv`
- Existing Yazelix AGENTS rules apply: no local-only host fixes, fail fast, avoid hiding root causes, and use project-owned config/runtime surfaces
- User clarification: "any session run in yazelix must use rtk"

## Progress Log

### 2026-07-01

- Reopened task after user clarified that any Codex/agent session run inside Yazelix must use RTK, not only the initial `yzx agent` launcher
- Added runtime session markers `YAZELIX_RTK_REQUIRED=true` and `YAZELIX_CODEX_COMMAND=rtk codex`
- Added generated shell initializer policy for Nushell, Bash, Fish, Zsh, and Xonsh so direct `codex` launches become `rtk codex`
- Added render-plan normalization so a direct Codex right-sidebar command is rendered as `rtk codex ...`
- Cloned `FlexNetOS/rtk-tokenkill` to `/home/flexnetos/FlexNetOS/src/rtk-tokenkill`
- Registered `rtk-tokenkill` in `/home/flexnetos/FlexNetOS/src/meta/.meta.yaml` with path `../rtk-tokenkill`
- Added `rtk-tokenkill/` to `/home/flexnetos/FlexNetOS/src/meta/.gitignore` as a child checkout boundary
- Updated `/home/flexnetos/FlexNetOS/src/release-workspace.meta.yaml` so `rtk-tokenkill` is checked out and the task/meta hashes are current
- Updated Yazelix `yzx agent` so managed Codex sessions launch as `rtk codex`, with a fail-fast `missing_rtk_tokenkill` error if Codex exists but RTK is absent
- Updated the workspace packet note and marked packet task `T238` done
- Created this GitKB task before implementation per `.kb/AGENTS.md`
- Verified the KB has no pre-existing context documents or task documents; this task is the active execution context

## Completion Evidence

- Local checkout: `/home/flexnetos/FlexNetOS/src/rtk-tokenkill`, branch `develop`, HEAD `d1f1bdc4389d9bc1cf22709985eafd71ba136862`
- Meta query: `/home/flexnetos/FlexNetOS/usr/bin/meta project list --json` lists `rtk-tokenkill`
- Meta check: `/home/flexnetos/FlexNetOS/usr/bin/meta project check` reports all projects are cloned and present
- Yazelix tests: `nix develop --accept-flake-config .#ci -c cargo test --manifest-path rust_core/yazelix_core/Cargo.toml agent -- --test-threads=1` passed
- Formatting: `nix develop --accept-flake-config .#ci -c cargo fmt --manifest-path rust_core/Cargo.toml --all -- --check` passed
- Runtime-facing help: `nix develop --accept-flake-config .#ci -c cargo run --quiet --manifest-path rust_core/yazelix_core/Cargo.toml --bin yzx_control -- agent --help` reports `rtk codex`
- Yazelix RTK session tests: `nix develop --accept-flake-config .#ci -c cargo test --manifest-path rust_core/yazelix_core/Cargo.toml rtk -- --test-threads=1` passed
- Zellij render-plan test: `nix develop --accept-flake-config .#ci -c cargo test --manifest-path rust_core/Cargo.toml -p yazelix_zellij_config_pack wraps_direct_codex_right_sidebar_with_rtk_tokenkill -- --test-threads=1` passed
