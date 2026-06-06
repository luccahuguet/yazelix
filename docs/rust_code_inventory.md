# Rust Code Inventory

This inventory records the current Rust ownership shape in the main Yazelix repository. It is a current-state extraction gate.

## Current Baseline

Measured on 2026-06-06:

- `tokei rust_core rust_plugins --exclude target` reports `73,941` Rust code LOC across `157` Rust files
- `find rust_core rust_plugins -path '*/target' -prune -o -name '*.rs' -type f -print0 | xargs -0 wc -l` reports `80,894` raw Rust file lines
- `config_metadata/rust_ownership_budget.toml` tracks `80,894` raw Rust file lines across `157` Rust files
- the small difference between `tokei` language lines and the budget total is measurement-method noise from embedded markdown/parser classification and line-count method differences, not a separate ownership surface
- `yzx_repo_validator validate-rust-ownership-budget` passes the ownership budget and warns while the tracked raw Rust surface remains above the long-term `60,000` LOC hard target
- the current budget excludes extracted child crates and tracked build output
- this rebaseline records accepted debt from the Rust/Nu control-plane cuts; it is not permission to raise ceilings again without deleting or justifying a real owner

The canonical family ownership, no-growth ceilings, and long-term warning target live in `config_metadata/rust_ownership_budget.toml`.

## Ownership Split

| Family | Files | Raw lines | Status | Extraction pressure |
| --- | ---: | ---: | --- | --- |
| Product runtime source | 110 | 59,250 | canonical runtime and integration adapters | High: contains the largest user-facing seams |
| Product integration tests | 21 | 8,249 | canonical behavior tests | Medium: split by behavior family, do not delete broadly |
| Maintainer tooling and tests | 26 | 13,395 | canonical maintainer tooling | Medium: keep in repo, but keep splitting large validator files |
| Pane orchestrator plugin | 0 | 0 | extracted child source | Keep source in `yazelix-zellij-pane-orchestrator` |
| Total | 157 | 80,894 | current budget ceiling | Reduce or extract before raising ceilings |

Detailed budget families:

| Family | Files | Raw lines | Budget target | Notes |
| --- | ---: | ---: | ---: | --- |
| `core_cli_and_public_surface` | 13 | 8,713 | 7,000 | Public command dispatch, command metadata, front-door rendering, agent command surface, and support/onboarding commands |
| `core_config_ui_and_materialization` | 54 | 24,548 | 14,000 | Largest product family; Yazelix config UI adapter, apply modes, runtime component manifest, materializers, settings surfaces |
| `core_diagnostics_and_recovery` | 8 | 8,347 | 4,500 | Doctor, install ownership, profile/status reporting |
| `core_workspace_and_pane_integration` | 35 | 17,642 | 11,000 | Action registry, launch private adapters, Zellij/session/workspace command surface, pane-orchestrator client, status cache IO, and workspace widgets |
| `core_integration_tests` | 21 | 8,249 | 4,500 | High-value tests, but several files are broad family buckets |
| `maintainer_tooling_and_validators` | 25 | 13,156 | 9,000 | In-repo validators, Beads/GitHub sync, release/update workflow, sweep/test runners, and repo maintenance commands |
| `maintainer_tests` | 1 | 239 | 239 | Small release/upgrade contract test surface |

## Largest Files

