# Rust Code Inventory

This inventory is the extraction gate for reusable Yazelix components. It records the current Rust shape before moving code out of the main repository so extraction decisions start from concrete ownership rather than a raw line-count hunch.

Current rebaseline measured on 2026-05-07 after extracting `yazelix-screen`, `yazelix-cursors`, and `yazelix-bar`, accepting the optional runtime component toggles, paying down the first post-v16.3 Rust budget debt, deleting stale strength-score and migration metadata, and dropping the visual sweep layout lane:

- `tokei rust_core rust_plugins --exclude target` reports `67,711` Rust code LOC across `142` Rust files
- the same `tokei` run reports `73,764` Rust lines including blanks and comments
- `config_metadata/rust_ownership_budget.toml` tracks `73,882` raw Rust file lines across `142` Rust files
- the remaining difference between `tokei` lines and the budget total is measurement-method noise from embedded markdown/parser classification and line-count method differences, not a separate ownership surface
- `yzx_repo_validator validate-rust-ownership-budget` passes the no-growth budget and still warns that the tracked Rust surface is above the long-term `60,000` LOC hard target
- `cargo-udeps` requires nightly Rust because it passes unstable `-Z` compiler flags; rerun it during explicit dependency-audit beads rather than treating this inventory as fresh unused-dependency evidence

The canonical family ownership, no-growth ceilings, and long-term warning target live in `config_metadata/rust_ownership_budget.toml`. The current budget excludes extracted child crates.

The latest budget-debt paydown deleted the hidden moved-Ghostty cursor-field runtime repair migration, the structured strength-score validator machinery, stale per-test strength-score comments, the separate default-test traceability command, automatic legacy config rewrite paths, the live legacy popup-runner cleanup, and the visual terminal sweep lane. Those cuts removed `1,899` Rust code LOC by `tokei` and `2,418` raw budget lines from the main repo, paying back the `650` code-LOC debt created by the optional runtime-component toggle slice.

## Ownership Split

| Family | Files | Raw lines | Status | Extraction pressure |
| --- | ---: | ---: | --- | --- |
| Product runtime source | 78 | 51,090 | canonical and extension surfaces | High: contains the largest user-facing seams |
| Product integration tests | 19 | 6,093 | canonical tests | Medium: split by behavior family, do not delete broadly |
| Maintainer tooling and tests | 18 | 11,314 | canonical maintainer | Medium: keep in repo, but split large validator files |
| Pane orchestrator plugin | 27 | 5,385 | extension surface | High: already has a natural Zellij plugin boundary |
| Total | 142 | 73,882 | current budget ceiling | Reduce or extract before raising ceilings |

Detailed budget families:

| Family | Files | Raw lines | Budget target | Notes |
| --- | ---: | ---: | ---: | --- |
| `core_cli_and_public_surface` | 12 | 8,159 | 7,000 | Public command dispatch and front-door rendering after child CLI extractions |
| `core_config_ui_and_materialization` | 41 | 20,653 | 14,000 | Largest product family; config UI, apply modes, runtime component manifest, ratconfig boundary, materializers, settings surfaces |
| `core_diagnostics_and_recovery` | 8 | 5,866 | 4,500 | Doctor, install ownership, profile/status reporting |
| `core_workspace_and_pane_integration` | 17 | 16,412 | 11,000 | Action registry, Zellij/session/workspace command surface, pane-orchestrator client, status/cache/widgets |
| `core_integration_tests` | 19 | 6,093 | 4,500 | High-value tests, but several files are broad family buckets |
| `maintainer_tooling_and_validators` | 17 | 11,075 | 9,000 | Keep in repo; split validators by domain before optimizing |
| `maintainer_tests` | 1 | 239 | 239 | Small release/upgrade contract test surface |
| `pane_orchestrator_plugin` | 27 | 5,385 | 4,500 | Extension surface; refactor runtime config, timer/status/sidebar modules before public extraction |

## Largest Files

