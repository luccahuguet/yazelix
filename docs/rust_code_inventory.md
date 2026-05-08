# Rust Code Inventory

This inventory is the extraction gate for reusable Yazelix components. It records the current Rust shape before moving code out of the main repository so extraction decisions start from concrete ownership rather than a raw line-count hunch.

Current rebaseline measured on 2026-05-08 after extracting `yazelix-screen`, `yazelix-cursors`, `yazelix-bar`, `yazelix-zellij-popup`, and `yazelix-yazi-assets`, accepting the optional runtime component toggles, paying down the first post-v16.3 Rust budget debt, deleting stale strength-score and migration metadata, dropping the visual sweep layout lane, moving popup lifecycle ownership to `yzpp`, closing the first yzpp cleanup tail, moving reusable Yazi plugin refresh ownership out of the main repo, moving integrated zjstatus command-definition rendering into `yazelix-bar`, splitting Yazi materialization into a private Yazelix adapter plus writer boundary, and splitting launch process/desktop/enter/restart adapters out of the launch parent:

- `tokei rust_core rust_plugins --exclude target` reports `67,020` Rust code LOC across `145` Rust files
- the same `tokei` run reports `73,085` Rust lines including blanks and comments
- `config_metadata/rust_ownership_budget.toml` tracks `73,203` raw Rust file lines across `145` Rust files
- the remaining difference between `tokei` lines and the budget total is measurement-method noise from embedded markdown/parser classification and line-count method differences, not a separate ownership surface
- `yzx_repo_validator validate-rust-ownership-budget` passes the no-growth budget and still warns that the tracked Rust surface is above the long-term `60,000` LOC hard target
- `cargo-udeps` requires nightly Rust because it passes unstable `-Z` compiler flags; rerun it during explicit dependency-audit beads rather than treating this inventory as fresh unused-dependency evidence

The canonical family ownership, no-growth ceilings, and long-term warning target live in `config_metadata/rust_ownership_budget.toml`. The current budget excludes extracted child crates.

The latest budget-debt paydown deleted the hidden moved-Ghostty cursor-field runtime repair migration, the structured strength-score validator machinery, stale per-test strength-score comments, the separate default-test traceability command, automatic legacy config rewrite paths, the live legacy popup-runner cleanup, and the visual terminal sweep lane. Those cuts removed `1,899` Rust code LOC by `tokei` and `2,418` raw budget lines from the main repo, paying back the `650` code-LOC debt created by the optional runtime-component toggle slice.

## Ownership Split

| Family | Files | Raw lines | Status | Extraction pressure |
| --- | ---: | ---: | --- | --- |
| Product runtime source | 83 | 51,333 | canonical and extension surfaces | High: contains the largest user-facing seams |
| Product integration tests | 19 | 6,039 | canonical tests | Medium: split by behavior family, do not delete broadly |
| Maintainer tooling and tests | 18 | 10,990 | canonical maintainer | Medium: keep in repo, but split large validator files |
| Pane orchestrator plugin | 25 | 4,841 | extension surface | High: already has a natural Zellij plugin boundary |
| Total | 145 | 73,203 | current budget ceiling | Reduce or extract before raising ceilings |

Detailed budget families:

| Family | Files | Raw lines | Budget target | Notes |
| --- | ---: | ---: | ---: | --- |
| `core_cli_and_public_surface` | 12 | 8,171 | 7,000 | Public command dispatch and front-door rendering after child CLI extractions |
| `core_config_ui_and_materialization` | 42 | 20,684 | 14,000 | Largest product family; config UI, apply modes, runtime component manifest, ratconfig boundary, materializers, settings surfaces |
| `core_diagnostics_and_recovery` | 8 | 5,866 | 4,500 | Doctor, install ownership, profile/status reporting |
| `core_workspace_and_pane_integration` | 21 | 16,612 | 11,000 | Action registry, launch private adapters, Zellij/session/workspace command surface, pane-orchestrator client, status/cache/widgets |
| `core_integration_tests` | 19 | 6,039 | 4,500 | High-value tests, but several files are broad family buckets |
| `maintainer_tooling_and_validators` | 17 | 10,751 | 9,000 | Keep in repo; split validators by domain before optimizing |
| `maintainer_tests` | 1 | 239 | 239 | Small release/upgrade contract test surface |
| `pane_orchestrator_plugin` | 25 | 4,841 | 4,300 | Extension surface; refactor runtime config, timer/status/sidebar modules before public extraction |

## Largest Files