| File | Raw lines | Current owner | Disposition |
| --- | ---: | --- | --- |
| `rust_core/yazelix_core/src/zellij_materialization.rs` | 3,688 | generated Zellij config materialization | Keep until keybinding ownership and layout-generation contracts settle |
| `rust_core/yazelix_core/tests/yzx_core_config_normalize.rs` | 1,893 | config/materialization integration tests | Split by behavior family; do not delete without replacement coverage |
| `rust_core/yazelix_core/src/doctor_commands.rs` | 1,779 | doctor orchestration | Split report rendering from fix orchestration only after doctor behavior stabilizes |
| `rust_core/yazelix_core/src/doctor_runtime_report.rs` | 1,758 | runtime doctor reporting | Collapse duplicate report/fix plumbing before adding more checks |
| `rust_core/yazelix_core/src/bin/yzx_control.rs` | 1,715 | public control-plane command implementation | Split only if routing remains obvious |
| `rust_core/yazelix_core/tests/yzx_control_workspace_surface.rs` | 1,628 | workspace/control-plane integration tests | Broad but behavior-backed; split by workspace/popup/session behaviors |
| `rust_core/yazelix_core/src/install_ownership_report.rs` | 1,520 | install ownership diagnostics | Prune only after supported recovery paths are narrower |
| `rust_core/yazelix_core/src/front_door_render.rs` | 1,497 | public help/menu rendering | Keep readable; avoid new command-surface trivia tests |
| `rust_core/yazelix_core/src/bin/yzx_core.rs` | 1,485 | machine helper used by shell/bootstrap surfaces | Collapse only after callers have a stable replacement |
| `rust_core/yazelix_core/src/config_normalize.rs` | 1,448 | config behavior normalizer | Split only around durable setting families |
| `rust_core/yazelix_core/src/doctor_helix_report.rs` | 1,354 | Helix doctor reporting | Collapse with doctor report plumbing where contracts overlap |
| `rust_core/yazelix_core/src/helix_bridge_client.rs` | 1,348 | Helix action bridge client | Keep local while editor/workspace bridge ownership is Yazelix-specific |
| `rust_core/yazelix_maintainer/src/repo_contract_validation/config_surface.rs` | 1,270 | config-surface contract validator | Split only by clear contract families |
| `rust_core/yazelix_core/src/profile_commands.rs` | 1,256 | startup profiling | Keep while startup profiling remains an active debugging surface |
| `rust_core/yazelix_core/tests/yzx_control_runtime_surface.rs` | 1,160 | runtime/control-plane integration tests | Split by runtime ownership only when coverage stays behavior-backed |
| `rust_core/yazelix_core/src/public_command_surface.rs` | 1,084 | public command registry | Keep central registry; future action registry may absorb part of this |
| `rust_core/yazelix_core/src/runtime_contract.rs` | 1,081 | runtime manifest and optional component ownership | Keep until component opt-out behavior stabilizes |
| `rust_core/yazelix_maintainer/src/repo_validation.rs` | 1,047 | validator shell and repo policy checks | Keep lean; avoid rebuilding deleted trivia parsers |
| `rust_core/yazelix_maintainer/src/repo_child_release.rs` | 1,045 | child release validation | Keep in repo until child-declared release contracts shrink it |
| `rust_core/yazelix_core/src/zellij_render_plan.rs` | 1,038 | typed Zellij render plan | Keep as source of truth for materialization fingerprints |
| `rust_core/yazelix_core/src/zellij_commands/workspace.rs` | 1,005 | workspace command adapter | Keep thinning config/runtime path policy before any workspace extraction |
| `rust_core/yazelix_core/src/runtime_materialization.rs` | 989 | generated runtime materialization | Keep while generated-state contract is local |
| `rust_core/yazelix_maintainer/src/repo_contract_validation/nix_package.rs` | 979 | Nix package validator | Split by package contract family when it grows again |
| `rust_core/yazelix_core/src/settings_surface.rs` | 966 | settings defaults/schema rendering | Keep close to config metadata contract |
| `rust_core/yazelix_core/src/launch_commands.rs` | 953 | launch command dispatch shell | Keep public routing thin while private launch modules own details |
| `rust_core/yazelix_core/src/yazi_materialization/writer.rs` | 917 | private Yazi writer | Keep generic writer logic thin before any public extraction |
| `rust_core/yazelix_core/src/config_ui/model_builder.rs` | 912 | Yazelix config UI model adapter | Keep Yazelix schema, ownership, validation, and persistence local while generic UI mechanics stay in `yazelix-ratconfig` |

## Current Boundaries

`zellij_commands.rs` is a small export shell. Pipe/get-root commands live under `zellij_commands/pipe.rs`, workspace/editor/terminal flows live under `zellij_commands/workspace.rs`, status command adapters live under `zellij_commands/status.rs`, cache IO lives under `zellij_commands/status/cache.rs`, cursor/workspace widget rendering lives under `zellij_commands/status/widgets.rs`, and status regressions live under `zellij_commands/status/tests/{widgets,cache}.rs`. Provider usage widget implementation lives in `yazelix-zellij-bar`.

`workspace_commands.rs` keeps `yzx cwd`, session config loading, managed-editor kind detection, zoxide/path resolution, and current-tab retargeting. `workspace_commands/popup.rs` owns the `yzpp`-backed popup adapter, and `workspace_commands/yazi_sidebar.rs` owns reveal/sidebar refresh, sidebar focus, `ya emit-to`, and command availability. Shared workspace target-directory, tab-name, and retarget-payload shaping live in `workspace_session.rs`.

`launch_commands.rs` keeps public command dispatch, desktop dispatch parsing, and the shared cwd resolver. Terminal selection, temporary config overrides, process/probe execution, desktop/macOS, launch fallback, enter, and restart live in private modules.

`config_ui.rs` is the small Yazelix config UI adapter shell. `config_ui/*` keeps Yazelix settings schema, Home Manager/read-only ownership, native config status, cursor config composition, action-registry detail text, validation, file writes, and runtime apply policy. Reusable model/editor/render helpers, row filtering/search, generic row/detail rendering, JSONC patch primitives, and migration primitives live in `yazelix-ratconfig`.

`zellij_materialization.rs` owns generated Zellij config policy, layout fragments, keybinding rendering, plugin block wiring, and runtime helper path resolution. It should remain local until layout and keybinding ownership are thinner.

`yazi_materialization.rs` is a private Yazelix adapter over `yazi_materialization/writer.rs`. Managed override discovery and parsing live in the adapter, while the writer receives optional overlays and writes the config pack.

