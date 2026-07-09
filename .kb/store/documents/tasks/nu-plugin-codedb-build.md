---
id: 019f20d5-6eb1-7c01-a056-d6e47e6eccc6
slug: tasks/nu-plugin-codedb-build
title: "Import and build Nu plugin codedb execution package"
type: task
status: completed
priority: high
tags: [nu_plugin, nushell, plugin, codedb, yazelix]
---

## Overview

Track `/home/flexnetos/Downloads/nu_plugin` in GitKB, preserve its execution package/task graph, and use the package workflow to build `nu_plugin_codedb`, the Nushell table cockpit for CodeDB.

## Package Inventory

- Package path: `/home/flexnetos/Downloads/nu_plugin`
- Source zip on this host: `/home/flexnetos/Downloads/nu_plugin_codedb_final_execution_package_csv_sot_repaired.zip`
- Package manifest: `/home/flexnetos/Downloads/nu_plugin/manifests/PACK_MANIFEST.json`
- Canonical PRD: `prd/nu_plugin_codedb_v1_1_full_prd.md`
- Task source of truth: `execution/TASK_GRAPH.csv`
- Package validation: `manifests/PACKAGE_VALIDATION.json` reports `passed`
- Checksum scope: 206 package files, excluding the self-hashing manifest/checksum/validation files and final validation log
- Task graph: 69 rows total, all represented in GitKB task documents or this umbrella package task
- Final package zip: `/home/flexnetos/Downloads/nu_plugin_codedb_current_resealed.zip`

## Goal

Deliver `nu_plugin_codedb` V1.1: a Rust-native Nushell plugin plus `codedb` CLI/MCP surface that captures the compiler-observable Rust crate envelope into redb-backed tables, blobs, proof rows, validation errors, and capture gaps.

The intended ownership boundary is: source files and upstream repository checkouts remain the raw input, CodeDB is the accurate structured fact store over those files, and `envctl` is the environment/export bridge that can consume CodeDB rows and materialize files again when needed. In that split, CodeDB is more precise than `envctl` for code/file semantics because it owns table, blob, crate, proof, validation, and gap rows; `envctl` owns target selection and file export semantics.

## Readiness Gate

Before mutating package implementation files:

- [x] Selected task ID from `execution/TASK_GRAPH.csv`: `CDB013`
- [x] Read the package entrypoint, navigation, drift/stop gates, goal, acceptance, architecture, commands, and Nu compatibility docs
- [x] Identified target surface: `code`
- [x] Identified allowed package-relative files: `Cargo.toml;crates/*`
- [x] Identified forbidden action: source overwrite in existing repos
- [x] Identified validation gate: `cargo metadata succeeds`
- [x] Identified raw log path: `logs/CDB013-workspace.log`
- [x] Confirmed no raw secret path is involved for skeleton creation

## Acceptance Criteria

- [x] KB task exists and references the whole downloaded package
- [x] All package CDB tasks are represented below from the canonical CSV
- [x] `CDB013` workspace skeleton exists under the package allowed paths
- [x] `cargo metadata` succeeds for the package workspace
- [x] `logs/CDB013-workspace.log` records the build command/evidence
- [x] `nu_plugin_codedb` builds and returns a table-shaped transient Nu plugin response
- [x] Next implementation task is selected from the CSV after `CDB013` completes
- [x] All canonical CSV task IDs `CDB000` through `CDB068` are represented in GitKB task documents or this umbrella task
- [x] Final package manifests, checksums, validation JSON, and resealed zip were regenerated after the CSV source-of-truth repair
- [x] Full workspace build and test gates pass for the current package

## CDB Task Graph Projection

Source: `/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv`