| File | Raw lines | Disposition |
| --- | ---: | --- |
| `rust_core/yazelix_maintainer/src/repo_contract_validation.rs` | 3,856 | Split by validator domain, keep in maintainer crate |
| `rust_core/yazelix_core/src/config_ui.rs` | 3,240 | Continue the `yazelix_ratconfig` split and keep Yazelix adapters local |
| `rust_core/yazelix_core/src/zellij_commands.rs` | 2,859 | Split remaining pipe primitive, workspace/editor flow, and terminal pane actions before workspace extraction |
| `rust_core/yazelix_core/src/zellij_materialization.rs` | 2,857 | Keep until keybinding ownership and layout-generation contracts settle; generic integrated zjstatus command-definition rendering lives in `yazelix-bar` |
| `rust_core/yazelix_core/src/zellij_commands/status/agent_usage.rs` | 1,933 | Provider usage cache refreshes, shared-cache locking, and agent usage widget rendering; keep Yazelix-owned unless a standalone provider usage contract appears |
| `rust_core/yazelix_core/src/bin/yzx_control.rs` | 1,741 | Public command implementation dispatcher; split only if routing remains obvious |
| `rust_core/yazelix_core/src/zellij_commands/status.rs` | 1,548 | Status bus/cache commands plus cursor/workspace widget rendering after agent usage split; keep cache-path and session-state ownership local |
| `rust_core/yazelix_core/tests/yzx_core_config_normalize.rs` | 1,533 | Split by config/materialization behavior family; do not delete without replacement coverage |
| `rust_core/yazelix_core/src/bin/yzx_core.rs` | 1,411 | Temporary machine helper; collapse only after shell callers have a stable replacement |
| `rust_core/yazelix_core/src/doctor_commands.rs` | 1,403 | Split report rendering from fix orchestration only after doctor behavior stabilizes |
| `rust_core/yazelix_core/src/profile_commands.rs` | 1,256 | Keep while startup profiling remains an active debugging surface |
| `rust_core/yazelix_core/src/public_command_surface.rs` | 1,234 | Keep central registry; future action registry may absorb part of this |
| `rust_core/yazelix_core/tests/yzx_control_workspace_surface.rs` | 1,203 | Broad but behavior-backed; split by workspace/popup/session behaviors |
| `rust_core/yazelix_maintainer/src/repo_update_workflow.rs` | 1,124 | Process-heavy maintainer workflow; Yazi plugin refresh moved to `yazelix-yazi-assets` |
| `rust_core/yazelix_core/src/install_ownership_report.rs` | 1,124 | Contains live recovery and legacy install diagnostics; prune only after transition windows |
| `rust_core/yazelix_core/src/launch_commands.rs` | 1,111 | Public launch command dispatch plus shared launch flow after terminal, config override, process, desktop, enter, and restart adapter split |
| `rust_core/yazelix_maintainer/src/repo_validation.rs` | 1,047 | Leaner validator shell; avoid rebuilding test-metadata parsers and cleanup-history heuristics |
| `rust_core/yazelix_core/src/runtime_contract.rs` | 1,030 | Runtime manifest and optional component ownership; keep until component opt-out behavior stabilizes |

## Maintainer Validator Classification

The maintainer crate should stay in this repo until reusable helpers can run
against arbitrary checkouts through a stable machine contract. Current large
owners classify as:

| Surface | Classification | Next deletion/split trigger |
| --- | --- | --- |
| `repo_contract_validation.rs` | keep in repo, split by validator domain | README/release surface moved to `repo_contract_validation/readme_surface.rs`; next split config-surface, flake/package, upgrade-note, installed-runtime, and helper IO groups before adding more validators |
| `repo_validation.rs` | keep lean validator shell | Stale score metadata and cleanup-history heuristics are deleted; do not rebuild score parsers |
| `repo_update_workflow.rs` | keep in repo as mutating release/update workflow | Yazi plugin refresh ownership moved to `yazelix-yazi-assets`; split runtime-pin sync, zjstatus refresh, activation, and canary materialization before considering a child repo |
| `repo_sweep_runner.rs` | keep as explicit sweep surface | Visual terminal-window lane is deleted; keep remaining config/runtime sweep focused |

## Dead-Code And Dependency Evidence

No broad Rust source deletion is justified from this rebaseline alone.

Previously recorded compiler/dependency evidence:

- Core workspace `cargo check --workspace --all-targets` is warning-free
- Pane orchestrator `cargo check --all-targets` is warning-free
- `cargo-udeps` found no unused dependency in the core workspace
- `cargo-udeps` found `shlex` unused as a direct pane-orchestrator dependency; it was removed
- no tracked Rust symbol named `record_demo` exists
- no tracked Rust symbol named `run_visual_verification` exists

