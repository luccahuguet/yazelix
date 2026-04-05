# Config Surface And Launch Profile Contract

## Summary

Yazelix should treat configuration surfaces, generated runtime state, cached launch profiles, and live session activation as four distinct concerns with explicit ownership. The canonical user-facing config surfaces are the managed TOML files under `~/.config/yazelix/user_configs/`, while Home Manager is an integration that renders the same user intent into those surfaces rather than inventing separate semantics. Generated runtime configs under `~/.local/share/yazelix/configs/` and cached launch-profile state under `~/.local/share/yazelix/state/` are derived artifacts, not user-owned sources of truth. Process-local activation markers are a separate runtime layer again, not persisted launch truth.

More concretely, Yazelix has four layers that should not collapse into one another:

1. Dynamic user intent
   - `yazelix.toml`
   - `yazelix_packs.toml`
   - Home Manager when it renders the same effective intent
2. Deterministic runtime code
   - the shipped runtime tree from the flake or repo checkout
   - Nushell scripts, wrappers, defaults, templates, and bundled assets
3. Materialized environment and generated state
   - the `devenv` profile or shell that results from combining runtime code with user intent
   - generated configs under `~/.local/share/yazelix/configs/`
   - cached rebuild and launch state under `~/.local/share/yazelix/state/`
4. Live session activation state
   - `DEVENV_PROFILE`
   - profile-derived `PATH` activation
   - `IN_NIX_SHELL`, `IN_YAZELIX_SHELL`
   - `YAZELIX_TERMINAL` and other session-local markers such as Zellij session state

The config layer expresses what the user wants. The runtime layer expresses what Yazelix is. The materialized layer is the result of combining those two. The activation layer is the process-local use of that result right now.

## Why

Yazelix already has working machinery for split config surfaces, generated runtime configs, and cached fast-launch profiles, but the ownership model is still spread across helpers and migration logic. That makes downstream work guess:

- which file really owns a setting
- which duplication is intentional versus accidental
- what must stay in sync between TOML and Home Manager
- which changes invalidate the reusable launch profile
- what a validator should defend versus ignore

Without a written contract, later cleanup work risks centralizing the wrong thing or validating noisy pseudo-invariants.

## Scope

- define the canonical user config surfaces
- define the ownership boundary for Home Manager
- define which files are generated runtime artifacts
- define what a launch profile is and how validity is determined
- define who owns launch-state recording versus shell-hook/setup work
- define where parity and validation are required

## Behavior

- The canonical managed user config surfaces are:
  - `~/.config/yazelix/user_configs/yazelix.toml`
  - `~/.config/yazelix/user_configs/yazelix_packs.toml`
- The shipped defaults are runtime templates, not the active user config:
  - `yazelix_default.toml`
  - `yazelix_packs_default.toml`
- On first run, when the managed user config surfaces do not exist, Yazelix bootstraps them from the shipped defaults.
- `yazelix.toml` owns the main user-facing Yazelix configuration.
- `yazelix_packs.toml` owns pack configuration when it exists. If `yazelix_packs.toml` exists, Yazelix must fail fast when pack settings are also defined in `yazelix.toml`.
- Legacy root-level managed config files under `~/.config/yazelix/` are no longer canonical. Yazelix should relocate them into `user_configs/` or fail fast if both old and canonical copies exist.
- `YAZELIX_CONFIG_OVERRIDE` is a development and testing override for selecting a different main config file. It is not a third normal user config surface.
- Home Manager is an integration surface. Its job is to generate the same effective user config that manual TOML editing would express. It must track the same schema and defaults as the managed TOML surfaces rather than introducing separate product semantics.
- The shipped runtime tree is deterministic product code, not mutable user state.
  - It may come from a repo checkout during maintainer work.
  - It may come from an installed flake/package runtime for normal usage.
  - It owns shipped scripts, wrappers, defaults, templates, and bundled assets.
- Generated runtime configs and merged downstream configs under `~/.local/share/yazelix/configs/` are derived artifacts. Users and maintainers should treat them as generated outputs, not as canonical handwritten config.
- A launch profile is the cached `devenv` profile path plus the recorded config/input hash that proved that profile was built for the current rebuild-relevant configuration.
- Launch-state ownership is intentionally narrow:
  - shell-hook/setup work may materialize generated configs and mark rebuild inputs as applied
  - only real runtime entry or refresh flows that own a successful built profile should record launch-profile state
  - install/setup paths must not overwrite launch-profile state merely because they ran in some shell
- Live session activation state is intentionally separate from launch-state recording:
  - `launch_state.json` and rebuild hashes are persisted materialized state
  - `DEVENV_PROFILE`, profile-derived `PATH`, `IN_NIX_SHELL`, `IN_YAZELIX_SHELL`, `YAZELIX_TERMINAL`, and Zellij session markers are activation-only markers
  - activation markers may be stale without invalidating a newer recorded launch profile
- Launch-profile validity is determined by:
  - the recorded profile path still existing
  - the recorded combined hash matching the current combined hash
  - the required synced runtime assets still existing
- For `devenv build shell` flows, Yazelix should resolve the embedded `DEVENV_PROFILE` from the generated shell artifact instead of treating the shell script path itself as the reusable profile.
- The `devenv build shell` command output is the canonical build-time evidence for the fresh reusable profile.
  - The shell path reported in build output must be resolved back to its embedded `DEVENV_PROFILE`.
  - A successful build-owned refresh/startup/restart path may record that fresh profile before the current shell or window has activated it.
  - In that case, `launch_state.json.profile_path` is allowed to be newer than the current process-local `DEVENV_PROFILE` until the explicit activation boundary happens.
  - Runtime-project `.devenv/profile` and `.devenv/gc/shell` entries under `~/.local/share/yazelix/runtime/project` are secondary build artifacts only.
  - Those runtime-project `.devenv` entries may be absent after a fresh build in a new state root, so launch correctness must not require them.
  - Helpers may use them as fallback evidence only when they already exist.
