# Ratconfig Extraction

## Summary

`ratconfig` is a first-party Rust child repository for reusable Ratatui config editing.

The child repo owns the project-agnostic config UI core: model, navigation, edit state, rendering, JSONC patch primitives, TOML text adapters, and deterministic contract/migration primitives. Yazelix remains the first consumer and keeps only the adapter code that knows about Yazelix settings, Home Manager ownership, runtime refreshes, and generated config behavior.

TOML is Yazelix's persistence format for `config.toml` and the separate `cursors.toml` registry. Ratconfig's JSONC primitives remain available only for bounded Classic migration from retired `settings.jsonc` inputs.

## Extraction State

The extraction state is `complete_multi_format`.

The separate `ratconfig` repository owns the reusable code and tests. Yazelix consumes the published child crate through Cargo/Nix dependency metadata, and the old in-repo reusable `rust_core/yazelix_core/src/ratconfig/` implementation has been deleted instead of kept as a duplicate copy.

The TOML adapter decision is `accepted_child_generic`. `ratconfig` owns generic TOML text adapters, contract-state reconciliation, examples, and tests. Main Yazelix owns only the `config.toml` product schema, migration policy, Home Manager rendering, and runtime semantics.

Future work should treat the child crate as the reusable owner and the main repo as a Yazelix adapter. If the boundary is painful, improve the child API or revise the contract; do not recreate a local mirror in the main repo.

## Child Repo Ownership

The child repo owns reusable behavior that another project can use without importing Yazelix:

- config document and field model
- tabs, rows, search, selection, and edit state
- bool toggle, scalar text/number parsing, single-select, and multiselect controls
- generic detail/list rendering
- project-supplied display rows and diagnostics
- JSONC set/unset/rename/delete/add-default patch primitives
- deterministic migration operation semantics
- project-agnostic errors that adapters can map into application-specific errors
- examples and tests that do not mention Yazelix paths or actions

The child repo must not own:

- Yazelix `config.toml` schema or `main_config_contract.toml`
- Home Manager ownership rules
- generated runtime materialization
- native Helix, Yazi, Zellij, or Ghostty integration policy
- pane-orchestrator runtime reloads
- Yazelix action registries, command names, docs text, or sidecar semantics

## Yazelix Adapter Ownership

Yazelix remains responsible for translating product state into the reusable ratconfig model.

The main repo owns:

- locating `~/.config/yazelix/config.toml` and related runtime metadata
- loading defaults, schema metadata, and `main_config_contract.toml`
- composing Yazelix-specific cursor settings into the visible model
- marking Home Manager-owned settings as read-only
- expanding Yazelix keybinding action registries into structured field/detail data
- classifying native, managed, imported, generated, and read-only config status
- mapping generic ratconfig errors into `CoreError`
- validating patched settings against Yazelix normalization
- running generated runtime refreshes and pane-owner refreshes after saves
- deciding which saved settings require pane reopen, Yazelix restart, or Home Manager switch

The reusable child repo receives these facts as data. It does not rediscover Yazelix paths or infer Yazelix runtime policy.

## Public API Shape

The public crate shape is small and data-driven.

Current modules:

- `model`: document, tab, row, field, value state, diagnostics, and display metadata
- `editor`: navigation, search, edit modes, input parsing, and control actions
- `render`: Ratatui rendering for the generic model
- `jsonc`: comment-preserving JSONC patch primitives
- `toml_adapter`: deterministic TOML text adapter primitives
- `contract`: joined deterministic config contracts
- `migration`: deterministic config migration operations

The application adapter owns file IO, validation, atomic writes, model reload, and post-save apply behavior.

Project-specific rich detail sections are supplied as data. The renderer may display them, but it must not know about Yazelix keybindings, Zellij, Yazi, Home Manager, or generated config ownership.

## JSONC Contract

The initial JSONC backend must preserve comments and surrounding document shape for supported edits.

Supported first operations:

- set a dotted field path
- unset a dotted field path
- rename a dotted field path
- delete stale fields
- add missing defaults
- run narrow, deterministic value transforms supplied by the caller

Unsupported patch shapes must fail clearly instead of silently rewriting the whole document. When a project wants a whole-file rewrite, that must be an explicit adapter decision.

## TOML Adapter Decision

TOML support belongs in `ratconfig` when it stays generic:

- The child crate may use `toml` and `toml_edit` so TOML parsing and comment-preserving text edits are not recreated by hand.
- TOML and JSONC must share the same contract semantics for rename, delete, add-default, transform, joined-state reads/writes, manual blockers, and contract-id checks.
- TOML-specific limits are adapter errors, not alternate migration behavior. Examples include rejecting JSON `null` because TOML has no null value and refusing to patch through a parent path that is not a TOML table.
- The main Yazelix repo uses Ratconfig's TOML adapter for `config.toml` and `cursors.toml`; JSONC remains only at bounded migration boundaries.

## Migration Contract

Ratconfig migrations are config arithmetic over a document.

The first migration engine should support simple deterministic operations:

- rename
- delete
- add default
- narrow value transform

The engine owns operation semantics and ordering. The application owns which migrations apply to its config version and how to write the result atomically.

More complex array reshaping, broad type migrations, cross-file migrations, and application-specific TOML config surfaces are future decisions.

## Verification

Before extraction:

- generic ratconfig tests prove model/edit/render behavior without Yazelix dependencies
- JSONC patch tests prove comment-preserving set/unset behavior
- TOML adapter tests prove comment-preserving set/unset behavior, shared migration semantics, unsupported-value errors, and joined contract-state reconciliation
- one non-Yazelix fixture exercises bool, scalar select, multiselect, diagnostics, and JSONC editing

After extraction:

- child repo Rust tests pass
- Yazelix config UI tests pass against the child dependency
- `yzx_repo_validator validate-config-surface-contract` passes
- `yzx_repo_validator validate-contracts` passes
- the main-repo LOC scorecard records deleted ownership
