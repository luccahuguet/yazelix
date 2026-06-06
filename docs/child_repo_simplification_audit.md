# Child Repo Simplification Audit

This audit is the master queue for Yazelix child-repo and delete-first cleanup candidates. Its job is to rank work by ownership removed from the main repo, not by whether a subsystem would look nice in another repository.

Use this with [LOC extraction scorecard](./loc_extraction_scorecard.md) and [Rust code inventory](./rust_code_inventory.md). A candidate that only moves code while leaving the main repo with the same runtime contract, generated mirrors, validators, wrappers, or adapters is a paper extraction and should be rejected or deferred.

## Current Baseline

- Rust budget ceiling: `73,083` raw tracked Rust lines across `140` files
- Long-term Rust hard target: `60,000` raw tracked Rust lines
- Largest pressure families: `core_config_ui_and_materialization` at `20,624` raw lines and `core_workspace_and_pane_integration` at `16,689` raw lines
- Current child repos already integrated: `yazelix-screen`, `yazelix-cursors`, `yazelix-terminal`, `yazelix-ratconfig`, `yazelix-zellij-bar`, `yazelix-zellij-popup`, and `yazelix-yazi-assets`
- Popup lifecycle ownership has moved to `yazelix-zellij-popup`; remaining main-repo popup work should be thin generated-spec/config integration plus Yazelix-specific close hooks

## Ranking Rules

Rank candidates in this order:

1. Delete-only: remove stale runtime, validators, generated files, docs, wrappers, or assets with no replacement
2. Move to child repo: transfer a reusable owner to an existing child repo and delete the duplicated main-repo owner
3. Thin adapter: keep Yazelix-specific settings, Home Manager, session/cache, and runtime paths in the main repo while isolating generic code behind a private boundary or child API
4. Defer: keep the current owner until layout, apply-mode, storage, or runtime behavior is clearer

Reject a move when the main repo would still own the same behavior through a broad adapter, copied generated file, compatibility shim, or validator that recreates the old subsystem.

## Ranked Candidates

| Rank | Candidate | Classification | Main-repo target | Expected impact | Decision |
| ---: | --- | --- | --- | --- | --- |
| 1 | Slim bundled Yazi config and plugin asset pack | Move to child repo / delete main copy | `configs/yazi`, Yazi asset sync, docs copied from upstream plugins/flavors | Main `configs/yazi/` reduced to Yazelix-owned templates/plugins while regular Yazelix still ships all flavors through `yazelix-yazi-assets` | Resolved by `yazelix-lzlg.1`; no flavor removal |
| 2 | Finish yzpp cleanup tail | Delete-only / thin adapter | stale `transient-pane-facts.compute` and `transient_pane_facts.rs` naming | Small LOC change, high ownership clarity; closes stale popup terminology after yzpp extraction | Resolved by `yazelix-g7bs.2`: renamed to `popup-session-facts.compute` and `popup_session_facts.rs` |
| 3 | Move remaining generic status-bar rendering into `yazelix_zellij_bar` | Move to existing child repo | `zellij_materialization.rs`, integrated zjstatus command-definition rendering | Removed `137` raw Rust lines from main while adding `169` Rust lines to `yazelix-zellij-bar`; cache/session ownership stayed local | Resolved by `yazelix-00nz` |
| 4 | Split Yazi materializer into private writer and Yazelix adapter | Thin adapter | `yazi_materialization.rs` | Medium Rust simplification after asset deletion; public extraction remains deferred until adapter is demonstrably thin | Keep `yazelix-lzlg.2` blocked by `yazelix-lzlg.1` |
| 5 | Split launch process execution and desktop/macOS adapters | Thin adapter | `launch_commands.rs` | Mostly organization first; unlocks later workspace extraction but should not claim LOC success unless deletion follows | Keep `yazelix-0nvl.1` |
| 6 | Split restart and enter flow after launch helper extraction | Thin adapter | `launch_commands.rs`, front-door launch dispatch | Same as rank 5; useful only if it shrinks the workspace extraction boundary | Keep `yazelix-0nvl.2` sequenced after `yazelix-0nvl.1` |
| 7 | Evaluate barless/native status component toggle | Defer | Zellij layout families, `zjstatus.wasm`, Home Manager component toggles | Real package/storage impact if accepted, but risky before layout ownership is stable | Keep `yazelix-jhu5` deferred |
| 8 | Public `yazelix-ratconfig` extraction | Move to child repo / thin adapter | `config_ui.rs`, deleted `yazelix_ratconfig/*` staging module | Reusable model/editor/render, JSONC patching, and migration primitives moved to child while Yazelix-specific schema, Home Manager/native status, validation, and apply behavior stayed local | Resolved by `yazelix-ylt4` |
| 9 | Move maintainer tooling to a child repo | Defer / reject now | `yazelix_maintainer`, validators, release/update workflow | Would likely harm maintainer workflow and add cross-repo validator coupling before deleting code | Keep in repo; split validator domains and delete trivia first |
| 10 | Extract `yazelix_workspace` publicly | Defer | `zellij_commands.rs`, `launch_commands.rs`, workspace/session state, pane orchestrator client | Highest theoretical LOC impact, highest coupling | Defer until launch, layout ownership, and Zellij materialization shrinkage land |