`repo_contract_validation.rs` and `repo_validation.rs` are maintainer-only. Config-surface, upgrade-contract, installed-runtime, Nix interface, package/profile, helper IO, docs, and README validators live in private modules.

## Extracted Child Ownership

The main repo intentionally consumes these child projects instead of owning their reusable source here:

| Child project | Main repo relationship |
| --- | --- |
| `yazelix-screen` | terminal animation package consumed by welcome/screen surfaces |
| `yazelix-cursors` | reusable cursor registry, `yzc`, Ghostty-compatible shader generation, and packaged shader assets |
| `yazelix-ratconfig` | reusable Ratatui config editor model/editor/render, JSONC patching, and migration primitives consumed by the Yazelix config UI adapter |
| `yazelix-zellij-bar` | reusable Zellij bar renderers, non-workspace widget commands, provider probing/cache behavior, CPU/RAM, and packaged standalone bar |
| `yazelix-zellij-popup` | `yzpp` popup plugin wasm and plain-Zellij popup behavior |
| `yazelix-zellij-pane-orchestrator` | pane orchestration plugin source; main repo consumes the synced runtime artifact and integration contracts |
| `yazelix-yazi-assets` | reusable Yazi flavor/plugin assets and refresh ownership |

Main keeps Yazelix-specific adapters: runtime path selection, generated config materialization, Home Manager behavior, session facts, workspace facts, status cache integration, and user-facing `yzx` command policy.

## Dead-Code And Dependency Evidence

No broad Rust source deletion is justified from this inventory alone.

Current evidence to keep fresh during focused audits:

- core workspace checks should remain warning-free
- pane-orchestrator source checks belong in the child repo
- `cargo-udeps` requires nightly Rust because it passes unstable `-Z` compiler flags; run it during explicit dependency-audit work rather than treating this inventory as unused-dependency evidence
- compatibility and recovery diagnostics need live-contract evidence to stay, not age alone

## Extraction Sequence

1. Keep this inventory and the no-growth budget current; every accepted Rust growth slice should record whether it is deletion debt or a justified new owner
2. Continue thinning Zellij command/status adapters before extracting workspace ownership
3. Continue thinning workspace adapters while keeping zoxide/path resolution, config facts, runtime wrapper paths, and current-tab retargeting local
4. Keep the config UI adapter thin against `yazelix-ratconfig`; do not reintroduce generic UI, JSONC patching, or migration behavior into the main repo
5. Keep `zellij_materialization.rs` local until keybinding ownership, layout-profile decisions, and plugin path policy are stable
6. Keep the Yazi writer boundary private until Yazelix paths, action ids, opener preservation, and override errors are thin enough to leave behind cleanly
7. Keep maintainer validators in repo, but keep splitting large validator files by current contract domain
8. Evaluate public `yazelix_workspace` last; it touches launch, restart, session facts, workspace roots, Zellij layout state, and the pane orchestrator

Do before extraction:

- split broad modules by behavior boundary
- remove unused dependencies and stale transition helpers unless a live contract requires them
- keep package/runtime surfaces distinct from maintainer-only tools
- update docs and validators while the source still lives in one repo

Wait until after extraction:

- public crate API polish
- package-size optimization for extracted crates
- cross-repo release automation
- lowering main-repo LOC ceilings to the post-extraction number

## LOC Targets

These targets separate deletion from extraction accounting. Moving code out of this repository reduces main-repo LOC, but it does not reduce total maintenance unless the extracted API is simpler than the old internal boundary.

| Target | Main-repo path | Total-maintenance interpretation |
| --- | --- | --- |
| `80k` | Requires roughly `894` raw budget-line reduction from doctor reporting, workspace bridge code, tests, or maintainer validators | Immediate debt paydown gate after the Rust/Nu control-plane rebaseline |
| `70k` | Requires roughly `10.9k` raw budget-line reduction from materialization, workspace, config UI adapter, tests, or maintainer validator surfaces | Restores the previous near-term ceiling discipline |
| `65k` | Requires roughly `15.9k` raw budget-line reduction from the same large runtime/test/validator families | Treat future growth above this as budget debt unless product behavior requires it |
| `60k` | Requires roughly `20.9k` raw budget-line reduction from `zellij_materialization.rs`, doctor reports, workspace bridge code, config UI adapters, maintainer validators, or more public extraction | Next main-repo hard-target gate |
| `50k` | Requires roughly `30.9k` raw budget-line reduction by moving maintainer tooling and several reusable components out of the main repo, or deleting large behavior surfaces | Not realistic as near-term true maintenance reduction; only valid if public extracted crates are independently useful and simpler |

The current budget sets `hard_target_loc = 60000`. The validator intentionally warns while the raw tracked Rust surface is above that target.

## Validation Commands

```bash
tokei rust_core rust_plugins --exclude target
cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_maintainer --bin yzx_repo_validator -- validate-rust-ownership-budget
```
