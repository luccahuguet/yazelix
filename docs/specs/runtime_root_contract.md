# Runtime Root Contract

## Summary

Yazelix treats the config root, runtime root, and state root as separate locations with different owners. The trimmed v15 branch no longer treats installer-owned launch profiles, pack sidecars, or the old runtime-local `devenv` layer as part of the normal user contract.

These roots line up with three filesystem-backed kinds of state plus one process-local layer:

1. Dynamic user intent
   - config under `~/.config/yazelix/user_configs/`
2. Deterministic runtime code
   - the shipped runtime tree from the package, flake output, or repo checkout
3. Materialized/generated state
   - generated configs, initializers, logs, rebuild hashes, and other derived artifacts under `~/.local/share/yazelix`
4. Live session activation state
   - process-local markers such as `IN_YAZELIX_SHELL`, `YAZELIX_TERMINAL`, Zellij session markers, and maintainer-shell activation markers from `nix develop`

## Why

The branch is smaller now, but path confusion is still one of the easiest ways to reintroduce old v14 assumptions:

- treating the config root like a source checkout
- treating the current shell as runtime truth
- treating generated state as handwritten config
- treating legacy aliases such as `YAZELIX_DIR` as canonical ownership

This contract keeps those boundaries explicit.

## Scope

- define the ownership and meaning of the config root
- define the ownership and meaning of the runtime root
- define the ownership and meaning of the generated state root
- define the intended role of `YAZELIX_DIR` during the transition
- define which surfaces may still assume a live source checkout

## Behavior

- The config root is the user-owned configuration surface.
  - Default location: `~/.config/yazelix`
  - Canonical environment variable: `YAZELIX_CONFIG_DIR`
  - Contents include:
    - `user_configs/yazelix.toml`
    - user-managed overrides such as `user_configs/zellij/`, `user_configs/yazi/`, `user_configs/helix/`, and `user_configs/shells/`
  - The trimmed v15 line does not treat `yazelix_packs.toml` as part of the current config contract.
- The runtime root is the shipped Yazelix asset tree used at runtime.
  - Canonical environment variable: `YAZELIX_RUNTIME_DIR`
  - It may be:
    - a source checkout during maintainer work
    - an installed runtime tree from the package or compatibility installer
  - Contents include shipped scripts, layouts, bundled plugins, templates, a curated interactive tool surface, and the runtime-private helper closure under `libexec/`.
  - It is deterministic product code tied to the repo or packaged runtime revision, not mutable user config state.
- The state root is the generated and cached Yazelix data surface.
  - Default location: `~/.local/share/yazelix`
  - Canonical environment variable: `YAZELIX_STATE_DIR`
  - Contents include generated configs, shell initializers, logs, rebuild hashes, and other derived runtime artifacts.
  - It is the materialized result of combining user intent with the shipped runtime.
- Live session activation state has no canonical filesystem root.
  - It is the current process-local activation of a runtime/session.
  - It includes values such as `IN_YAZELIX_SHELL`, `YAZELIX_TERMINAL`, `ZELLIJ`, `ZELLIJ_SESSION_NAME`, `ZELLIJ_PANE_ID`, and related session-local markers.
  - Maintainer shells may also carry extra activation markers from `nix develop`, but that is maintainer activation state, not normal user runtime truth.
  - It must not be treated as persisted runtime truth.
- `YAZELIX_DIR` is a legacy compatibility alias only.
  - New code should prefer `YAZELIX_RUNTIME_DIR` and `YAZELIX_CONFIG_DIR` explicitly.
  - User entrypoints and maintainer shells should clear or ignore inherited `YAZELIX_DIR` rather than trusting it as canonical runtime identity.
- User-facing runtime entrypoints must resolve shipped assets through the runtime root, not by assuming a repo clone under `~/.config/yazelix`.
  - Examples: terminal wrappers, desktop launchers, `yzx` menu/popup helpers, bundled Yazi plugins, editor integration scripts.
- User-facing config lookups must resolve through the config root, not through the runtime root.
- Generated configs and cached state must resolve through the state root or the derived runtime config paths, not through the source checkout.
- The normal user runtime contract is the fixed packaged runtime plus explicit update ownership.
  - `yzx update upstream` owns default-profile installs of `#yazelix`
  - `yzx update home_manager` owns Home Manager installs
  - the flake no longer exposes `#install`
- Maintainer-only workflows may still assume a source checkout and `maintainer_shell.nix` when the task is explicitly about repository maintenance.
  - Examples: release automation, source validators, repo-local profiling, issue/bead reconciliation, and repo-shell development helpers.
  - Those assumptions should stay explicit instead of leaking into normal user entrypoints.

## Non-goals

- defining the final packaging format for every future Yazelix release
- removing source-checkout workflows for maintainers
- redesigning every environment variable in one step
- moving user config out of `~/.config/yazelix`

## Acceptance Cases

1. A normal installed Yazelix runtime can launch without a git clone at `~/.config/yazelix`, as long as the runtime root, config root, and state root are present in supported locations.
2. Bundled runtime integrations invoke shipped scripts through the runtime root instead of hardcoded source-checkout paths under `~/.config/yazelix`.
3. User config continues to load from the config root even when the runtime root is somewhere else.
4. Generated configs, logs, and repair hashes remain derived state rather than becoming user-owned config.
5. A repo maintainer shell can exist without becoming part of the normal user runtime contract.

## Verification

- config/runtime path checks in `nushell/scripts/dev/test_yzx_generated_configs.nu`
- workspace/runtime launch checks in `nushell/scripts/dev/test_yzx_workspace_commands.nu`
- maintainer-shell runtime-boundary checks in `nushell/scripts/dev/test_yzx_maintainer.nu`
- installed-runtime validation in `nushell/scripts/dev/validate_installed_runtime_contract.nu`

## Traceability

- Bead: `yazelix-qgj7.2.4.3`
- Defended by: `nu nushell/scripts/dev/test_yzx_generated_configs.nu`
- Defended by: `nu nushell/scripts/dev/test_yzx_workspace_commands.nu`
- Defended by: `nu nushell/scripts/dev/test_yzx_maintainer.nu`
- Defended by: `nu nushell/scripts/dev/validate_installed_runtime_contract.nu`

## Open Questions

- Should `YAZELIX_DIR` eventually disappear entirely once all supported entrypoints use explicit runtime/config roots?
- Which shipped assets, if any, should later move out of the runtime root and into a more sharply named generated-state subtree?