## Top Follow-Ups

1. `yazelix-lzlg.1`: resolved by moving reusable Yazi flavors, reusable plugins, Starship config, and pinned upstream metadata into `yazelix-yazi-assets`, deleting the main-repo copies, and keeping Yazelix-only sidebar/editor plugins local.
2. `yazelix-g7bs.2`: top ownership-clarity cleanup. Resolved by renaming the remaining internal helper to `popup-session-facts.compute` and `popup_session_facts.rs`, so the main repo no longer implies that popup lifecycle belongs to the pane orchestrator after `yzpp`.
3. `yazelix-00nz`: resolved by moving generic integrated zjstatus command-definition rendering to `yazelix-zellij-bar` while Yazelix core keeps session/cache/helper ownership.

## 2026-06-06 Second Pass

Second-pass measured snapshot, excluding `.git`, `.beads`, `target`, and `rust_plugins` build outputs:

- `413` tracked files
- `84,352` tokei code lines across counted files
- `74,050` Rust code lines across `157` Rust files
- resolved tracked storage pressure from `configs/zellij/plugins/zjstatus.wasm` by consuming the locked upstream `zjstatus` package artifact instead of a copied main-repo wasm
- README-only media under `assets/` is smaller but currently enters the packaged runtime through a broad `assets/*` symlink in `packaging/mk_runtime_tree.nix`

Fresh follow-ups:

1. `yazelix-audit-deletion-extraction-second-pass-4z0ef.1`: resolved by replacing broad runtime `assets/*` linking with an `assets/icons` allowlist, keeping desktop icon assets live while leaving the README logo and screenshot as repo/docs-only media
2. `yazelix-audit-deletion-extraction-second-pass-4z0ef.2`: reduce config metadata to one semantic source
3. `yazelix-audit-deletion-extraction-second-pass-4z0ef.3`: define the Helix Steel plugin-pack ownership boundary
4. `yazelix-audit-deletion-extraction-second-pass-4z0ef.4`: retire legacy install-artifact repair paths by contract

Second-pass non-candidates:

- Do not create another status-bar extraction bead. Runnable non-workspace widgets, provider probing/cache behavior, CPU/RAM, cursor display, and runtime plugin-block rendering already moved through the closed SP9/status beads; the remaining main adapter is workspace/session path selection and status-bus cache integration.
- Do not move the full Helix managed config surface to `yazelix-helix`. Only the default Steel plugin-pack asset boundary is worth evaluating; managed paths, user plugin selection, and bridge/session behavior remain Yazelix product policy.
- Do not move the full config UI adapter to `yazelix-ratconfig`. The child owns generic UI, patch, and migration primitives; Yazelix owns settings semantics, Home Manager ownership, validation, action metadata, and runtime apply behavior.
- Do not create a public workspace child repo until `zellij_materialization`, launch ownership, and workspace request boundaries are thinner than their current adapters.

## 2026-06-06 Third Pass

Third-pass scope was deliberately narrower than the first two passes: look for runtime payload breadth, stale tracked files, and child-repo validation seams that still make the main repo own too much.

Additional evidence:

