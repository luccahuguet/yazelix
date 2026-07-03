---
id: 019f2289-7aaa-7d61-a7a1-01cb2e7d6598
slug: tasks/nu-plugin-envctl-exhaustive-file-content-import
title: "Import exhaustive Yazelix file contents into envctl tables through Nu plugin"
type: task
status: completed
priority: high
tags: [codedb, envctl, nu_plugin, nushell, yazelix, blobs, datatables]
---

# Overview

After every Yazelix-related file target is inventoried, CodeDB's Nu plugin should import the safe file contents into envctl-visible tables. The goal is not only path discovery: envctl needs table rows rich enough to reproduce files later, while CodeDB preserves blob/content semantics and native Nushell file-to-table behavior where appropriate.

This task turns the exhaustive Yazelix target inventory into envctl tables, using the Nu plugin as the ingestion engine and envctl as the projection/render/reproduction layer.

## Goals

- Use the Nu plugin from `/home/flexnetos/Downloads/nu_plugin` or its packaged CodeDB runtime location to import Yazelix-related files into envctl tables.
- Preserve exact file bytes for safe import targets with blob hashes and reversible content metadata.
- Use metadata-only rows for unsafe, generated, immutable, cache/log/state, real-home, or non-reproducible targets.
- Leverage Nushell native file-to-table capability for structured formats instead of ad hoc parsing when the plugin can do so safely.
- Make the imported rows visible through envctl table/render/dashboard surfaces.
- Keep envctl responsible for converting table/blob rows back to files when an explicit verifier-gated apply path exists.

## Acceptance Criteria

- [x] Depends on completed inventory from [[tasks/yazelix-exhaustive-file-target-inventory]].
- [x] Nu plugin command imports every inventory row into envctl tables or records a precise skipped/metadata-only reason.
- [x] Table schema includes at minimum:
  - [x] target id and logical owner
  - [x] absolute path and normalized path
  - [x] source-of-truth class
  - [x] file kind / parser hint
  - [x] content hash and byte length
  - [x] blob reference or inline safe structured value
  - [x] import safety policy
  - [x] reproduction policy
  - [x] last observed/provenance fields
- [x] Structured files are converted to datatable rows where safe: Nix, TOML, JSON/JSONC, KDL, Nu, Lua, YAML, Markdown, service files, desktop entries, shell fragments, and plain config formats.
- [x] Binary or unsafe files are represented with blob metadata and not lossy text decoding.
- [x] `.local`, real-home, Nix store, system, generated, cache/state/log, and source files retain distinct safety semantics in envctl.
- [x] envctl render/import output proves rows are visible from app/dashboard/table surfaces.
- [x] A no-mutation proof shows the plugin import did not write to source, system, Nix store, real-home, or runtime-owned targets.
- [x] Round-trip/reproduction planning identifies which rows can be converted back to files now and which require additional verifier-gated tooling.

## Context

- Inventory producer: [[tasks/yazelix-exhaustive-file-target-inventory]]
- Prior repo-only envctl import: [[tasks/codedb-envctl-yazelix-config-ingest]]
- CodeDB Nu plugin package: [[tasks/nu-plugin-codedb-build]]
- CodeDB envctl export contract: [[tasks/nu-plugin-codedb-envctl-export]]

The user expectation is that CodeDB is the more accurate store: it should preserve file/blob semantics and richer Rust/crate/package layers, while envctl can project tables and eventually reproduce files through explicit apply tooling.

## Implementation Notes

- Do not replace CodeDB blob semantics with envctl-only flattened rows.
- Do not silently read real-home or system secrets into content blobs. Use metadata-only rows when safety is uncertain.
- Prefer native Nushell parsing/table conversion for supported file formats.
- Keep all writes to explicit output/catalog/plugin database paths.
- Any discovered import gaps should create additional GitKB tasks before implementation proceeds.

## Progress Notes

### 2026-07-02

- Inventory dependency is implemented by [[tasks/yazelix-exhaustive-file-target-inventory]].
- The import input artifact is `docs/generated/yazelix_file_target_inventory.json`.
- Current artifact summary:
  - 3,549 inventory rows
  - 1,909 `content_blob` candidates
  - 1,640 `metadata_only` rows
  - source-of-truth classes include `repo_source`, `envctl_control_surface`, `nix_store_package_output`, `real_home_runtime_state`, `real_home_user_config`, and `real_home_desktop_entry`
- Next implementation loop should add a Nu plugin or CodeDB CLI command that consumes that artifact and emits envctl-visible rows with hashes/blob references for `content_blob` candidates and precise skip reasons for `metadata_only` rows.

