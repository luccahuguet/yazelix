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

- Rust `yzx_core` already owns typed config normalization and runtime-env
  evaluation
- Nushell still carries a small bridge layer in `yzx_core_bridge.nu`
- packaged runtime and source-checkout helper resolution must preserve a sharp
  failure contract
- Home Manager parity with the shipped default config is a maintained invariant,
  not just a convention
- several default-lane tests already cover this subsystem, but their
  traceability is still too broad for delete-first architecture work

## Scope

- typed config normalization ownership
- the explicit boundary for `runtime-env.compute`
- `yzx_core` helper resolution and transport failure behavior
- config-surface parity between `settings_default.jsonc` and
  `home_manager/module.nix`
- pilot findings about weak traceability and duplicate-owner debt
- no product behavior changes

## Contract Items

#### CRCP-001
- Type: ownership
- Status: live
- Owner: Rust `yzx_core config.normalize`
- Statement: Typed normalization of the managed main Yazelix config is
  Rust-owned. Remaining Nushell callers may use helper transport, but they must
  not become second semantic owners for default merging, schema interpretation,
  or diagnostic classification.
- Verification: automated
  `rust_core/yazelix_core/tests/yzx_core_config_normalize.rs`
  (`config.normalize` success and error envelope tests); automated
  `nushell/scripts/dev/test_yzx_generated_configs.nu`
  (`installed runtimes use the packaged Rust config helper`, `packaged helper
  failures must be visible`, `packaged runtimes must include yzx_core`);
  validator `yzx_repo_validator validate-contracts`
- Source: `docs/contracts/rust_nushell_bridge_contract.md`;
  `docs/contracts/v15_trimmed_runtime_contract.md`
- Deletion note: future bridge collapse must not reintroduce Nushell
  normalization

#### CRCP-002
- Type: boundary
- Status: live
- Owner: Rust `yzx_core runtime-env.compute`
- Statement: Runtime env evaluation is Rust-owned. Any remaining shell caller
  must pass explicit inputs or process environment through the helper boundary;
  derived runtime-env policy stays in `runtime-env.compute` and must not be
  re-derived by Nu callers.
- Verification: automated
  `rust_core/yazelix_core/tests/yzx_core_runtime_env.rs`; automated
  `nushell/scripts/dev/test_yzx_core_commands.nu`
  (`yzx run must forward dash-prefixed child args`, `yzx run must not consume
  child --verbose flags`); validator
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
  `$YAZELIX_RUNTIME_DIR/libexec/yzx_core`. Source checkouts may use
  `YAZELIX_YZX_CORE_BIN` or the freshest local helper build, but missing or
  broken helpers must fail loudly and must not silently revive the deleted
  Nushell parser.
- Verification: automated
  `nushell/scripts/dev/test_yzx_generated_configs.nu`
  (`installed runtimes use the packaged Rust config helper`, `packaged runtimes
  must include yzx_core`, `source checkouts can use an explicit yzx_core
  helper`, `source checkouts without yzx_core must fail clearly`,
  `source-checkout helper fallback must prefer the freshest local yzx_core
  build`); validator
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
- Statement: The shipped `settings_default.jsonc` template and
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
- Status: quarantine
- Owner: Nushell bridge file `yzx_core_bridge.nu`
- Statement: The surviving Nu bridge layer is narrow transport glue around
  Rust-owned helper commands. It may resolve helper paths, execute helpers,
  parse JSON envelopes, and render Yazelix-owned helper errors, but it is not
  allowed to grow new config/runtime semantics while a bridge-collapse lane
  remains open.
- Verification: manual review of
  `nushell/scripts/utils/yzx_core_bridge.nu`; manual review against
  `docs/contracts/cross_language_runtime_ownership.md`; unverified long-term exit
  tracked as bridge-collapse debt
- Source: `docs/contracts/cross_language_runtime_ownership.md`
- Deletion note: collapse or delete the bridge once callers can reach the Rust
  owner directly or a smaller caller-local adapter boundary is proven

## Pilot Findings

### Duplicate-owner and deletion findings

- `yzx_core_bridge.nu` is a useful transport seam today, but it is still a real
  bridge owner because helper discovery, JSON envelope parsing, and helper
  failure rendering live there. It has no caller-owned error surface and no
  Zellij process wrappers

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

- changing product code or deleting any bridge during the pilot
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
