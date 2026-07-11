# Config Runtime Control-Plane Contract Item Pilot

## Summary

This pilot applies the canonical contract-item schema to one representative
mixed-ownership subsystem: config normalization, runtime-env derivation, helper
transport, and config-surface parity. The pilot is planning-only. It does not
change product behavior, but it does make the surviving owners, bridge debt,
verification paths, and deletion implications explicit.

## Why

Config, runtime, and control-plane glue are a good pilot because they touch all
of the protocol pressure points at once:

- Rust already owns typed config normalization and runtime-env evaluation
- the former Nushell bridge layer was deleted after Rust took direct startup and
  helper-transport ownership
- packaged runtime and source-checkout helper resolution must preserve a sharp
  failure contract
- Home Manager parity with the shipped default config is a maintained invariant,
  not just a convention
- several default-lane tests already cover this subsystem, but their
  traceability is still too broad for delete-first architecture work

## Scope

- typed config normalization ownership
- the explicit runtime-env derivation boundary
- `yzx_core` helper resolution and transport failure behavior for the
  remaining helper-only slices
- config-surface parity between `config_default.toml` and
  `home_manager/module.nix`
- pilot findings about weak traceability and duplicate-owner debt
- no product behavior changes

## Contract Items

#### CRCP-001
- Type: ownership
- Status: live
- Owner: Rust config normalization library used by the public launch/setup
  paths
- Statement: Typed normalization of the managed main Yazelix config is
  Rust-owned. Callers must not become second semantic owners for default
  merging, schema interpretation, or diagnostic classification.
- Verification: automated
  `rust_core/yazelix_core/src/config_normalize.rs`; automated
  `rust_core/yazelix_core/tests/yzx_control_runtime_surface.rs`; validator
  `yzx_repo_validator validate-contracts`
- Source: `docs/contracts/rust_nushell_bridge_contract.md`;
  `docs/contracts/v15_trimmed_runtime_contract.md`
- Deletion note: future bridge collapse must not reintroduce Nushell
  normalization

#### CRCP-002
- Type: boundary
- Status: live
- Owner: Rust runtime-env library used by public launch/setup paths
- Statement: Runtime env evaluation is Rust-owned. Any remaining shell caller
  must pass explicit inputs or process environment through one request
  boundary; derived runtime-env policy must not be re-derived by Nu callers.
- Verification: automated
  `rust_core/yazelix_core/tests/yzx_core_runtime_env.rs`; automated
  `rust_core/yazelix_core/tests/yzx_control_runtime_surface.rs`; validator
  `yzx_repo_validator validate-contracts`
- Source: `docs/contracts/rust_nushell_bridge_contract.md`;
  `docs/contracts/cross_language_runtime_ownership.md`
- Deletion note: future owner cuts may move request construction or the caller
  boundary, but they must keep one explicit request boundary and must not
  restore ambient-host inference

#### CRCP-003
- Type: failure_mode
- Status: live
- Owner: Nushell and POSIX helper-resolution bridge
- Statement: Packaged runtimes must prefer
  `$YAZELIX_RUNTIME_DIR/libexec/yzx_core` for the remaining helper-only
  runtime glue. Source checkouts may use `YAZELIX_YZX_CORE_BIN` or the freshest
  local helper build, but missing or broken helpers must fail loudly and must
  not silently revive deleted Nushell parser logic.
- Verification: automated
  `cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_maintainer --bin yzx_repo_validator -- validate-installed-runtime-contract`
- Source: `docs/contracts/rust_nushell_bridge_contract.md`;
  `docs/contracts/runtime_root_contract.md`
- Deletion note: direct Rust or launcher ownership is allowed only if the same
  explicit resolution order and fail-fast no-fallback contract survives

#### CRCP-004
- Type: invariant
- Status: live
- Owner: `config_metadata/main_config_contract.toml` plus the config-surface
  validators
- Statement: The shipped `config_default.toml` template and
  `home_manager/module.nix` default-option contract must stay synchronized
  through maintained config metadata and validation, not through ad hoc
  fallbacks or divergent duplicate defaults.
- Verification: validator
  `yzx_repo_validator validate-config-surface-contract`; validator
  `yzx_repo_validator validate-upgrade-contract`
- Source: `docs/contracts/v15_trimmed_runtime_contract.md`
- Deletion note: delete duplicate config-default owners only after this parity
  contract remains executable

#### CRCP-005
- Type: ownership
- Status: deprecated
- Owner: Rust startup/control-plane callers
- Statement: The former `yzx_core_bridge.nu` helper transport is deleted.
  Startup and maintained command surfaces call Rust owners directly instead of
  routing config/runtime semantics through Nushell JSON-envelope glue.
- Verification: automated `rust_core/yazelix_core/tests/yzx_control_runtime_surface.rs`
  startup handoff tests; manual review that no runtime entrypoint invokes the
  deleted bridge file
- Source: `docs/contracts/cross_language_runtime_ownership.md`
- Deletion note: do not recreate a Nushell helper bridge as a compatibility
  fallback

## Pilot Findings

### Duplicate-owner and deletion findings

- `yzx_core_bridge.nu` is deleted. Helper discovery, JSON-envelope parsing, and
  startup failure rendering no longer have a Nushell owner

### Weak traceability finding

- `nushell/scripts/dev/test_yzx_generated_configs.nu`,
  `nushell/scripts/dev/test_yzx_core_commands.nu`, and
  `nushell/scripts/dev/test_yzx_workspace_commands.nu` still open with the broad
  header `# Defends: docs/contracts/test_suite_governance.md`
- For this pilot, the helper-resolution and config-normalize cases in
  `test_yzx_generated_configs.nu` already map naturally to `CRCP-001` and
  `CRCP-003`. That means the validator ratchet should require contract IDs for
  touched default-lane tests before it tries to backfill the untouched backlog

### Schema outcome

- the schema works as-is for a mixed subsystem
- `quarantine` status is sufficient to describe temporary bridge ownership
  without pretending the bridge is canonical product behavior
- no schema edits are required before broad validator rollout

## Non-goals

- reviving the deleted bridge during the pilot
- backfilling every historical contract with IDs
- remapping every governed test in one pass
- choosing the final bridge-collapse implementation strategy

## Acceptance Cases

1. One mixed subsystem has a small set of indexed items with owner, status,
   verification, and deletion implications
2. At least one temporary bridge owner is marked as quarantine instead of being
   left implicit
3. At least one existing weak traceability pattern is named concretely without
   deleting the test yet
4. The pilot can feed public protocol docs, validators, and later audits
   without first rewriting the whole repo

## Verification

- `yzx_repo_validator validate-contracts`
- manual review against `docs/contracts/canonical_contract_item_schema.md`
- manual review against `docs/contracts/rust_nushell_bridge_contract.md`
- manual review against `docs/contracts/cross_language_runtime_ownership.md`
- manual review of the cited test and bridge files

## Traceability
- Depends on: `docs/contracts/canonical_contract_item_schema.md`
- Defended by: `yzx_repo_validator validate-contracts`
