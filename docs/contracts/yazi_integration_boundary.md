# Yazi Integration Boundary

This contract defines the current ownership boundary for Yazelix-managed Yazi
configuration, bundled Yazi plugins, and future Yazi extraction work.

## Current Rule

Do not extract full Yazi integration as a public child repository yet.

The reusable Yazi flavor, plugin, and config-pack renderer lives in
`yazelix-yazi-assets`. Regular Yazelix users still get the same integrated
generated config, flavors, and plugins through the packaged runtime; the child
repo exists so non-Yazelix users can consume those assets and the pure renderer
without adopting the full Yazelix workspace.

The remaining materializer is split into a private Yazelix adapter and a
private writer, but it still owns too much Yazelix-specific behavior for a
public Yazi integration extraction: managed config roots, state-dir generation,
semantic Yazelix keybindings, pane-orchestrator sidebar registration, and
explicit rejection of legacy override paths.

## Boundary Matrix

| Surface | Current owner | Future movement |
| --- | --- | --- |
| `yazelix-yazi-assets/src/lib.rs` render-plan API | Generic Yazi render-plan semantics, validation, template loading, and pure config-pack rendering | Child crate consumed by main through Cargo git dependency |
| `yazelix-yazi-assets/config_metadata/yazi_render_plan.toml` | Shared machine metadata for sort, default plugins, core plugins, and random theme palettes | Child-owned metadata embedded in the child crate |
| `yazelix-yazi-assets/config_templates/*` | Generated Yazi base config, theme, and keymap templates | Child-owned templates embedded in the child crate |
| `yazelix-yazi-assets/flavors/*` | Bundled Yazi flavor catalog | Child asset package; optional Home Manager/runtime component toggles can decide whether a full Yazelix install links it |
| `yazelix-yazi-assets/plugins/git.yazi`, `lazygit.yazi`, `starship.yazi` | Bundled reusable Yazi plugin pack | Child asset package; vendored update workflow belongs with the child repo, not Yazelix core |
| `yazelix-yazi-assets/plugins/auto-layout.yazi` | Yazelix-maintained Yazi sidebar fit behavior | Child asset package, still part of the default Yazelix runtime because the managed sidebar expects it |
| `yazelix-yazi-assets/config_metadata/yazi_assets_manifest.toml` | Child-declared reusable asset manifest | Child package shape contract for runtime asset linking |
| `sidebar-status.yazi`, `sidebar-state.yazi`, `zoxide-editor.yazi` | Yazelix editor/sidebar integration | Keep in Yazelix until pane-orchestrator protocol is separately extracted |
| `yazi_materialization.rs` adapter | Yazelix runtime materializer | Keep in Yazelix; it resolves settings, the managed Yazi home, semantic action ids, managed output paths, and legacy ownership errors |
| `yazi_materialization/writer.rs` generated file writes | Private Yazelix writer boundary | Keep private; it calls the child renderer, writes generated outputs, and syncs packaged child assets from the runtime tree |
| `yazi_materialization.rs` semantic keymap expansion | Yazelix action registry adapter | Keep in Yazelix; it depends on Yazelix-owned action ids and generated integration commands |
| `[opener].edit` preservation | Child renderer behavior for the Yazelix config-pack template | Keep in the child renderer unless the full adapter boundary moves |
| Managed Yazi home under `~/.config/yazelix/yazi/` | Yazelix user Yazi config ownership | Keep in Yazelix; import, config UI, and Yazi package state use this vocabulary |
| Generated output under `~/.local/share/yazelix/configs/yazi/` | Yazelix runtime state | Keep in Yazelix; it is not a user-editable source tree |
| `repo_update_workflow.rs` vendored plugin refresh | Removed from Yazelix main repo | Recreate only inside `yazelix-yazi-assets` if the child repo needs automated upstream refresh |

## Delete-First Decisions

Legacy `configs/yazi/user/*` override docs are obsolete. Runtime generation now
rejects that directory and tells users to import or move overrides into
`~/.config/yazelix/yazi/`. The stale guide should not ship because it advertises
a path that no longer works.

Generated runtime files are not source templates. Documentation should not tell
users to edit `~/.config/yazelix/configs/yazi/yazelix_*.toml`; supported
customization lives in `settings.jsonc` plus the managed Yazi home under
`~/.config/yazelix/yazi/`.

Do not delete the legacy error path in `yazi_materialization.rs` yet. It is the
guard that prevents silent adoption of old mutable config locations.

## Extraction Readiness

Full Yazi integration extraction is deferred.

The asset and config-pack movement is complete: `configs/yazi/` in this
repository keeps only the README and Yazelix-owned sidebar/editor plugins, while
the packaged runtime links reusable flavors, Starship config, `auto-layout.yazi`,
`git.yazi`, `lazygit.yazi`, and `starship.yazi` from `yazelix-yazi-assets`.
The child crate owns render-plan metadata, generated config templates, TOML/Lua
merge behavior, and equivalence tests.

The private writer/adapter split is complete, but the writer is not a public
Yazi integration API. It still receives Yazelix-managed output roots, reads
managed sidecars, expands semantic keybindings, and owns runtime-state writes.
Only after the adapter is thin should a public Yazi integration repository be
considered. Main must not grow fallback copies of child templates, renderer
logic, or render-plan metadata.

## LOC Scorecard

Current Yazi surface measured on 2026-06-15:

| Surface | Lines | Notes |
| --- | ---: | --- |
| `rust_core/yazelix_core/src/yazi_materialization.rs` | 656 | Yazelix adapter for config normalization, managed paths, semantic keybindings, and legacy guard |
| `rust_core/yazelix_core/src/yazi_materialization/writer.rs` | 586 | Private writer that calls the child renderer and syncs runtime assets |
| `rust_core/yazelix_core/tests/yzx_core_yazi_materialization.rs` | 546 | Behavior coverage for generated files, assets, keybindings, and legacy rejection |
| `configs/yazi/` | 349 | Main repo keeps only README and sidebar/editor integration plugins |
| `yazelix-yazi-assets/` | 7,952 | Child repo package containing 24 flavors, reusable Yazi plugins, Starship config, render-plan metadata, generated config templates, package metadata, lockfile, and pinned upstream refresh metadata |

Main-repo `configs/yazi/` and Rust config-pack ownership shrank. Future
reduction comes from thinning the remaining adapter and broad tests, not from
re-creating asset or renderer mirrors in this repository.

## Verification

- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core yazi_materialization`
- `cargo test` in `yazelix-yazi-assets`
- `nix build .#yazelix_yazi_assets --no-link --no-write-lock-file`
- `yzx_repo_validator validate-contracts`
- `yzx_repo_validator validate-docs-experience`
