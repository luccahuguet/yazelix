# Rust Code Inventory

This inventory is the extraction gate for reusable Yazelix components. It records the current Rust shape before moving code out of the main repository so extraction decisions start from concrete ownership rather than a raw line-count hunch.

Baseline measured on 2026-05-05 before extracting `yazelix-screen`:

- `tokei rust_core rust_plugins --exclude target` reports `67,462` Rust code LOC across `128` Rust files
- the same `tokei` run reports `74,009` Rust lines including blanks and comments
- `config_metadata/rust_ownership_budget.toml` tracks `72,385` raw Rust file lines across `123` Rust files after extracting `yazelix-screen`
- the remaining difference between `tokei` lines and the budget total is measurement-method noise from embedded blobs and parser classification, not a separate ownership surface
- `cargo check --workspace --all-targets` under `rust_core/` reports no warnings
- `cargo check --manifest-path rust_plugins/zellij_pane_orchestrator/Cargo.toml --all-targets` reports no warnings
- `cargo +nightly udeps --manifest-path rust_core/Cargo.toml --workspace --all-targets` reports all dependencies used
- `cargo +nightly udeps --manifest-path rust_plugins/zellij_pane_orchestrator/Cargo.toml --all-targets` reported direct dependency `shlex` unused; this pass removed it
- `cargo-udeps` requires nightly Rust because it passes unstable `-Z` compiler flags

The canonical family ownership, no-growth ceilings, and long-term warning target live in `config_metadata/rust_ownership_budget.toml`. The current budget excludes the extracted `yazelix-screen` crate.

## Ownership Split

| Family | Files | Raw lines | Status | Extraction pressure |
| --- | ---: | ---: | --- | --- |
| Product runtime source | 63 | 49,483 | canonical and extension surfaces | High: contains the largest user-facing seams |
| Product integration tests | 19 | 6,107 | canonical tests | Medium: split by behavior family, do not delete broadly |
| Maintainer tooling and tests | 16 | 11,583 | canonical maintainer | Medium: keep in repo, but split large validator files |
| Pane orchestrator plugin | 25 | 5,212 | extension surface | High: already has a natural Zellij plugin boundary |
| Total | 123 | 72,385 | current budget ceiling | Reduce or extract before raising ceilings |

Detailed budget families:

| Family | Files | Raw lines | Budget target | Notes |
| --- | ---: | ---: | ---: | --- |
| `bar_runtime` | 1 | 321 | 300 | Small crate; real extraction value comes from status cache/widgets in `zellij_commands.rs` |
| `core_cli_and_public_surface` | 12 | 8,652 | 7,000 | Public command dispatch and front-door rendering |
| `core_config_ui_and_materialization` | 32 | 19,412 | 14,000 | Largest product family; config UI, materializers, cursors, settings surfaces |
| `core_diagnostics_and_recovery` | 8 | 5,350 | 4,500 | Doctor, install ownership, profile/status reporting |
| `core_workspace_and_pane_integration` | 10 | 15,748 | 11,000 | Zellij/session/workspace command surface; biggest pre-extraction cleanup target |
| `core_integration_tests` | 19 | 6,107 | 4,500 | High-value tests, but several files are broad family buckets |
| `maintainer_tooling_and_validators` | 15 | 11,339 | 9,000 | Keep in repo; split validators by domain before optimizing |
| `maintainer_tests` | 1 | 244 | 244 | Small release/upgrade contract test surface |
| `pane_orchestrator_plugin` | 25 | 5,212 | 4,500 | Extension surface; refactor timer/status/sidebar modules before public extraction |

## Largest Files

| File | Raw lines | Disposition |
| --- | ---: | --- |
| `rust_core/yazelix_core/src/zellij_commands.rs` | 6,468 | Split before extracting bar/workspace/status surfaces |
| `rust_core/yazelix_maintainer/src/repo_contract_validation.rs` | 3,707 | Split by validator domain, keep in maintainer crate |
| `rust_core/yazelix_core/src/launch_commands.rs` | 3,316 | Split terminal selection, config overrides, launch execution, desktop/macOS handling |
| `rust_core/yazelix_core/src/config_ui.rs` | 3,123 | Split model, rendering, editing, and schema metadata before `yazelix_ratconfig` extraction |
| `rust_core/yazelix_core/src/zellij_materialization.rs` | 2,202 | Keep until keybinding ownership and layout-generation contracts settle |
| `rust_core/yazelix_core/tests/yzx_core_config_normalize.rs` | 1,671 | Split by config/materialization behavior family; do not delete without replacement coverage |
| `rust_core/yazelix_maintainer/src/repo_validation.rs` | 1,496 | Split generic validation helpers by contract/test/package domain |
| `rust_core/yazelix_core/src/public_command_surface.rs` | 1,486 | Keep central registry; future action registry may absorb part of this |
| `rust_core/yazelix_core/src/workspace_commands.rs` | 1,457 | Split popup/session/workspace concerns before workspace extraction |
| `rust_core/yazelix_maintainer/src/repo_update_workflow.rs` | 1,421 | Process-heavy maintainer workflow; keep local but modularize |
| `rust_core/yazelix_core/src/bin/yzx_core.rs` | 1,410 | Temporary machine helper; collapse only after shell callers have a stable replacement |
| `rust_core/yazelix_core/src/profile_commands.rs` | 1,292 | Keep while startup profiling remains an active debugging surface |
| `rust_core/yazelix_core/src/bin/yzx_control.rs` | 1,290 | Public command implementation dispatcher; split only if routing remains obvious |
| `rust_core/yazelix_core/src/yazi_materialization.rs` | 1,276 | Keep until Yazi config ownership/import mode is settled |
| `rust_core/yazelix_core/src/doctor_commands.rs` | 1,270 | Split report rendering from fix orchestration only after doctor behavior stabilizes |
| `rust_core/yazelix_core/tests/yzx_control_workspace_surface.rs` | 1,215 | Broad but behavior-backed; split by workspace/popup/session behaviors |
| `rust_core/yazelix_core/src/ghostty_cursor_registry.rs` | 1,174 | Strong `yazelix_cursors` extraction candidate |
| `rust_core/yazelix_core/src/runtime_materialization.rs` | 1,142 | Keep as runtime generated-state lifecycle owner |
| `rust_core/yazelix_core/src/install_ownership_report.rs` | 1,131 | Contains live recovery and legacy install diagnostics; prune only after transition windows |
| `rust_core/yazelix_maintainer/src/repo_sweep_runner.rs` | 1,068 | Live maintainer sweep surface, not demo code |