| Task | Status | Phase | Name | Depends on | Validation gate | Raw log |
| --- | --- | --- | --- | --- | --- | --- |
| CDB000 | complete | package | Initialize execution package |  | all P0 docs listed | logs/CDB000-package-init.log |
| CDB001 | complete | package | Create AI navigation graph | CDB000 | links validate | logs/CDB001-navigation.log |
| CDB002 | complete | package | Create readiness and stop gates | CDB000 | gate checklist covers task/prd/log/secret | logs/CDB002-gates.log |
| CDB003 | complete | package | Create task graph and task-file map | CDB000 | CSV parses and task IDs unique | logs/CDB003-task-graph.log |
| CDB004 | complete | package | Create command ledger and worklog | CDB000 | CSV parses with expected header | logs/CDB004-ledger.log |
| CDB005 | complete | package | Generate manifest, checksums, link report | CDB001;CDB003;CDB004 | checksums match files and links pass | logs/CDB005-manifest.log |
| CDB006 | complete | docs | Write architecture document | CDB005 | covers crates/data flow/runtime modes | logs/CDB006-architecture.log |
| CDB007 | complete | docs | Write schema document | CDB006 | table groups and IDs defined | logs/CDB007-schema.log |
| CDB008 | complete | docs | Write command reference | CDB006 | CLI/Nu/MCP commands documented | logs/CDB008-commands.log |
| CDB009 | complete | docs | Write integration contracts | CDB006 | Codex/Yazelix/meta/envctl/runner covered | logs/CDB009-integration.log |
| CDB010 | complete | docs | Write security and unsafe capture policies | CDB006 | source blob and unsafe gates covered | logs/CDB010-security.log |
| CDB011 | complete | docs | Write compatibility bridge docs | CDB009 | Codex/Nu/Yazelix conflicts bridged | logs/CDB011-bridge.log |
| CDB012 | complete | docs | Write test and fixture matrix | CDB007 | all required fixtures listed | logs/CDB012-tests-docs.log |
| CDB013 | planned | workspace | Create Rust workspace skeleton | CDB006;CDB068 | cargo metadata succeeds | logs/CDB013-workspace.log |
| CDB014 | planned | core | Implement codedb-core schemas | CDB013;CDB007 | unit tests pass | logs/CDB014-core.log |
| CDB015 | planned | store | Implement redb store init | CDB014 | store init/metadata tests pass | logs/CDB015-redb-init.log |
| CDB016 | planned | store | Implement redb schema version, locks, backup, restore | CDB015 | backup restore smoke passes | logs/CDB016-redb-restore.log |
| CDB017 | planned | scan | Implement filesystem scanner | CDB014;CDB015 | fixture scan rows stable | logs/CDB017-fs.log |
| CDB018 | planned | scan | Implement exact source metadata and blob policy | CDB017 | secret policy tests pass | logs/CDB018-source.log |
| CDB019 | planned | cargo | Implement cargo metadata capture | CDB014;CDB015 | cargo metadata fixture passes | logs/CDB019-cargo.log |
| CDB020 | planned | cargo | Implement Cargo source provenance capture | CDB019 | registry/git/path facts captured | logs/CDB020-cargo-sources.log |
| CDB021 | planned | context | Implement cfg/feature/target/toolchain context | CDB019 | context rows deterministic | logs/CDB021-context.log |
| CDB022 | planned | rust-static | Implement static Rust item inventory | CDB018;CDB021 | simple item fixture passes | logs/CDB022-rust-items.log |
| CDB023 | planned | rust-static | Implement macro_rules static inventory | CDB022 | macro fixture passes with gaps where needed | logs/CDB023-macros.log |
| CDB024 | planned | rust-static | Implement proc-macro static detection and gaps | CDB022 | proc macro fixture emits static rows/gaps | logs/CDB024-proc-macro.log |
| CDB025 | planned | rust-static | Implement build.rs static detection and gaps | CDB022 | build script fixture emits static rows/gaps | logs/CDB025-build-static.log |
| CDB026 | planned | rust-static | Implement static include/path edge detection | CDB022 | include fixture passes | logs/CDB026-include.log |
| CDB027 | planned | native | Implement native/linker static/gap rows | CDB025 | native fixture emits rows/gaps | logs/CDB027-native.log |
| CDB028 | planned | proof | Implement no-mutation proof | CDB017 | clean/dirty git fixtures pass | logs/CDB028-no-mutation.log |
| CDB029 | planned | cli | Implement codedb CLI scan/export/schema | CDB015;CDB017;CDB019;CDB022 | JSON/NUON/CSV outputs validate | logs/CDB029-cli.log |
| CDB030 | planned | nu-plugin | Implement Nushell plugin commands | CDB029 | Nu command smoke passes | logs/CDB030-nu-plugin.log |
| CDB031 | planned | doctor | Implement doctor checks | CDB029;CDB030 | doctor reports Nu/Yazelix/Codex status | logs/CDB031-doctor.log |
| CDB032 | planned | mcp | Implement bounded read-only MCP server | CDB029 | MCP page/limit/source guard tests pass | logs/CDB032-mcp.log |
| CDB033 | planned | unsafe | Implement unsafe build capture gate scaffold | CDB025;CDB032 | refuses without unsafe flag | logs/CDB033-unsafe-gate.log |
| CDB034 | planned | unsafe | Implement optional build/proc-macro raw log capture | CDB033 | approved fixture captures logs or gaps | logs/CDB034-build-capture.log |
| CDB035 | planned | exports | Implement envctl export contract | CDB029 | envctl export validates | logs/CDB035-envctl-export.log |
| CDB036 | planned | integration | Implement meta repo selection inputs | CDB029 | meta selected repo scan works | logs/CDB036-meta.log |
| CDB037 | planned | integration | Implement Codex bridge docs and sample MCP config | CDB032 | manual config lint passes | logs/CDB037-codex-bridge.log |
| CDB038 | planned | integration | Implement Yazelix placement docs | CDB031 | host/runtime Nu distinction documented | logs/CDB038-yazelix.log |
| CDB039 | planned | integration | Implement runner proof contract | CDB028;CDB029;CDB032 | runner-readable proof manifest exists | logs/CDB039-runner.log |
| CDB040 | planned | integration | Implement GitKB/RTK/Kache/wild/Fenix docs | CDB009 | facts/export boundaries clear | logs/CDB040-tooling.log |
| CDB041 | planned | fixtures | Create fixture matrix | CDB012;CDB013 | fixtures present and documented | logs/CDB041-fixtures.log |
| CDB042 | planned | tests | Add deterministic scan tests | CDB041;CDB029 | repeat scan checksums stable | logs/CDB042-determinism.log |
| CDB043 | planned | tests | Add security/no-leak tests | CDB041;CDB032 | MCP/source secret tests pass | logs/CDB043-security-tests.log |
| CDB044 | planned | tests | Add no-mutation tests | CDB028;CDB041 | clean/dirty no-mutation tests pass | logs/CDB044-no-mutation-tests.log |
| CDB045 | planned | tests | Add unsafe capture tests | CDB033;CDB034;CDB041 | unsafe capture gate tests pass | logs/CDB045-unsafe-tests.log |
| CDB046 | planned | release | Run full local validation | CDB042;CDB043;CDB044;CDB045 | fmt/clippy/test/doctor pass | logs/CDB046-validation.log |
| CDB047 | planned | release | Generate release manifest | CDB046 | manifest checksums match | logs/CDB047-manifest.log |
| CDB048 | planned | release | Prepare handoff and backlog | CDB047 | capture gaps and MVP2 listed | logs/CDB048-handoff.log |
| CDB049 | planned | yazelix-nu | Inspect Yazelix Nushell runtime bridge | CDB038 | report cites runtime nu/config/initializer boundaries | logs/CDB049-yazelix-nu-bridge.log |
| CDB050 | planned | packaging | Package nu_plugin_codedb as runtime tool | CDB049;CDB030 | runtime tool metadata and `codedb --version` smoke pass | logs/CDB050-runtime-tool.log |
| CDB051 | planned | compat | Validate host Nu vs Yazelix runtime Nu protocol | CDB050 | doctor reports protocol/runtime status and mismatch degrades explicitly | logs/CDB051-nu-protocol.log |
| CDB052 | planned | nu-plugin | Implement transient nu --plugins smoke test | CDB051 | transient plugin command returns table-shaped output | logs/CDB052-transient-plugin.log |
| CDB053 | planned | nu-plugin | Implement temp-HOME plugin registry smoke test | CDB051 | registry test passes in isolated HOME and leaves real HOME unchanged | logs/CDB053-plugin-registry.log |
| CDB054 | planned | yazelix-init | Generate CodeDB extern/init bridge artifact | CDB050;CDB052 | generated initializer has provenance/checksum and does not edit tracked config.nu | logs/CDB054-init-bridge.log |
| CDB055 | planned | provenance | Verify generated initializer checksums/provenance | CDB054 | checksum/provenance manifest validates | logs/CDB055-init-provenance.log |
| CDB056 | planned | syntax | Extend syntax validator path for CodeDB fixtures | CDB054 | temp-HOME syntax validation passes | logs/CDB056-nu-syntax.log |
| CDB057 | planned | safety | Add no-real-HOME plugin registration test | CDB053 | real HOME unchanged before/after | logs/CDB057-no-real-home.log |
| CDB058 | planned | yazelix-smoke | Add Yazelix launch smoke with CodeDB disabled | CDB049;CDB056 | Yazelix launch unaffected without CodeDB | logs/CDB058-yazelix-disabled.log |
| CDB059 | planned | yazelix-smoke | Add Yazelix launch smoke with CodeDB enabled | CDB058;CDB054 | Yazelix launch with CodeDB bridge passes without heavy startup import | logs/CDB059-yazelix-enabled.log |
| CDB060 | planned | security | Add plugin stderr/trace secret-leak guard | CDB052;CDB032 | secret-looking fixtures are not leaked by default | logs/CDB060-plugin-secret-guard.log |
| CDB061 | planned | storage | Add redb lock/plugin-GC behavior test | CDB014;CDB050 | lock contention/GC behavior documented and safe | logs/CDB061-redb-gc.log |
| CDB062 | planned | codex | Add Codex bounded CLI/MCP invocation smoke | CDB032;CDB060 | bounded CLI/MCP output passes limits and exposes no raw source by default | logs/CDB062-codex-bounded.log |
| CDB063 | planned | envctl | Add envctl table rows for CodeDB runtime integration | CDB035;CDB055 | export includes runtime/tool/checksum rows | logs/CDB063-envctl-runtime.log |
| CDB064 | complete | package | Verify ZIP extraction proof before construction | CDB005 | EXTRACTION_PROOF.json parses and source ZIP SHA matches | logs/CDB064-extraction-proof.log |
| CDB065 | complete | package | Upgrade controlled task graph table and Markdown projection | CDB064 | DAG validates, dependencies resolve, all tasks have evidence paths | logs/CDB065-task-graph-final.log |
| CDB066 | complete | package | Complete checklist evidence map | CDB065 | no checklist item is unmapped | logs/CDB066-checklist-completion.log |
| CDB067 | complete | package | Validate and seal final execution package | CDB066 | PACKAGE_VALIDATION.json status is passed | logs/CDB067-final-validation.log |
| CDB068 | complete | package-repair | Repair TASK_GRAPH CSV source-of-truth file linkage | CDB067 | TASK_GRAPH parses; all current artifact references are exact package-relative paths; completed task evidence logs exist; dependency graph remains acyclic; checksums resealed | logs/CDB068-csv-source-of-truth-repair.log |

