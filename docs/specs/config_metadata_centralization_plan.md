# Config Metadata Centralization Plan

## Summary

Yazelix should centralize machine-meaningful config metadata instead of repeating the same semantic contract across the default TOML template, Home Manager option definitions, Nushell parser defaults, schema validation, and rebuild-sensitivity metadata. The right goal is not “one giant generated config system,” but one canonical data model for defaults, validation, ownership, and rebuild sensitivity, with user-facing prose and integration-specific descriptions kept as intentionally separate layers.

## Why

The current system repeats the same meaning in too many places:

- `yazelix_default.toml` defines user-facing keys and defaults
- `home_manager/module.nix` defines Nix-side defaults, types, and emits TOML again
- `nushell/scripts/utils/config_parser.nu` repeats defaults and validation rules
- deleted `nushell/scripts/utils/config_schema.nu` no longer repeats enum and schema rules
- `config_metadata/main_config_contract.toml` is now the canonical artifact, but not every consumer has been migrated to it yet

That repetition already drifts. One concrete example today is `yazi.plugins`:

- the default TOML template ships `["git", "starship"]`
- Home Manager defaults to `["git", "starship"]`
- the parser fallback still defaults to `["git"]`

This is the exact kind of subtle mismatch that a centralized contract should prevent.

## Scope

- define which duplication should be deleted
- define which duplication should remain but be validated explicitly
- propose the canonical shape for centralized config metadata
- propose the follow-up implementation phases

## Behavior

- Yazelix should have one canonical machine-readable config contract for the main config surface and one canonical machine-readable pack catalog for the pack surface.
- The canonical machine-readable config contract should be language-neutral so both Nushell and Nix can consume it without one language becoming the hidden owner. A checked-in TOML or JSON artifact is the preferred direction.
- The first canonical artifact now lives at `config_metadata/main_config_contract.toml`.
- The canonical config contract should carry the semantic fields that are currently duplicated mechanically:
  - config path such as `core.refresh_output`
  - owning surface such as `main` or `packs`
  - default value
  - value kind such as bool, string, int, float, list, nullable string
  - allowed enum values or numeric range where applicable
  - rebuild sensitivity
  - parser output key name where it differs from the TOML path
  - Home Manager option name where it differs from the TOML path
  - serialization hints only where they are actually needed
- The pack catalog should be treated as a separate canonical artifact instead of being folded into the same scalar-option table. Pack declarations are structured content, not just another primitive option default.
- The canonical pack artifact now lives at `config_metadata/pack_catalog_contract.toml`.
- The pack artifact should own:
  - pack-sidecar defaults for `enabled` and `user_packages`
  - the canonical `declarations` mapping
  - the machine-readable boundary between pack-surface semantics and the main config contract
- The first consumers of the canonical config contract should be:
  - Nushell parser defaults and validation metadata
  - rebuild-required key metadata
  - parity validation work in `yazelix-gcng`
- The first consumers of the canonical pack artifact should be:
  - future pack-template parity work
  - Home Manager pack-declaration parity work
  - future pack-surface validation without overloading the main config contract
- Human-facing prose should remain intentionally duplicated where audiences differ:
  - explanatory comments in `yazelix_default.toml`
  - Home Manager option descriptions
  - docs examples and migration notes
- That prose duplication should not be treated as the semantic source of truth. It should be validated or spot-checked against the machine-readable contract instead of driving it.
- Generated artifacts should remain derived:
  - Home Manager-generated `user_configs/yazelix.toml`
  - Home Manager-generated `user_configs/yazelix_packs.toml`
  - runtime-generated files under `~/.local/share/yazelix/configs/`
- The centralization plan should be phased:
  - Phase 1: centralize machine-meaningful metadata for the main config surface
  - Phase 2: make Nushell parser/schema/rebuild metadata consume that contract
  - Phase 3: add a high-signal parity validator using the contract
  - Phase 4: centralize the pack catalog separately
  - Phase 5: only then evaluate whether any Home Manager option stanzas or default-template fragments are worth partially generating

## Non-goals

- generating every comment block in `yazelix_default.toml`
- generating every Home Manager option description from one blob of text
- moving semantic ownership entirely into Nix or entirely into Nushell
- collapsing main config and pack catalog into one undifferentiated metadata file
- performing the full centralization refactor in this bead

## Acceptance Cases

1. There is an explicit decision on which duplication Yazelix should delete first: machine-meaningful defaults, validation, and rebuild metadata rather than user-facing prose.
2. The plan distinguishes between the main config contract and the pack catalog instead of treating all duplication as one problem.
3. The plan identifies at least one real drift example from the current codebase to justify the refactor direction.
4. The plan states which downstream work should consume the centralized contract first and which duplication is acceptable to keep.
5. Follow-up implementation work can be broken into narrower beads without redefining the target architecture.

## Verification

- manual review: compare `yazelix_default.toml`, `home_manager/module.nix`, `config_metadata/main_config_contract.toml`, and the Rust `config_normalize.rs` consumers
- manual review: confirm the `yazi.plugins` default drift example in the current tree

## Traceability

- Bead: `yazelix-0oj1`
- Defended by: `manual comparison of current config metadata duplication sites in yazelix_default.toml, home_manager/module.nix, main_config_contract.toml, and Rust config normalization consumers`

## Open Questions

- Should the canonical machine-readable contract live in one `config_contract.toml`, or should main config metadata and pack catalog metadata be split into separate artifacts from the start?
- Should `yazelix_default.toml` eventually be rendered from the contract, or should it remain hand-authored and merely validated for semantic parity?
- How much Home Manager code generation is actually worth the complexity once parity validation exists?
