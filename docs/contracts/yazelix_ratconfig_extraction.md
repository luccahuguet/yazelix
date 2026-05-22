# Yazelix Ratconfig Extraction

## Summary

`yazelix-ratconfig` is a first-party Rust child repository for reusable Ratatui config editing.

The child repo owns the project-agnostic config UI core: model, navigation, edit state, rendering, JSONC patch primitives, and deterministic migration primitives. Yazelix remains the first consumer and keeps only the adapter code that knows about Yazelix settings, Home Manager ownership, runtime refreshes, and generated config behavior.

JSONC is the first supported persistence adapter because `settings.jsonc` is Yazelix's canonical user config format. A TOML adapter is a future decision, not part of the initial extraction contract.

## Extraction State

The extraction state is `complete_jsonc_first`.

The separate `yazelix-ratconfig` repository owns the reusable code and tests. Yazelix consumes the published child crate through Cargo/Nix dependency metadata, and the old in-repo reusable `rust_core/yazelix_core/src/yazelix_ratconfig/` implementation has been deleted instead of kept as a duplicate copy.

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

- Yazelix `settings.jsonc` schema or `main_config_contract.toml`
- Home Manager ownership rules
- generated runtime materialization
- native Helix, Yazi, Zellij, or Ghostty integration policy
- pane-orchestrator runtime reloads
- Yazelix action registries, command names, docs text, or sidecar semantics

## Yazelix Adapter Ownership

Yazelix remains responsible for translating product state into the reusable ratconfig model.

The main repo owns:

- locating `~/.config/yazelix/settings.jsonc` and related runtime metadata
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

## Migration Contract

Ratconfig migrations are config arithmetic over a document.

The first migration engine should support simple deterministic operations:

- rename
- delete
- add default
- narrow value transform

The engine owns operation semantics and ordering. The application owns which migrations apply to its config version and how to write the result atomically.

More complex array reshaping, broad type migrations, cross-file migrations, and TOML support are future decisions.

## Verification

Before extraction:

- generic ratconfig tests prove model/edit/render behavior without Yazelix dependencies
- JSONC patch tests prove comment-preserving set/unset behavior
- one non-Yazelix fixture exercises bool, scalar select, multiselect, diagnostics, and JSONC editing

After extraction:

- child repo Rust tests pass
- Yazelix config UI tests pass against the child dependency
- `yzx_repo_validator validate-config-surface-contract` passes
- `yzx_repo_validator validate-contracts` passes
- the main-repo LOC scorecard records deleted ownership