- `packaging/mk_runtime_tree.nix` still links broad top-level trees into every runtime: `assets`, `docs`, `nushell`, `shells`, `configs`, and `config_metadata`
- `docs/` is about `1.1 MiB` across `102` files, but runtime code found in this pass reads only `docs/upgrade_notes.toml` and top-level `CHANGELOG.md`
- Removed `assets/font_tests/ubuntu_mono_regular_test.gif` and `assets/tapes/yazelix_v7_quick_demo.tape` after `rg` found no live references to those paths or names; the runtime preview-asset packaging bead remains separate
- `repo_child_release.rs` is about `1,045` lines and validates child package internals through `nix derivation show` markers such as `dontCargoBuild`, `export CARGO=`, `export RUSTC=`, `magick`, and `frame_%03d`

Fresh follow-ups:

1. `yazelix-audit-deletion-extraction-third-pass-c0gmk.1`: resolved by keeping only `docs/upgrade_notes.toml` plus top-level `CHANGELOG.md` in the runtime docs contract instead of linking the whole docs tree
2. `yazelix-audit-deletion-extraction-third-pass-c0gmk.2`: resolved as a decision to replace child-release implementation-detail checks with child-declared package contracts, without deleting the current Darwin regression guard before the child metadata exists
3. `yazelix-audit-deletion-extraction-third-pass-c0gmk.3`: delete unreferenced font-test GIF and old demo tape assets

Child-release validator decision:

- Stable main-owned contracts: first-party child lock entries point at published GitHub revisions; local adjacent child checkouts are clean before a coupled release; first-party Cargo git dependencies have matching fixed-output hashes in `packaging/rust_core_helper.nix`; package outputs expose the stable runtime artifact paths Yazelix consumes
- Transitional implementation-detail checks: `repo_child_release.rs` currently inspects Zellij plugin build phases for `dontCargoBuild`, explicit `CARGO`/`RUSTC`/`PATH`, wasm target preflight, and marker ordering, and inspects the screen package for ImageMagick/magician frame-generation markers
- Target seam: implementation-sensitive claims move to child-owned package metadata or child-owned check outputs. Main should validate declared contract data and publication state, not exact child build recipe strings
- Do not delete the current Darwin wasm and screen/ImageMagick checks until the child repos expose the replacement contract surface or the magician style is deleted. Dropping those guards first would turn a known regression into silent trust
- Follow-up beads now own the actual cuts: add Zellij plugin child contract metadata, consume that metadata in the main validator, and retire the screen/ImageMagick guard after magician deletion removes the live contract

Third-pass non-candidates:

- Do not create another runtime-tool registry split bead. `yazelix-evaluate-runtime-tool-registry-decomposition-pl6g7` already closed with a keep/narrow decision: the registry is still one coherent Nix manifest owner.
- Do not create another runtime preview asset bead. `yazelix-audit-deletion-extraction-second-pass-4z0ef.1` already covers package-runtime exclusion for README-only assets; the new asset bead is only for repo deletion of unreferenced files.
- Do not create another Zellij materialization shrink bead. `yazelix-audit-deletion-extraction-candidates-i0xoh.5` already owns that split; any weak string-assert cleanup belongs there rather than a parallel plan.
- Do not extract maintainer tooling as a whole. The fresh concern is narrower: main should stop asserting child package build recipes when a child-declared artifact contract can carry that evidence.

## Explicit Rejections

- Do not create a standalone Yazi integration repo while the main repo still owns the same materializer paths; the existing `yazelix-yazi-assets` child repo is only the reusable asset package
- Do not move `config_ui.rs` wholesale to `yazelix-ratconfig`; Home Manager/native status, settings metadata, action registry detail text, validation, file writes, and runtime apply behavior are Yazelix-specific
- Do not move status-cache paths or pane-orchestrator payloads to `yazelix_zellij_bar`; provider usage polling and cursor display are child-owned when they run from explicit standalone facts, provider tools, or `yazelix-cursors`
- Do not split maintainer tooling only to call it back through wrappers from this repo; that would make the workflow worse without reducing user runtime ownership
- Do not preserve old popup command/config names that have not been released or that have no current caller; stale aliases are budget debt

## Verification Gate

Each follow-up must report:

- baseline and candidate refs
- main-repo LOC, storage, or generated-clutter delta
- child-repo LOC added, when applicable
- remaining main-repo adapter surface
- accepted modularity costs
- whether the budget ceiling was ratcheted down