Transition helpers need live-contract evidence to stay, not only age:

| Surface | Evidence needed before deletion |
| --- | --- |
| `yzx_core` machine helper | Shell/bootstrap/Home Manager/Helix/Yazi callers need another stable machine protocol |
| `internal_nu_runner.rs` | Remaining `yzx dev`, popup/menu, and process-heavy Nu leaves need Rust replacements or explicit ownership |
| legacy config diagnostics | Keep clear hard errors for old `yazelix.toml`, `cursors.toml`, and `user_configs/` paths; do not rebuild automatic rewrite helpers |
| legacy wrapper/install diagnostics | Delete only after supported upgrade windows no longer need doctor recovery |
| `migration_available` upgrade-note rendering | Keep for historical upgrade-note display unless old note rendering is removed |

## Overengineering Hotspots

The main overengineering risk is not one bad abstraction; it is several broad modules owning too many unrelated contracts.

- `zellij_commands.rs` has had status cache IO, AI usage widgets, cursor widgets, and session inspection moved into `zellij_commands/status.rs`. The remaining file still mixes Zellij pipe primitives, workspace/editor flow, terminal pane actions, and pane actions; split those before extracting `yazelix_workspace`. The extraction readiness state is `internal_boundary_only`, not standalone-public-ready.
- `launch_commands.rs` now keeps public command dispatch, launch fallback flow, argument parsing, and shared terminal cursor facts while terminal selection, temporary config overrides, process/probe execution, desktop/macOS, enter, and restart live in private modules. The next split should isolate launch fallback flow itself before workspace/session extraction.
- `config_ui.rs` is already product-useful, but it should be split into schema model, list/editor state, rendering, write-back, and Yazelix adapter policy before `yazelix_ratconfig`. The extraction readiness state is `internal_split_ready`, not standalone-public-ready.
- `zellij_materialization.rs` contains real generated-config ownership, but it should wait for keybinding ownership and layout-profile decisions before major extraction.
- `yazi_materialization.rs` is now a private Yazelix adapter over `yazi_materialization/writer.rs`. The split separates settings/path/keybinding policy from generated file and asset writes, but public extraction remains deferred until the adapter is thinner and the writer has a reusable config-pack contract.
- `repo_contract_validation.rs` and `repo_validation.rs` are maintainer-only, but their domain split would make future cleanup safer.

## Extraction Sequence

1. Keep this inventory and the no-growth budget current; every accepted Rust growth slice should record whether it is deletion debt or a justified new owner
2. Keep `yazelix-screen` external and avoid reintroducing duplicated screen source into the main repo
3. Continue the `zellij_commands.rs` split: status/cache/widget code now lives under `zellij_commands/status.rs`, agent usage refreshers live under `zellij_commands/status/agent_usage.rs`, and integrated bar command definitions render through a typed adapter. Next isolate workspace/editor pane flow before `yazelix_workspace`
4. Keep `#yazelix_cursors` as the standalone cursor package; reusable registry, `yzc`, Ghostty shader generation, and packaged shader assets live in `github:luccahuguet/yazelix-cursors`, while `ghostty_cursor_registry.rs` remains the Yazelix settings adapter
5. Split `config_ui.rs` before extracting `yazelix_ratconfig`; keep JSONC patching, schema metadata, read-only ownership, and apply-status contracts stable first
6. Keep the external `yazelix-zellij-popup` source project separate while packaging `yzpp.wasm` in Yazelix for the integrated popup/menu/config UI path; `yzpp` remains its short Zellij plugin alias and artifact name
7. Keep the Yazi writer boundary private until Yazelix paths, action ids, opener preservation, and legacy override errors are thin enough to leave behind cleanly
8. Evaluate `yazelix_workspace` last; it touches launch, restart, session facts, workspace roots, Zellij layout state, and the pane orchestrator. Session persistence/resurrection remains out of scope for the extraction gate.

Do before extraction:

- split broad modules by behavior boundary
- remove unused dependencies and stale transition or migration helpers unless a live contract requires them
- keep package/runtime surfaces distinct from maintainer-only tools
- update docs and validators while the source still lives in one repo

Wait until after extraction:

- aggressive unrelated test deletion; weak tests and fixtures attached to moved or deleted surfaces should be removed in the same bead
- public crate API polish
- package-size optimization for extracted crates
- cross-repo release automation
- lowering main-repo LOC ceilings to the post-extraction number