| File | Raw lines | Disposition |
| --- | ---: | --- |
| `rust_core/yazelix_maintainer/src/repo_contract_validation.rs` | 3,856 | Split by validator domain, keep in maintainer crate |
| `rust_core/yazelix_core/src/config_ui.rs` | 3,240 | Continue the `yazelix_ratconfig` split and keep Yazelix adapters local |
| `rust_core/yazelix_core/src/zellij_commands.rs` | 2,859 | Split remaining pipe primitive, workspace/editor flow, and terminal pane actions before workspace extraction |
| `rust_core/yazelix_core/src/zellij_materialization.rs` | 2,809 | Keep until keybinding ownership and layout-generation contracts settle; integrated zjstatus command definitions live behind a typed adapter |
| `rust_core/yazelix_core/src/launch_commands.rs` | 2,626 | Top-level enter/launch/desktop/restart flow after terminal and config override split |
| `rust_core/yazelix_core/src/zellij_commands/status/agent_usage.rs` | 1,933 | Provider usage cache refreshes, shared-cache locking, and agent usage widget rendering; keep Yazelix-owned unless a standalone provider usage contract appears |
| `rust_core/yazelix_core/src/bin/yzx_control.rs` | 1,739 | Public command implementation dispatcher; split only if routing remains obvious |
| `rust_core/yazelix_core/src/zellij_commands/status.rs` | 1,548 | Status bus/cache commands plus cursor/workspace widget rendering after agent usage split; keep cache-path and session-state ownership local |
| `rust_core/yazelix_core/tests/yzx_core_config_normalize.rs` | 1,516 | Split by config/materialization behavior family; do not delete without replacement coverage |
| `rust_core/yazelix_core/src/yazi_materialization.rs` | 1,464 | Keep in core; current owner map is in `docs/contracts/yazi_integration_boundary.md` |
| `rust_core/yazelix_maintainer/src/repo_update_workflow.rs` | 1,417 | Process-heavy maintainer workflow; keep local but modularize |
| `rust_core/yazelix_core/src/bin/yzx_core.rs` | 1,412 | Temporary machine helper; collapse only after shell callers have a stable replacement |
| `rust_core/yazelix_core/src/doctor_commands.rs` | 1,403 | Split report rendering from fix orchestration only after doctor behavior stabilizes |
| `rust_core/yazelix_core/tests/yzx_control_workspace_surface.rs` | 1,257 | Broad but behavior-backed; split by workspace/popup/session behaviors |
| `rust_core/yazelix_core/src/profile_commands.rs` | 1,256 | Keep while startup profiling remains an active debugging surface |
| `rust_core/yazelix_core/src/public_command_surface.rs` | 1,223 | Keep central registry; future action registry may absorb part of this |
| `rust_core/yazelix_core/src/install_ownership_report.rs` | 1,124 | Contains live recovery and legacy install diagnostics; prune only after transition windows |
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
| `repo_update_workflow.rs` | keep in repo as mutating release/update workflow | Split runtime-pin sync, vendored plugin refresh, activation, and canary materialization before considering a child repo |
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
- `launch_commands.rs` still mixes desktop/macOS launchers, process spawning, and restart/enter behavior. Terminal selection and temporary config overrides live in private submodules; split process launch and desktop/macOS handling before workspace/session extraction.
- `config_ui.rs` is already product-useful, but it should be split into schema model, list/editor state, rendering, write-back, and Yazelix adapter policy before `yazelix_ratconfig`. The extraction readiness state is `internal_split_ready`, not standalone-public-ready.
- `zellij_materialization.rs` contains real generated-config ownership, but it should wait for keybinding ownership and layout-profile decisions before major extraction.
- `yazi_materialization.rs` is smaller than the Zellij/config UI surfaces, but it mixes generic Yazi render planning with Yazelix-owned state paths, semantic keybindings, opener preservation, and legacy override rejection. The current extraction readiness state is `audit_deferred`; slim the bundled asset pack and split a private writer/adapter boundary before a public child repo.
- `repo_contract_validation.rs` and `repo_validation.rs` are maintainer-only, but their domain split would make future cleanup safer.

## Extraction Sequence

1. Keep this inventory and the no-growth budget current; every accepted Rust growth slice should record whether it is deletion debt or a justified new owner
2. Keep `yazelix-screen` external and avoid reintroducing duplicated screen source into the main repo
3. Continue the `zellij_commands.rs` split: status/cache/widget code now lives under `zellij_commands/status.rs`, agent usage refreshers live under `zellij_commands/status/agent_usage.rs`, and integrated bar command definitions render through a typed adapter. Next isolate workspace/editor pane flow before `yazelix_workspace`
4. Keep `#yazelix_cursors` as the standalone cursor package; reusable registry, `yzc`, Ghostty shader generation, and packaged shader assets live in `github:luccahuguet/yazelix-cursors`, while `ghostty_cursor_registry.rs` remains the Yazelix settings adapter
5. Split `config_ui.rs` before extracting `yazelix_ratconfig`; keep JSONC patching, schema metadata, read-only ownership, and apply-status contracts stable first
6. Keep the external `yazelix-zellij-popup` source project separate while packaging `yzpp.wasm` in Yazelix for the integrated popup/menu/config UI path; `yzpp` remains its short Zellij plugin alias and artifact name
7. Defer Yazi public extraction until the Yazi config/plugin asset pack is slimmed and the materializer has a private generic writer plus Yazelix adapter boundary
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
| `v16.3..a001fab0` | `3,550` insertions, `4,093` deletions, net `-543` | `-2` | Main repo is roughly flat after generated clutter deletion and optional component toggles |
| `a001fab0^..a001fab0` | `804` insertions, `78` deletions, net `+726` | `+650` | Optional runtime component toggles are accepted product behavior, but they increased the cleanup debt |

## Maintainer Tooling Split

Personal Home Manager should own day-to-day maintainer binaries that speed local work: `cargo-nextest`, `cargo-udeps`, `tokei`, `gh`, `jq`, `nu-lint`, Beads, and similar tools. This avoids repeated `nix shell` startup for common audits.

The repo should still own reproducible gates and package/runtime contracts. Do not move Cargo `target/` directories, incremental build state, or project-specific generated artifacts into Home Manager; those caches are project-local by design.

`cargo-udeps` is useful but manual. It belongs in the maintainer toolbox, not in the user runtime package.
