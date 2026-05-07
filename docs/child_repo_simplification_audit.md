# Child Repo Simplification Audit

This audit is the master queue for Yazelix child-repo and delete-first cleanup candidates. Its job is to rank work by ownership removed from the main repo, not by whether a subsystem would look nice in another repository.

Use this with [LOC extraction scorecard](./loc_extraction_scorecard.md) and [Rust code inventory](./rust_code_inventory.md). A candidate that only moves code while leaving the main repo with the same runtime contract, generated mirrors, validators, wrappers, or adapters is a paper extraction and should be rejected or deferred.

## Current Baseline

- Rust budget ceiling: `73,513` raw tracked Rust lines across `140` files
- Long-term Rust hard target: `60,000` raw tracked Rust lines
- Largest pressure families: `core_config_ui_and_materialization` at `20,624` raw lines and `core_workspace_and_pane_integration` at `16,689` raw lines
- Current child repos already integrated: `yazelix-screen`, `yazelix-cursors`, `yazelix-bar`, and `yazelix-zellij-popup`
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
| 1 | Slim bundled Yazi config and plugin asset pack | Delete-only | `configs/yazi`, Yazi asset sync, docs copied from upstream plugins/flavors | Runtime storage and generated clutter reduction; low Rust LOC impact | Do `yazelix-lzlg.1` before any Yazi public extraction |
| 2 | Finish yzpp cleanup tail | Delete-only / thin adapter | stale `transient-pane-facts.compute` and `transient_pane_facts.rs` naming | Small LOC change, high ownership clarity; closes stale popup terminology after yzpp extraction | Resolved by `yazelix-g7bs.2`: renamed to `popup-session-facts.compute` and `popup_session_facts.rs` |
| 3 | Move remaining generic status-bar rendering into `yazelix_bar` | Move to existing child repo | `zellij_materialization.rs`, integrated zjstatus command-definition rendering | Likely `300-900` Rust lines if generic placeholders and command KDL move without duplicating Yazelix cache/session ownership | Created `yazelix-00nz` |
| 4 | Split Yazi materializer into private writer and Yazelix adapter | Thin adapter | `yazi_materialization.rs` | Medium Rust simplification after asset deletion; public extraction remains deferred until adapter is demonstrably thin | Keep `yazelix-lzlg.2` blocked by `yazelix-lzlg.1` |
| 5 | Split launch process execution and desktop/macOS adapters | Thin adapter | `launch_commands.rs` | Mostly organization first; unlocks later workspace extraction but should not claim LOC success unless deletion follows | Keep `yazelix-0nvl.1` |
| 6 | Split restart and enter flow after launch helper extraction | Thin adapter | `launch_commands.rs`, front-door launch dispatch | Same as rank 5; useful only if it shrinks the workspace extraction boundary | Keep `yazelix-0nvl.2` sequenced after `yazelix-0nvl.1` |
| 7 | Evaluate barless/native status component toggle | Defer | Zellij layout families, `zjstatus.wasm`, Home Manager component toggles | Real package/storage impact if accepted, but risky before layout ownership is stable | Keep `yazelix-jhu5` deferred |
| 8 | Public `yazelix_ratconfig` extraction | Defer | `config_ui.rs`, `yazelix_ratconfig/*` | Potentially large, but only after the private adapter survives real saves and Yazelix-specific apply/status logic stays local | Defer public repo; continue private boundary work only |
| 9 | Move maintainer tooling to a child repo | Defer / reject now | `yazelix_maintainer`, validators, release/update workflow | Would likely harm maintainer workflow and add cross-repo validator coupling before deleting code | Keep in repo; split validator domains and delete trivia first |
| 10 | Extract `yazelix_workspace` publicly | Defer | `zellij_commands.rs`, `launch_commands.rs`, workspace/session state, pane orchestrator client | Highest theoretical LOC impact, highest coupling | Defer until launch, layout ownership, and Zellij materialization shrinkage land |

## Top Follow-Ups

1. `yazelix-lzlg.1`: top delete-only candidate. Expected impact is runtime asset/storage reduction and less generated ownership. It must prove which Yazi assets are required before deleting anything.
2. `yazelix-g7bs.2`: top ownership-clarity cleanup. Resolved by renaming the remaining internal helper to `popup-session-facts.compute` and `popup_session_facts.rs`, so the main repo no longer implies that popup lifecycle belongs to the pane orchestrator after `yzpp`.
3. `yazelix-00nz`: top move-to-existing-child-repo candidate. Expected impact is `300-900` Rust lines if `yazelix_bar` absorbs only generic zjstatus rendering while Yazelix core keeps session/cache/helper ownership.

## Explicit Rejections

- Do not create a standalone Yazi repo while the main repo still ships the same `configs/yazi` asset pack and owns the same materializer paths
- Do not move `config_ui.rs` wholesale to `yazelix_ratconfig`; JSONC patching, Home Manager/native status, settings metadata, and runtime apply behavior are Yazelix-specific
- Do not move provider usage polling, cursor status facts, status-cache paths, or pane-orchestrator payloads to `yazelix_bar`
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
