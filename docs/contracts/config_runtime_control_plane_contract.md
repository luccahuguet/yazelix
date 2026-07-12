# Config Runtime Control-Plane Contract

## Summary

This contract defines the current owners for config normalization, runtime-env
derivation, private helper transport, and config-surface parity.

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
- Owner: POSIX managed-tool helper resolution
- Statement: Packaged runtimes must prefer
  `$YAZELIX_RUNTIME_DIR/libexec/yzx_core` for the remaining helper-only
  runtime glue. Source checkouts may use `YAZELIX_YZX_CORE_BIN` or the freshest
  local helper build, but missing or broken helpers must fail loudly and must
  not silently revive deleted shell parser logic.
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

## Traceability Outcome

- The broad governed Nu suites were deleted after their durable behavior moved
  to `rust_core/yazelix_core/tests/yzx_control_runtime_surface.rs`,
  `rust_core/yazelix_core/tests/yzx_control_workspace_surface.rs`, and focused
  Rust module tests
- Contract validation remains the current traceability owner; deleted Nu files
  are not retained as verification references

## Non-goals

- reviving the deleted bridge
- backfilling every historical contract with IDs
- remapping every governed test in one pass

## Acceptance Cases

1. Rust is the single config-normalization and runtime-env policy owner
2. POSIX launchers resolve the matching private helper explicitly and fail visibly
3. Home Manager and the sparse semantic root derive their mapping from maintained contract metadata
4. Deleted Nushell bridges are not restored as compatibility fallbacks

## Verification

- `yzx_repo_validator validate-contracts`
- manual review against `docs/contracts/canonical_contract_item_schema.md`
- manual review against `docs/contracts/rust_nushell_bridge_contract.md`
- manual review against `docs/contracts/cross_language_runtime_ownership.md`
- manual review of the cited test and bridge files

## Traceability
- Depends on: `docs/contracts/canonical_contract_item_schema.md`
- Defended by: `yzx_repo_validator validate-contracts`
