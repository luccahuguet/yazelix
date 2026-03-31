# Config Surface And Launch Profile Contract

## Summary

Yazelix should treat configuration surfaces, generated runtime state, and cached launch profiles as three distinct concerns with explicit ownership. The canonical user-facing config surfaces are the managed TOML files under `~/.config/yazelix/user_configs/`, while Home Manager is an integration that renders the same user intent into those surfaces rather than inventing separate semantics. Generated runtime configs under `~/.local/share/yazelix/configs/` and cached launch-profile state under `~/.local/share/yazelix/state/` are derived artifacts, not user-owned sources of truth.

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
- Generated runtime configs and merged downstream configs under `~/.local/share/yazelix/configs/` are derived artifacts. Users and maintainers should treat them as generated outputs, not as canonical handwritten config.
- A launch profile is the cached `devenv` profile path plus the recorded config/input hash that proved that profile was built for the current rebuild-relevant configuration.
- Launch-profile validity is determined by:
  - the recorded profile path still existing
  - the recorded combined hash matching the current combined hash
  - the required synced runtime assets still existing
- The current combined hash is derived from:
  - rebuild-relevant Yazelix config keys
  - `devenv.lock`
  - `devenv.nix`
  - `devenv.yaml`
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
8. When a validator is added for this contract, it checks maintained ownership and parity rules rather than noisy generated-output trivia.

## Verification

- unit tests: `nushell/scripts/utils/config_surfaces.nu`
- unit tests: `nushell/scripts/utils/config_state.nu`
- integration tests: `nu nushell/scripts/dev/test_yzx_generated_configs.nu`
- integration tests: `nu nushell/scripts/dev/test_yzx_core_commands.nu`
- manual verification: inspect a manual TOML setup and a Home Manager setup to confirm they produce the same effective ownership model

## Traceability

- Bead: `yazelix-b16x`
- Defended by: `nu nushell/scripts/dev/test_yzx_generated_configs.nu`
- Defended by: `nu nushell/scripts/dev/test_yzx_core_commands.nu`

## Open Questions

- Should Yazelix eventually centralize config metadata such as ownership, defaults, rebuild sensitivity, and Home Manager parity in one declarative table so parsing, validation, docs, and integration code stop repeating those rules?
- Should launch-profile validity stay tied only to rebuild-relevant keys, or are there additional generated-runtime invariants that deserve to participate in the hash without creating unnecessary rebuild churn?
