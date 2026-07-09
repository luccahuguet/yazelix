---
id: 019f2289-7a96-7533-8b4d-830dd7f36ddd
slug: tasks/yazelix-exhaustive-file-target-inventory
title: "Inventory every Yazelix file target across repo Nix system and .local depths"
type: task
status: completed
priority: high
tags: [codedb, envctl, yazelix, inventory, nix, system, local]
---

# Overview

The first envctl catalog pass intentionally loaded repo-owned Yazelix config/settings surfaces only. That is not broad enough for the next CodeDB target: envctl needs an exhaustive, reproducible inventory of every file related to Yazelix across source checkout, Nix outputs, generated runtime targets, system/user service targets, XDG paths, and `.local` depths.

This task defines the discovery work required before importing file contents into envctl tables. It should produce a concrete target inventory with path ownership, source-of-truth classification, mutation policy, and reproduction semantics for each file or file family.

## Goals

- Locate every Yazelix-related file target, not just files under the Yazelix repo root.
- Include repo source files, Nix expressions, flake inputs/locks, packaged Nix store outputs, runtime generated files, systemd/user service targets, XDG config/data/state/cache surfaces, `$META_ROOT/.local/**`, and real-home `.local/**` bridge/adoption paths when relevant.
- Distinguish source-owned files from generated projections, package outputs, runtime state, cache, logs, compatibility bridges, and user-owned files.
- Preserve safety boundaries: do not mutate real home, system paths, Nix store paths, or user-managed configs during inventory.
- Produce a machine-readable inventory that the Nu plugin/envctl import task can consume.

## Acceptance Criteria

- [x] Inventory command or script walks all relevant Yazelix roots:
  - [x] `/home/flexnetos/FlexNetOS/src/yazelix`
  - [x] `/home/flexnetos/FlexNetOS/src/envctl` surfaces that generate or consume Yazelix catalog rows
  - [x] `$META_ROOT` layout paths from envctl/yazelix runtime contracts
  - [x] Nix build outputs and package closures needed to identify packaged Yazelix files
  - [x] XDG config/data/state/cache targets for Yazelix
  - [x] `$META_ROOT/.local/**` and any real-home `.local/**` bridge/adoption targets
  - [x] system/user service and desktop-entry targets that start or supervise Yazelix
- [x] Each inventory row records absolute path, normalized logical path, owner, source-of-truth class, current existence, file kind, parser hint, mutability, reproduction policy, and safety policy.
- [x] Nix targets are resolved with cheap eval/build-info probes first, avoiding unnecessary large builds.
- [x] `.local` and real-home targets are classified as owned, bridge, adopted, ignored, cache/state/log, or unsafe-to-import.
- [x] The inventory identifies files that should be imported as content blobs versus metadata-only rows.
- [x] The inventory identifies gaps in the current envctl catalog coverage, including repo-only assumptions from [[tasks/codedb-envctl-yazelix-config-ingest]].
- [x] The inventory output is committed or rendered to an explicit, reviewable artifact path outside mutable runtime state.
- [x] Verification proves no source, system, Nix store, or real-home path was mutated during discovery.

## Context

- Prior narrower import task: [[tasks/codedb-envctl-yazelix-config-ingest]]
- Nu plugin package task: [[tasks/nu-plugin-codedb-build]]
- Follow-up import task: [[tasks/nu-plugin-envctl-exhaustive-file-content-import]]

The prior live proof showed envctl could import 58 repo-owned Yazelix config files into 10 catalog tables, but it did not cover `/etc`, `/usr`, `$META_ROOT/.local`, real-home `.local`, or generated runtime/deploy targets. This task is the correction point for that missing scope.

## Discovery Notes

### 2026-07-02 read-only target check

Current answer to "are all files from the Nix store and `.local` in envctl?": no. The existing envctl proof from [[tasks/codedb-envctl-yazelix-config-ingest]] imported the repo-owned Yazelix config catalog only: 58 config files, 2,032 settings rows, and 0 `/nix/store` or real-home `.local` target rows. This inventory task and [[tasks/nu-plugin-envctl-exhaustive-file-content-import]] are the active work items that must close that gap.

Read-only discovery found the following target families that are not represented by the current repo-only envctl catalog:

