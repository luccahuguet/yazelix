# Runtime Root Contract

## Summary

Yazelix must treat the user config root, the shipped runtime root, and the generated state root as three separate locations with different owners. Normal usage must not require a source checkout living under `~/.config/yazelix`.

These three roots line up with three filesystem-backed kinds of state plus one process-local layer:

1. Dynamic user intent
   - config under `~/.config/yazelix/user_configs/`
2. Deterministic runtime code
   - the shipped runtime tree from the flake, package, or repo checkout
3. Materialized/generated state
   - generated configs, cached hashes, launch state, and other derived artifacts under `~/.local/share/yazelix`
4. Live session activation state
   - process-local markers such as `DEVENV_PROFILE`, profile-derived `PATH`, `IN_NIX_SHELL`, `IN_YAZELIX_SHELL`, `YAZELIX_TERMINAL`, and Zellij session markers

## Why

Yazelix already has helpers for `YAZELIX_CONFIG_DIR`, `YAZELIX_RUNTIME_DIR`, and `YAZELIX_STATE_DIR`, but parts of the product still collapse those concerns back into one hardcoded path. That keeps the source checkout requirement alive in practice even when the intended architecture says otherwise.

Without a sharper contract:

- terminal and desktop launchers keep assuming runtime scripts live under `~/.config/yazelix`
- bundled integrations like Yazi shell-outs keep invoking source-checkout paths
- packaged/runtime installs remain second-class even though the codebase already has split-root helpers

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
    - `user_configs/yazelix_packs.toml`
    - other user-managed overrides under `user_configs/`
- The runtime root is the shipped Yazelix asset tree used at runtime.
  - Canonical environment variable: `YAZELIX_RUNTIME_DIR`
  - It may be:
    - a source checkout during maintainer work
    - an installed runtime tree from a package or generated deployment
  - Contents include shipped scripts, layouts, bundled plugins, templates, and other runtime assets Yazelix executes or reads directly.
  - It is deterministic product code tied to the repo or packaged runtime revision, not mutable user config state.
- The state root is the generated and cached Yazelix data surface.
  - Default location: `~/.local/share/yazelix`
  - Canonical environment variable: `YAZELIX_STATE_DIR`
  - Contents include generated configs, cached launch-profile state, rebuild state, and other derived runtime artifacts.
  - It is the materialized result of combining user intent with the shipped runtime.
- Live session activation state has no canonical filesystem root.
  - It is the current process-local activation of a built profile and session.
  - It includes values such as `DEVENV_PROFILE`, profile-derived `PATH`, `IN_NIX_SHELL`, `IN_YAZELIX_SHELL`, `YAZELIX_TERMINAL`, and Zellij session markers.
  - It may be stale even while the runtime root and state root are correct.
  - It must not be treated as persisted runtime truth.
- `YAZELIX_DIR` is a legacy compatibility alias for the runtime root only.
  - New code should prefer `YAZELIX_RUNTIME_DIR` and `YAZELIX_CONFIG_DIR` explicitly.
  - Code must not treat `YAZELIX_DIR` as the user config root.
- User-facing runtime entrypoints must resolve shipped assets through the runtime root, not by assuming a repo clone under `~/.config/yazelix`.
  - Examples: terminal wrappers, desktop launchers, `yzx` menu/popup helpers, bundled Yazi plugins, editor integration scripts.
- User-facing config lookups must resolve through the config root, not through the runtime root.
- Generated configs and cached state must resolve through the state root or the derived runtime config paths, not through the source checkout.
- Runtime/profile identity should not be inferred from whichever shell the user happens to be sitting in.
  - Maintainer shells may still hold an older `DEVENV_PROFILE`.
  - That older shell profile is stale live activation state, not necessarily stale materialized launch state.
  - Reusable launch state should be recorded from real launch/refresh flows against the runtime project state, not from shell-hook setup alone.
  - External launch helpers should sanitize inherited activation markers instead of assuming the current shell is the authoritative runtime session.
- Maintainer-only workflows may still assume a source checkout when the task is explicitly about repository maintenance.
  - Examples: release automation, source validators, README syncing, issue/bead reconciliation, repo-local dev helpers.
  - Those assumptions should stay explicit instead of leaking into normal user entrypoints.

## Non-goals

- defining the final packaging format for Yazelix
- removing source-checkout workflows for maintainers
- redesigning every environment variable in one step
- moving user config out of `~/.config/yazelix`

## Acceptance Cases

1. A normal installed Yazelix runtime can launch without a git clone at `~/.config/yazelix`, as long as the runtime root, config root, and state root are present in their supported locations.
2. Bundled runtime integrations invoke shipped scripts through the runtime root instead of hardcoded source-checkout paths under `~/.config/yazelix`.
3. User config continues to load from the config root even when the runtime root is somewhere else.
4. Maintainer-only commands that still require a source checkout are explicit about that requirement instead of being used silently by normal user entrypoints.
5. Reinstalling Yazelix from a repo shell does not silently redefine launch-profile ownership or treat the current maintainer shell as the launched runtime.
6. New path-model work can classify each lookup as config-owned, runtime-owned, state-owned, or activation-only without guessing.

## Verification

- manual review of runtime entrypoints against this contract
- integration tests that run with split `YAZELIX_CONFIG_DIR` and `YAZELIX_RUNTIME_DIR`
- config/runtime path checks in `nushell/scripts/dev/test_yzx_generated_configs.nu`
- core command checks in `nushell/scripts/dev/test_yzx_core_commands.nu`

## Traceability

- Bead: `yazelix-hac.2`
- Defended by: `nu nushell/scripts/dev/test_yzx_generated_configs.nu`
- Defended by: `nu nushell/scripts/dev/test_yzx_core_commands.nu`

## Open Questions

- Should `YAZELIX_DIR` eventually disappear entirely once all supported user entrypoints are on explicit runtime/config root variables?
- Which shipped assets, if any, should move out of the runtime root and into the state root during package-ready work?
- Should future runtime helpers expose activation-only markers through dedicated helper names so they stop looking like alternate runtime roots?
