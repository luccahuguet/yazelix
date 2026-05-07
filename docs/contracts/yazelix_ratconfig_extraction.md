# Yazelix Ratconfig Extraction

## Summary

`yazelix_ratconfig` is the selected future name for a reusable Ratatui config editor toolkit proven first inside Yazelix.

The extraction is not ready for a standalone repository or public crate yet. The current correct step is an internal boundary split that separates reusable config-editing concepts from Yazelix-specific ownership, materialization, and apply policy.

Alternatives considered:

- `yazelix_config_ui`: accurate for the current product surface, but too tied to Yazelix as an application instead of a reusable Ratatui toolkit
- `ratconfig`: concise and generic, but too likely to conflict with unrelated projects and less clearly connected to the Yazelix extraction family
- keeping the config UI internal indefinitely: acceptable as a fallback, but it leaves a large reusable Ratatui/schema editing surface trapped in `yazelix_core`

## Readiness Decision

The extraction readiness state is `private_boundary_active`.

Yazelix should not publish `yazelix_ratconfig` yet. The in-repo private boundary now lives under `rust_core/yazelix_core/src/yazelix_ratconfig/` and owns the reusable model, editor, and render modules. `rust_core/yazelix_core/src/config_ui.rs` remains the Yazelix adapter.

The private boundary currently covers:

- schema-backed field inventory and grouping
- editor state and actions
- terminal rendering
- reusable validation-facing diagnostics
- apply-status display data

The Yazelix adapter still covers:

- persistence and patch application
- ownership and apply-status adaptation
- settings/schema/contract loading
- Home Manager and native config status
- generated runtime refreshes

The current `rust_core/yazelix_core/src/config_ui.rs` surface is product-useful, but it is still too coupled to Yazelix settings, native-config ownership, Home Manager read-only behavior, JSONC save semantics, generated runtime refreshes, and status wording. Publishing that shape would fossilize a Yazelix adapter as if it were a reusable toolkit API.

## Crate Shape

The first implementation shape should remain inside the Yazelix repository.

The current focused modules under `yazelix_core/src/yazelix_ratconfig/` can still change without release promises:

- field and section model
- editor action model
- renderer helpers
- validation diagnostic model

The still-deferred internal pieces are write-plan or patch-plan traits. Yazelix keeps settings metadata, JSONC patching, Home Manager ownership, native config status, and runtime apply modes in the adapter until those traits are proven by real saves and a second fixture.

A public `yazelix_ratconfig` crate or standalone repository becomes acceptable only after Yazelix consumes that internal API for real saves and the reusable layer can be demonstrated with a small non-Yazelix fixture schema.

## Reusable Boundary

The reusable layer may own:

- typed field metadata such as id, label, description, type, default, allowed values, grouping, and read-only state
- section, tab, and search indexing models
- scalar editors for strings, numbers, booleans, enums, lists, and enum-list enablement
- editor navigation and edit actions independent of Yazelix command text
- validation diagnostics with severity, field id, message, and optional remediation text
- theming hooks that do not encode Yazelix colors as a mandatory palette
- a persistence interface that reports changed fields and write/apply outcomes

The reusable layer must not own:

- the Yazelix `settings.jsonc` schema
- Home Manager ownership rules
- generated runtime materialization
- native Helix, Yazi, Zellij, or Ghostty config integration status
- pane-orchestrator runtime reloads
- Yazelix command names, docs text, or sidecar file semantics

## Yazelix Adapter Boundary

Yazelix remains responsible for translating product state into the reusable model.

The adapter owns:

- loading `~/.config/yazelix/settings.jsonc`
- mapping `main_config_contract.toml` and schema metadata into fields
- preserving JSONC comments when saving supported edits
- displaying Home Manager-owned settings as read-only
- displaying native, managed, imported, generated, and read-only status vocabulary
- attaching apply-mode metadata and saved-versus-active status
- running generated runtime refreshes after saves
- deciding when a running pane, tab, or session must be restarted

The reusable toolkit should receive those facts as data. It should not rediscover Yazelix paths or infer Yazelix runtime policy.

## Config Format Assumptions

JSONC is the first real persistence backend because it is Yazelix's canonical user config format.

The reusable API should not be permanently JSONC-only. It should model writes as field changes and backend-specific patch plans. A JSONC backend can be the first implementation, but the core editor model should be able to work with TOML, YAML, or application-owned serializers later.

When a backend cannot preserve comments or ordering, that must be visible in the write plan or save result. Silent rewrites are not acceptable as a default.

## Editing And Validation Guarantees

The extraction must preserve Yazelix's current safety expectations:

- invalid edits fail before writing when validation can catch them locally
- write failures are visible and actionable
- read-only fields are not editable through keyboard shortcuts or save actions
- generated-runtime refresh failures are reported after the setting is saved
- no fallback should make a failed save look successful
- unknown fields in user config are not discarded by ordinary supported edits

The reusable layer may provide generic diagnostics, but Yazelix owns product-specific remediation text.

## Maintenance Cost

Extraction is worthwhile only if it reduces the main repo's long-term coupling.

The extraction is not worthwhile if it creates:

- a second schema language beside the existing Yazelix config contract
- duplicated JSONC patching logic
- duplicated validation messages in the config UI and CLI
- public compatibility promises for unstable Yazelix-specific editing flows
- extra release automation before a non-Yazelix consumer exists

The near-term maintenance win is splitting the existing file by responsibility. Public packaging comes later.

## Migration Plan

1. Split the in-repo config UI into model, editor state/actions, rendering, validation, and Yazelix adapter modules without changing behavior. Completed as a private `yazelix_ratconfig` namespace plus `config_ui` adapter.
2. Keep JSONC patching and apply-mode handling behind adapter-owned functions while the split settles.
3. Add one small non-Yazelix fixture schema in tests to prove the model is not hardcoded to Yazelix settings.
4. Keep Yazelix as the only shipped consumer until real save, read-only, diagnostics, and generated-refresh flows use the split boundary.
5. Re-evaluate a public `yazelix_ratconfig` crate or repository after the internal boundary survives normal Yazelix config UI changes.

## Proof Plan

The first proof remains Yazelix itself:

- edit ordinary scalar settings in the config UI
- edit enum and enum-list settings
- preserve JSONC comments on supported saves
- show Home Manager-owned settings as read-only
- show saved-versus-active apply status
- surface generated-runtime refresh failures as save/apply errors

The second proof should be a small synthetic config fixture that uses the same reusable model but no Yazelix paths, contracts, or runtime concepts.

## Verification

- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core config_ui`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core settings_jsonc_patch`
- `yzx_repo_validator validate-config-surface-contract`
- `yzx_repo_validator validate-contracts`

## Traceability

- Defended by: `yzx_repo_validator validate-contracts`