Durable CodeDB plugin progress:

- Created durable repo `https://github.com/FlexNetOS/nu_plugin` from the execution package at `/home/flexnetos/FlexNetOS/src/nu_plugin`.
- Initial repo commit: `8a05df1 Initial CodeDB Nushell plugin package`.
- Added `nu_plugin_codedb` command: `codedb envctl import inventory <inventory_path>`.
- Added TDD coverage in `crates/nu_plugin_codedb/src/main.rs` for content-blob hashing and metadata-only skip reasons.
- The command emits `envctl_yazelix_file_import` rows with target id, logical owner, absolute path, normalized path, source-of-truth class, file kind, parser hint, content hash, byte length, blob ref, safety policy, reproduction policy, import mode, import status, skip reason, and provenance fields.
- Verification commands passed from `/home/flexnetos/FlexNetOS/src/nu_plugin` through the Yazelix CI shell:
  - `cargo fmt --all -- --check`
  - `cargo test -p nu_plugin_codedb envctl_inventory_import_rows_hash_content_and_skip_metadata_only`
  - `cargo build --quiet -p nu_plugin_codedb`
  - `nu --no-config-file --plugins target/debug/nu_plugin_codedb -c 'let rows = (codedb envctl import inventory /home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json); {rows: ($rows | length), blob_ready: ($rows | where import_status == blob_metadata_ready | length), metadata_only: ($rows | where import_status == metadata_only | length), tables: ($rows | get table | uniq)} | to json'`
- Full import smoke result:
  - `rows`: 3,549
  - `blob_ready`: 1,874
  - `metadata_only`: 1,675
  - `tables`: `envctl_yazelix_file_import`
- Structured import follow-up:
  - Plugin commit: `4756713 plugin: add structured envctl inventory rows`
  - Plugin PR: `https://github.com/FlexNetOS/nu_plugin/pull/1`
  - Added native `structured_rows` payloads, `structured_status`, `structured_row_count`, and `last_observed`.
  - JSON/JSONC rows are flattened into key/value records.
  - Safe text/config formats are exposed as deterministic line/key records for Nix, TOML, KDL, Nu, Lua, YAML, Markdown, desktop/service/shell/plain config-like files.
  - Unsafe or metadata-only targets keep empty structured payloads and remain blob/metadata rows rather than lossy decoded text.
- Structured plugin verification:
  - `cargo test -p nu_plugin_codedb envctl_inventory_import_rows -- --nocapture`
  - `cargo fmt --all -- --check`
  - Real Yazelix smoke: 3,549 rows, 1,874 blob-ready, 1,675 metadata-only, 1,427 structured-ready rows, 336,937 structured payload rows, all rows carrying `unix:` `last_observed`.

Envctl catalog progress:

- Envctl worktree: `/home/flexnetos/FlexNetOS/src/envctl-codedb-file-import`.
- Envctl commit: `fc6d074 catalog: import Yazelix CodeDB file inventory`.
- Envctl PR: `https://github.com/FlexNetOS/envctl/pull/410`.
- Added read-only envctl catalog table `codedb_file_imports`, also accepting aliases `codedb-file-imports` and `envctl_yazelix_file_import`.
- The table consumes `docs/generated/yazelix_file_target_inventory.json` from a Yazelix repo root and preserves CodeDB blob semantics:
  - `content_blob` rows read current bytes only when readable and emit SHA-256 `content_hash` plus `sha256:<hash>` `blob_ref`.
  - `metadata_only` rows do not read file contents and keep safety policy as `skip_reason`.
  - Nix store rows stay metadata-only in envctl.
  - `.local`/real-home rows keep distinct real-home classes and are metadata-only unless the inventory marks a row `content_blob`.
- Envctl verification passed:
  - `cargo test -p envctl-engine scan_imports_yazelix_config_files_without_manifest -- --nocapture`
  - `cargo test -p envctl-engine render_imports_yazelix_config_files_without_manifest -- --nocapture`
  - `cargo test -p envctl --test cli_contract catalog_repo_root_imports_yazelix_codedb_file_inventory -- --nocapture`
  - `cargo fmt --all -- --check`