## Final Execution Audit

Completed on 2026-07-02 after the package task graph ran through `CDB068`.

Coverage:

- CSV task rows: 69
- GitKB CDB IDs found: 69
- Missing CDB IDs: 0
- GitKB task documents: 58, with `CDB000` through `CDB012` represented in this umbrella task and all implementation/package tasks represented in individual task documents or umbrella references

Boundary decision:

- CodeDB owns structured file/code evidence: datatable rows, source blob metadata, redb persistence, Rust/crate facts, proof rows, validation errors, capture gaps, and bounded CLI/Nu/MCP reads
- Nushell native file-to-table behavior is leveraged as a user-facing table idiom, while the plugin supplies the CodeDB-specific schema, storage, provenance, safety, and crate semantics
- `envctl` consumes/export CodeDB table rows for runtime integration and converts rows back to files only at explicit export/materialization boundaries
- Source package files live outside this Yazelix git worktree at `/home/flexnetos/Downloads/nu_plugin`; the Yazelix PR records the GitKB workflow/evidence projection

Final gates:

- `sha256sum -c manifests/CHECKSUMS.sha256` passed for 206 scoped files
- `cargo fmt --check` passed
- `cargo test --workspace` passed
- Fixture generated-lock guard passed with no `fixtures/**/Cargo.lock` files
- `PACKAGE_VALIDATION.json` status: `passed`
- `PACKAGE_VALIDATION.json` rows: 69 task graph rows, 69 task file map rows, 109 checklist items, 0 unmapped checklist items
- CSV source-of-truth repair status: `passed`
- Resealed zip: `/home/flexnetos/Downloads/nu_plugin_codedb_current_resealed.zip`
- Resealed zip SHA-256: `e7eeca0dff54c433dfd3eb81dd02b9b496204a4b737b40b3356ca5c1e63acbd6`
- `unzip -t /home/flexnetos/Downloads/nu_plugin_codedb_current_resealed.zip` reported no compressed data errors

## Initial Execution

`CDB013` completed in this session. The package allows only `Cargo.toml` and `crates/*` for this step, with `cargo metadata` as the validation gate.

Evidence:

- Added Rust workspace skeleton at `/home/flexnetos/Downloads/nu_plugin/Cargo.toml`
- Added required architecture crates under `/home/flexnetos/Downloads/nu_plugin/crates/`
- Built `target/debug/nu_plugin_codedb` and `target/debug/codedb`
- Preserved raw validation/build log at `/home/flexnetos/Downloads/nu_plugin/logs/CDB013-workspace.log`
- Smoke-tested transient Nu plugin loading with `nu --plugins /home/flexnetos/Downloads/nu_plugin/target/debug/nu_plugin_codedb -c 'codedb tables | to json'`

Next package task by dependency order: `CDB014` (`Implement codedb-core schemas`).