- `/nix/store` has many Yazelix-related immutable package/output classes, including `yazelix`, `yzx`, `yazelix-runtime`, `yazelix-core-0.1.0`, `yazelix-helix-25.7.1`, `yazelix-rust-core-source`, `yazelix-package-source`, `yazelix-runtime-release-contracts`, `yazelix-zellij-pane-orchestrator-0.1.0`, `yazelix_zellij_bar-0.1.0`, `yazelix_yazi_assets-0.1.0`, `yazelix_screen-0.1.0`, `yazelix_cursors-0.1.0`, `mars`, `mars.desktop`, and `mars_terminal_icon.png`. These should be inventory rows with immutable/package-output safety semantics, not write targets.
- `/home/flexnetos/.local/share/yazelix` exists and currently contains 1,331 files. Observed classes include generated runtime configs, Helix/Yazi/Zellij config projections, Bash and Nushell initializer projections, session snapshots, status-bar caches, startup handoff JSON, terminal launch logs, welcome logs, startup profile JSONL, sidebar bootstrap temp files, config overrides, agent usage cache, and upgrade/rebuild state files.
- `/home/flexnetos/.local/share/applications` contains Yazelix desktop-entry targets: `com.flexnetos.Yazelix.Agent.desktop` and `com.yazelix.Yazelix.Mars.desktop`.
- `/home/flexnetos/FlexNetOS/.local` does not currently exist on this machine, so the observed `.local` depth for this pass is real-home `.local`. The inventory still needs to handle `$META_ROOT/.local/**` for envctl-managed workspaces where that bridge exists.

Classification implications:

- Repo source and safe generated config projections can become content/blob rows after the Nu plugin import path is implemented.
- Nix store paths should be exact immutable package evidence, with content imported only when safe and useful; closures and derivations should remain metadata-rich and mutation-prohibited.
- Real-home `.local` runtime state, logs, caches, sessions, and desktop entries need distinct owner/safety labels. Logs, caches, session state, temp files, and user-home bridge/adoption paths should default to metadata-only unless a contract explicitly permits content import.
- The current envctl catalog is therefore a partial projection, not the authoritative exhaustive store. CodeDB/nu_plugin is expected to become the more accurate file/blob store, with envctl projecting tables and later converting safe rows back to files through verifier-gated commands.

## Implementation Notes

- Start with read-only discovery only.
- Prefer repo-local contracts and existing envctl layout/migration code before adding new traversal rules.
- Treat Nix store paths as immutable package evidence, not write targets.
- Treat real-home `.local` as user-owned unless an existing Yazelix/envctl contract proves a bridge or adoption path.
- The output should be suitable for CodeDB blob/file-table semantics: exact bytes where safe, metadata-only where bytes are unsafe or non-reproducible.

## Completion Evidence

Completed on 2026-07-02.

Implemented:

- Added `yazelix_maintainer::repo_yazelix_file_inventory` as the inventory producer.
- Added `yzx_repo_maintainer yazelix-file-inventory --out <path>` as the explicit artifact command.
- Added TDD coverage in `rust_core/yazelix_maintainer/tests/yazelix_file_target_inventory.rs`.
- Rendered the reviewable artifact to `docs/generated/yazelix_file_target_inventory.json`.

Live artifact summary:

- Total inventory rows: 3,549.
- Source-of-truth classes:
  - `repo_source`: 802
  - `envctl_control_surface`: 1,039
  - `nix_store_package_output`: 366
  - `real_home_runtime_state`: 1,335
  - `real_home_user_config`: 5
  - `real_home_desktop_entry`: 2
- Import modes:
  - `content_blob`: 1,909
  - `metadata_only`: 1,640

Commands run:

- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_maintainer --test yazelix_file_target_inventory`
- `cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_maintainer --bin yzx_repo_maintainer -- yazelix-file-inventory --out docs/generated/yazelix_file_target_inventory.json`
- `cargo fmt --manifest-path rust_core/Cargo.toml --all -- --check`
- `cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_maintainer --bin yzx_repo_validator -- validate-rust-test-traceability`
- `cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_maintainer --bin yzx_repo_validator -- validate-package-rust-test-purity`

All commands were run through `nix develop --accept-flake-config .#ci -c ...` because ambient `cargo` was not on `PATH`. The inventory command reported `mutating=false`, and the scanner only reads discovered targets; the sole write is the explicit `--out` artifact path.