- Envctl structured follow-up:
  - Envctl commit: `4d8ce75 catalog: expose structured CodeDB inventory rows`
  - Envctl PR comment: `https://github.com/FlexNetOS/envctl/pull/410#issuecomment-4865520699`
  - Added `structured_table`, `structured_status`, `structured_row_count`, `structured_rows`, and `last_observed` to `codedb_file_imports`.
  - Real Yazelix catalog smoke: 3,549 rows, 1,403 structured-ready rows, 324,835 structured payload rows, all rows with ISO-like `last_observed`, 366 Nix store rows metadata-only.
  - Render proof: `catalog/tables/codedb_file_imports.json` generated with 3,549 rows and 1,403 structured-ready rows.
- Real-data envctl smoke against `/home/flexnetos/FlexNetOS/src/yazelix`:
  - `envctl_control_surface`: 1,039 rows, 1,022 blob-ready, 17 metadata-only
  - `nix_store_package_output`: 366 rows, 0 blob-ready, 366 metadata-only
  - `real_home_desktop_entry`: 2 rows, 0 blob-ready, 2 metadata-only
  - `real_home_runtime_state`: 1,335 rows, 68 blob-ready, 1,267 metadata-only
  - `real_home_user_config`: 5 rows, 0 blob-ready, 5 metadata-only
  - `repo_source`: 802 rows, 784 blob-ready, 18 metadata-only
  - `.local` subset: 1,337 rows across `real_home_runtime_state` and `real_home_desktop_entry`

No-mutation evidence:

- Hermetic plugin regression:
  - Plugin commit: `df03c96 plugin: prove inventory import is read-only`
  - Plugin PR comment: `https://github.com/FlexNetOS/nu_plugin/pull/1#issuecomment-4865547276`
  - Test: `envctl_inventory_import_rows_do_not_mutate_targets`
  - The test snapshots a source-like content blob file and a real-home-like runtime `status_bar_cache.json` file before and after `envctl_inventory_import_rows`, includes a metadata-only Nix-store row, and proves source/runtime bytes and metadata are unchanged.
  - Verification: `cargo test -p nu_plugin_codedb envctl_inventory_import_rows -- --nocapture` ran 3 tests including the no-mutation regression.
- Full stat proof over all 3,549 inventory target paths was attempted twice and failed both times on the same live runtime-owned file pattern:
  - `/home/flexnetos/.local/share/yazelix/sessions/1782960525907993465/status_bar_cache.json`
  - Only runtime cache file metadata changed during the proof window, on a 30-second cadence, consistent with a live Yazelix status writer.
- Scoped stat proof excluding `*/status_bar_cache.json` passed:
  - `NO_MUTATION_NON_VOLATILE_STAT_MATCH=1`
  - `NON_VOLATILE_TARGETS=3527`
  - `PLUGIN_ROWS=3549`
- The live all-target stat proof remains noisy until a quiesced Yazelix runtime/manual gate can suppress the 22 volatile status-bar cache rows, but the hermetic regression proves the plugin import path itself is read-only for source, real-home-like runtime, and metadata-only Nix-store rows.

Round-trip/reproduction policy:

- Rows with `import_status = blob_metadata_ready`, a `blob_ref`, and `reproduction_policy` such as `git_checkout`, `user_config_source_or_import`, or source-controlled package/config policies can be converted back to files once envctl gains an explicit verifier-gated apply path.
- Rows with `structured_status = structured_rows_ready` can provide table-level inspection and merge/review data, but the raw blob hash remains the byte-exact reproduction anchor.
- `metadata_only` rows, Nix store package outputs, real-home runtime state, generated/cache/log rows, and unsafe/opaque rows require regeneration from their owner (`nix_realise`, runtime re-observation, package build, or explicit user import) rather than CodeDB writing file bytes back directly.
- Envctl `catalog sync --apply` still correctly refuses pending verifier-gated row edit/apply support; reproduction is planned, not enabled as an implicit write path.

Default-branch landing:

- `https://github.com/FlexNetOS/nu_plugin/pull/1` merged on 2026-07-02 at `55b5ff0140531fb80b980ee774e69cc92fb4d286`.
- `https://github.com/FlexNetOS/envctl/pull/410` merged on 2026-07-02 at `52100614ab4666e6abab52a3292d0149351a9453`.

## Completion Evidence

- All acceptance criteria above are checked.
- CodeDB Nu plugin durable repo exists at `https://github.com/FlexNetOS/nu_plugin`.
- Envctl default branch includes `codedb_file_imports` table import/render support through merged PR #410.
- Plugin default branch includes structured inventory rows and hermetic no-mutation regression through merged PR #1.
- GitKB evidence records focused TDD tests, fmt checks, real Yazelix import/render smokes, no-mutation proof, and round-trip/reproduction policy.