- The current combined hash is derived from:
  - rebuild-relevant Yazelix config keys
  - `devenv.lock`
  - `devenv.nix`
  - `devenv.yaml`
- The current maintainer shell is not automatically the same thing as the current launch profile.
  - A repo-local or maintainer shell may still hold an older `DEVENV_PROFILE`.
  - That older shell profile is stale live activation state, not proof that persisted launch state is wrong.
  - Launch/restart/refresh logic should prefer the runtime-owned recorded or freshly built profile unless it is already operating inside a live launched Yazelix session.
- External launch helpers should clear inherited live activation markers before starting a new session, rather than treating the current shell as canonical launch truth.
- Changes outside the rebuild-relevant config subset do not require a new `devenv` profile build, but they still remain part of the active Yazelix behavior and should apply through normal config parsing and runtime regeneration.
- `yzx launch --reuse` is allowed to reuse the last recorded launch profile even when the current config/input hash is stale, but only when a cached profile actually exists. It does not promise that local config changes are applied.
- Validators should defend explicit ownership and parity boundaries only:
  - Home Manager schema/default parity with the canonical managed TOML contract
  - pack-surface ownership rules
  - launch-profile validity invariants that Yazelix explicitly relies on
- Validators should not enforce generated-file byte identity or other noisy invariants unless the contract explicitly makes them canonical.

## Non-goals

- refactoring the implementation into a centralized config metadata table in this spec alone
- removing Home Manager support
- collapsing all config into a single file
- making generated runtime config user-editable
- redefining every individual config key

## Acceptance Cases

1. When a user starts Yazelix with no managed user config surfaces yet, Yazelix bootstraps `user_configs/yazelix.toml` and `user_configs/yazelix_packs.toml` from the shipped defaults, and those new files become the canonical user-editable config surfaces.
2. When `yazelix_packs.toml` exists, pack settings defined in `yazelix.toml` are rejected with a clear ownership error instead of being merged heuristically.
3. When both canonical `user_configs` files and legacy root-level managed config files exist, Yazelix fails fast and asks the user to keep one clear owner instead of guessing.
4. When Home Manager is enabled, it renders the same config semantics and defaults as the managed TOML contract rather than creating a divergent configuration model.
5. When only non-rebuild runtime settings change, Yazelix may still use the existing launch profile without forcing a `devenv` rebuild, while the updated runtime behavior is still applied on the next normal launch flow.
6. When rebuild-relevant config or `devenv` inputs change, Yazelix treats the recorded launch profile as stale for normal fast-launch paths until a new matching profile is built and recorded.
7. When `yzx launch --reuse` is used, Yazelix may reuse the last recorded profile despite current config drift, but it fails clearly if no cached launch profile exists.
8. When install or shell-hook setup runs from a stale maintainer shell, it must not overwrite `launch_state.json` with that stale shell profile. Real launch and refresh flows own launch-profile recording.
9. When `yzx refresh` builds through `devenv build shell`, Yazelix records the embedded `DEVENV_PROFILE`, not the shell-script wrapper path and not an unrelated ambient maintainer-shell profile.
10. When a stale maintainer shell and a correct `launch_state.json` coexist, the docs and helpers treat that as a live-activation-versus-materialized-state split, not as contradictory runtime truth.
11. When a fresh state root runs `devenv build shell`, Yazelix can still derive and record the reusable profile from build output even if `runtime/project/.devenv/profile` and `runtime/project/.devenv/gc/shell` do not exist yet.
12. When a startup, restart, or other runtime-entry rebuild succeeds, Yazelix may record the fresh reusable profile before the newly launched session has fully activated it.
13. When a validator is added for this contract, it checks maintained ownership and parity rules rather than noisy generated-output trivia.

## Verification

- CI checks: `nu nushell/scripts/dev/validate_config_surface_contract.nu`
- unit tests: `nushell/scripts/utils/config_surfaces.nu`
- unit tests: `nushell/scripts/utils/config_state.nu`
- integration tests: `nu nushell/scripts/dev/test_yzx_generated_configs.nu`
- integration tests: `nu nushell/scripts/dev/test_yzx_core_commands.nu`
- maintainer tests: `nu nushell/scripts/dev/test_yzx_maintainer.nu`
- manual verification: inspect a manual TOML setup and a Home Manager setup to confirm they produce the same effective ownership model

## Traceability

- Bead: `yazelix-b16x`
- Defended by: `nu nushell/scripts/dev/validate_config_surface_contract.nu`
- Defended by: `nu nushell/scripts/dev/test_yzx_generated_configs.nu`
- Defended by: `nu nushell/scripts/dev/test_yzx_core_commands.nu`
- Defended by: `nu nushell/scripts/dev/test_yzx_maintainer.nu`

## Open Questions

- Should Yazelix eventually centralize config metadata such as ownership, defaults, rebuild sensitivity, and Home Manager parity in one declarative table so parsing, validation, docs, and integration code stop repeating those rules?
- Should launch-profile validity stay tied only to rebuild-relevant keys, or are there additional generated-runtime invariants that deserve to participate in the hash without creating unnecessary rebuild churn?
- Should launch-profile helpers eventually stop inferring activation-vs-materialized precedence implicitly and instead use narrower resolvers for recorded state, runtime build artifacts, and live activation markers?