## Dead-Code And Dependency Evidence

No broad Rust source deletion is justified from compiler evidence in this pass.

Observed evidence:

- Core workspace `cargo check --workspace --all-targets` is warning-free
- Pane orchestrator `cargo check --all-targets` is warning-free
- `cargo-udeps` found no unused dependency in the core workspace
- `cargo-udeps` found `shlex` unused as a direct pane-orchestrator dependency; it was removed
- no tracked Rust symbol named `record_demo` exists
- `run_visual_verification` remains live maintainer sweep code

Deletion candidates need transition evidence, not only age:

| Surface | Evidence needed before deletion |
| --- | --- |
| `yzx_core` machine helper | Shell/bootstrap/Home Manager/Helix/Yazi callers need another stable machine protocol |
| `internal_nu_runner.rs` | Remaining `yzx dev`, popup/menu, and process-heavy Nu leaves need Rust replacements or explicit ownership |
| old flat config migration helpers | Delete only after the migration-retirement heuristic bead closes |
| legacy wrapper/install diagnostics | Delete only after supported upgrade windows no longer need doctor recovery |
| legacy popup-runner cleanup in Zellij materialization | Delete only after old runtime artifacts are outside the support boundary |
| `migration_available` upgrade-note rendering | Keep for historical upgrade-note display unless old note rendering is removed |

## Overengineering Hotspots

The main overengineering risk is not one bad abstraction; it is several broad modules owning too many unrelated contracts.

- `zellij_commands.rs` mixes Zellij pipe commands, status cache IO, AI usage widgets, cursor widgets, session inspection, and pane actions. Split this before extracting `yazelix_bar`, `yazelix_workspace`, or public status widgets.
- `launch_commands.rs` mixes terminal discovery, temporary config overrides, desktop/macOS launchers, process spawning, and restart/enter behavior. Split config override parsing and terminal selection before workspace/session extraction.
- `config_ui.rs` is already product-useful, but it should be split into schema model, list/editor state, rendering, and write-back before `yazelix_ratconfig`.
- `zellij_materialization.rs` contains real generated-config ownership, but it should wait for keybinding ownership and layout-profile decisions before major extraction.
- `repo_contract_validation.rs` and `repo_validation.rs` are maintainer-only, but their domain split would make future cleanup safer.

## Extraction Sequence

1. Finish this inventory and keep the no-growth budget current
2. Keep `yazelix-screen` external and avoid reintroducing duplicated screen source into the main repo
3. Split `zellij_commands.rs` before attempting `yazelix_bar` or workspace extraction
4. Extract `yazelix_cursors` after the Ghostty cursor registry is separated from terminal materialization and status-widget rendering
5. Split `config_ui.rs` before extracting `yazelix_ratconfig`; keep JSONC patching and schema metadata contracts stable first
6. Evaluate `yazelix_zellij_popup` after transient-pane commands and plugin transient policy have a clean boundary
7. Evaluate `yazelix_workspace` last; it touches launch, restart, session facts, workspace roots, Zellij layout state, and the pane orchestrator

Do before extraction:

- split broad modules by behavior boundary
- remove unused dependencies and stale transition helpers with evidence
- keep package/runtime surfaces distinct from maintainer-only tools
- update docs and validators while the source still lives in one repo

Wait until after extraction:

- aggressive test deletion
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

## Maintainer Tooling Split

Personal Home Manager should own day-to-day maintainer binaries that speed local work: `cargo-nextest`, `cargo-udeps`, `tokei`, `gh`, `jq`, `nu-lint`, Beads, and similar tools. This avoids repeated `nix shell` startup for common audits.

The repo should still own reproducible gates and package/runtime contracts. Do not move Cargo `target/` directories, incremental build state, or project-specific generated artifacts into Home Manager; those caches are project-local by design.

`cargo-udeps` is useful but manual. It belongs in the maintainer toolbox, not in the user runtime package.