## LOC Targets

These targets separate deletion from extraction accounting. Moving code out of this repository reduces main-repo LOC, but it does not reduce total maintenance unless the extracted API is simpler than the old internal boundary.

| Target | Main-repo path | Total-maintenance interpretation |
| --- | --- | --- |
| `70k` | Achievable by keeping `yazelix-screen` external plus focused stale transition cleanup | Useful first target; should not require risky rewrites |
| `65k` | Requires another `5k` beyond the first cut, likely from cursors/config UI/bar boundary work | Good medium target if extracted packages have clean ownership |
| `60k` | Requires multiple successful extractions or real simplification of `zellij_commands.rs`, `launch_commands.rs`, and `config_ui.rs` | Realistic as a main-repo target, not as immediate deletion |
| `50k` | Requires moving maintainer tooling and several reusable components out of the main repo, or deleting large behavior surfaces | Not realistic as near-term true maintenance reduction; only valid if public extracted crates are independently useful and simpler |

The current budget sets `hard_target_loc = 60000`. The validator intentionally warns while the repo is above that target; the warning should push deletion, simplification, or extraction before anyone raises ceilings again.

Recent accepted full-repo scorecards:

| Range | Raw text diff excluding `.beads` | Tokei code LOC delta | Budget interpretation |
| --- | --- | ---: | --- |
| `yazelix-epiw` | split plus trivia deletion, net `-18` raw Rust lines | `-19` | Moved README/latest-release validation and sync into a private domain module, deleted duplicate generated-block heading checks, and ratcheted the maintainer budget while accepting one extra Rust file for clearer ownership |
| `yazelix-0nvl` | mechanical move, net `+33` raw Rust lines | `+30` | Split launch terminal selection and config override logic into private modules; budget remains below the pre-gr41 ceiling but this is organization debt, not deletion |
| `yazelix-gr41/yazelix-gr41.1` | `82` insertions, `786` deletions, net `-704` | `-502` | Deleted the visual sweep lane, its runtime-only KDL layout, and legacy popup-runner cleanup; ratcheted the Rust budget ceiling down to the measured surface |
| `yazelix-lzlg.1` | removed Yazi plugin-refresh workflow from main repo, net `-293` raw Rust lines | `-291` | Moved reusable Yazi asset ownership to `yazelix-yazi-assets`, deleted main-repo vendored plugin refresh code, and ratcheted the Rust budget ceiling down |
| `yazelix-00nz` | `11` insertions, `148` deletions, net `-137` raw Rust lines in main; `+169` Rust lines in `yazelix-bar` | `-131` | Moved generic integrated zjstatus command-definition KDL rendering into `yazelix-bar`; main keeps runtime helper path resolution, status-cache ownership, and session facts |
| `yazelix-lzlg.2` | private boundary split, net `+60` raw Rust lines | `+54` | Split `yazi_materialization.rs` into a 567-line Yazelix adapter and a 957-line private writer; this is accepted organization debt, not deletion or public extraction |
| `yazelix-0nvl.1` | private boundary split, net `+23` raw Rust lines | `-47` | Split launch process/probe execution and desktop/macOS adapters into private modules; `launch_commands.rs` dropped from 2,617 to 1,632 lines, but the file-count seam is accepted organization debt |
| `yazelix-0nvl.2` | private boundary split, net `+37` raw Rust lines | `+36` | Split enter and restart orchestration into private modules; `launch_commands.rs` dropped from 1,632 to 1,111 lines, but the new file seams are accepted organization debt |
| `v16.3..a001fab0` | `3,550` insertions, `4,093` deletions, net `-543` | `-2` | Main repo is roughly flat after generated clutter deletion and optional component toggles |
| `a001fab0^..a001fab0` | `804` insertions, `78` deletions, net `+726` | `+650` | Optional runtime component toggles are accepted product behavior, but they increased the cleanup debt |

## Maintainer Tooling Split

Personal Home Manager should own day-to-day maintainer binaries that speed local work: `cargo-nextest`, `cargo-udeps`, `tokei`, `gh`, `jq`, `nu-lint`, Beads, and similar tools. This avoids repeated `nix shell` startup for common audits.

The repo should still own reproducible gates and package/runtime contracts. Do not move Cargo `target/` directories, incremental build state, or project-specific generated artifacts into Home Manager; those caches are project-local by design.

`cargo-udeps` is useful but manual. It belongs in the maintainer toolbox, not in the user runtime package.
