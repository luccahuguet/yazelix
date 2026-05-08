# Yazi Integration Boundary

This contract defines the current ownership boundary for Yazelix-managed Yazi
configuration, bundled Yazi plugins, and future Yazi extraction work.

## Current Rule

Do not extract full Yazi integration as a public child repository yet.

The reusable Yazi flavor and plugin asset pack lives in
`yazelix-yazi-assets`. Regular Yazelix users still get the same integrated
flavors and plugins through the packaged runtime; the child repo exists so
non-Yazelix users can consume those assets without adopting the full Yazelix
workspace.

The remaining materializer still owns too much Yazelix-specific behavior for a
public Yazi integration extraction: managed config roots, state-dir generation,
semantic Yazelix keybindings, editor opener preservation, pane-orchestrator
sidebar registration, and explicit rejection of legacy override paths.

## Boundary Matrix

| Surface | Current owner | Future movement |
| --- | --- | --- |
| `yazi_render_plan.rs` | Generic Yazi render-plan semantics with Yazelix metadata inputs | Candidate for a private generic Yazi config-pack boundary after asset ownership is slimmed |
| `config_metadata/yazi_render_plan.toml` | Shared machine metadata for sort, default plugins, core plugins, and random theme palettes | Candidate for the same config-pack boundary |
| `yazelix-yazi-assets/flavors/*` | Bundled Yazi flavor catalog | Child asset package; optional Home Manager/runtime component toggles can decide whether a full Yazelix install links it |
| `yazelix-yazi-assets/plugins/git.yazi`, `lazygit.yazi`, `starship.yazi` | Bundled reusable Yazi plugin pack | Child asset package; vendored update workflow belongs with the child repo, not Yazelix core |
| `yazelix-yazi-assets/plugins/auto-layout.yazi` | Yazelix-maintained Yazi sidebar fit behavior | Child asset package, still part of the default Yazelix runtime because the managed sidebar expects it |
| `sidebar-status.yazi`, `sidebar-state.yazi`, `zoxide-editor.yazi` | Yazelix editor/sidebar integration | Keep in Yazelix until pane-orchestrator protocol is separately extracted |
| `yazi_materialization.rs` generated file writes | Yazelix runtime materializer | Keep in Yazelix; it writes managed state paths and enforces Yazelix ownership errors |
| `yazi_materialization.rs` semantic keymap expansion | Yazelix action registry adapter | Keep in Yazelix; it depends on Yazelix-owned action ids and generated integration commands |
| `[opener].edit` preservation | Yazelix managed editor contract | Keep in Yazelix; native Yazi config must not replace the managed editor open path |
| Flat sidecars under `~/.config/yazelix/` | Yazelix user config ownership | Keep in Yazelix; Home Manager, import, config UI, and JSONC patching use this vocabulary |
| Generated output under `~/.local/share/yazelix/configs/yazi/` | Yazelix runtime state | Keep in Yazelix; it is not a user-editable source tree |
| `repo_update_workflow.rs` vendored plugin refresh | Removed from Yazelix main repo | Recreate only inside `yazelix-yazi-assets` if the child repo needs automated upstream refresh |

## Delete-First Decisions

Legacy `configs/yazi/user/*` override docs are obsolete. Runtime generation now
rejects that directory and tells users to import or move overrides into the flat
`~/.config/yazelix/` sidecars. The stale guide should not ship because it
advertises a path that no longer works.

Generated runtime files are not source templates. Documentation should not tell
users to edit `~/.config/yazelix/configs/yazi/yazelix_*.toml`; supported
customization lives in `settings.jsonc` plus flat managed Yazi sidecars.

Do not delete the legacy error path in `yazi_materialization.rs` yet. It is the
guard that prevents silent adoption of old mutable config locations.

## Extraction Readiness

Full Yazi integration extraction is deferred.

The asset pack movement is complete: `configs/yazi/` in this repository keeps
only Yazelix-owned base templates and sidebar/editor plugins, while the packaged
runtime links reusable flavors, Starship config, `auto-layout.yazi`, `git.yazi`,
`lazygit.yazi`, and `starship.yazi` from `yazelix-yazi-assets`.

The next useful movement is to split the Rust materializer into:

1. a generic render-plan/config-pack writer with no Yazelix paths
2. a Yazelix adapter that owns settings normalization, flat override paths,
   Home Manager/import vocabulary, semantic action ids, and runtime apply
   reporting

Only after that adapter is thin should a public Yazi integration repository be
considered. The asset child repo should not grow a generated mirror in this
repository.

## LOC Scorecard

Current Yazi surface measured on 2026-05-07:

| Surface | Lines | Notes |
| --- | ---: | --- |
| `rust_core/yazelix_core/src/yazi_materialization.rs` | 1,464 | Mixed materializer, runtime-state writer, keymap adapter, and legacy guard |
| `rust_core/yazelix_core/src/yazi_render_plan.rs` | 276 | Small enough to keep until a config-pack writer exists |
| `rust_core/yazelix_core/tests/yzx_core_yazi_materialization.rs` | 459 | Behavior coverage for generated files, assets, keybindings, and legacy rejection |
| `rust_core/yazelix_core/tests/yzx_core_yazi_render_plan.rs` | 58 | Machine CLI envelope coverage |
| `configs/yazi/` | 356 | Main repo keeps only Yazelix base templates and sidebar/editor integration plugins |
| `yazelix-yazi-assets/` | 6,822 | Child repo package containing 24 flavors, reusable Yazi plugins, Starship config, package metadata, lockfile, and pinned upstream refresh metadata |

Main-repo `configs/yazi/` shrank by `6,157` raw lines. Future reduction comes
from optionalizing the asset package and splitting the current Rust materializer,
not from re-creating asset mirrors in this repository.

## Verification

- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core yazi_materialization`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core yazi_render_plan`
- `yzx_repo_validator validate-contracts`
- `yzx_repo_validator validate-docs-experience`
