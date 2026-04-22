# Rust Nextest Harness Boundary

## Summary

This document defines the minimum shared Rust-owned harness support needed to
retire strong governed Nushell tests cleanly.

The target is not a broad test framework. The target is a small nextest-friendly
support layer that makes strong contract ports cheaper than preserving Nu helper
glue. Anything outside that narrow boundary should be deleted instead of
recreated in Rust by habit.

## Scope

In scope:

- shared fixture setup for repo-root, runtime-root, config-dir, and temp-home
  test cases
- typed command launching for `yzx`, `yzx_control`, and `yzx_core`
- machine-readable JSON envelope parsing and assertion helpers
- small file-copy helpers for canonical repo assets used by strong tests

Out of scope:

- a generic subprocess DSL
- screenshot or visual sweep helpers
- replaying shell-only behavior through fake Rust wrappers
- porting weak or trivia-heavy Nu tests one-to-one

## Minimal Rust Harness Surface

The shared Rust support should live under `rust_core/yazelix_core/tests/support/`
and stay limited to four categories:

1. repo/runtime fixture setup
   - resolve repo root
   - copy canonical runtime assets such as `yazelix_default.toml`,
     `config_metadata/main_config_contract.toml`, and minimal generated config
     fixtures into temporary test roots
2. managed-config fixture setup
   - create temp HOME/config/state roots
   - write temporary managed `yazelix.toml` surfaces
   - expose the canonical env vars used by `yzx`, `yzx_control`, and `yzx_core`
3. typed command wrappers
   - `yzx_core_command(...)`
   - `yzx_control_command(...)`
   - `yzx_root_command(...)`
   - these should return configured `assert_cmd::Command` values rather than a
     stringly wrapper layer
4. envelope helpers
   - parse success/error JSON envelopes
   - small assertion helpers for common status/command/data checks

Everything else should stay local to the specific Rust test file unless it is
reused by at least two strong migration lanes.

## Nu Helper Classification

| Nu helper | Current role | Decision | Why |
| --- | --- | --- | --- |
| `nushell/scripts/dev/yzx_test_helpers.nu` | repo-root lookup, temp HOME/config fixtures, helper-bin resolution, ad hoc logging/profile formatting | `split_port_and_delete` | fixture setup and command resolution should move to Rust support; log formatting and broad convenience helpers should not survive by default |
| `nushell/scripts/dev/config_normalize_test_helpers.nu` | active-config and normalize helper wrapper | `delete_after_port` | strong replacements should call `yzx_core` directly through typed Rust wrappers instead of preserving a second helper layer |
| `nushell/scripts/dev/materialization_dev_helpers.nu` | yazi/zellij/runtime materialization wrapper calls | `delete_after_port` | the Rust migration lanes should call the helper binaries directly and keep file-local fixture shaping where needed |
| deleted `contract_traceability_helpers.nu` surface | spec and contract-item parsing for validators | `ported_to_rust_validator` | deterministic source parsing now belongs to the Rust validator lane, not the shared test harness |

## Concrete Rust Support Targets

The first implementation bead should create these concrete support files:

- `rust_core/yazelix_core/tests/support/mod.rs`
- `rust_core/yazelix_core/tests/support/fixtures.rs`
- `rust_core/yazelix_core/tests/support/commands.rs`
- `rust_core/yazelix_core/tests/support/envelopes.rs`

The first intended consumers are:

- `yzx_core_config_normalize.rs`
- `yzx_core_owned_facts.rs`
- `yzx_control_public_commands.rs`
- the next migration lanes for generated-config, workspace/session/doctor, and
  managed-config contract assertions

## Nextest Compatibility Rules

The harness must remain nextest-friendly by construction:

- no shared mutable global temp roots
- no dependence on shell process state outside per-test env configuration
- no implicit working-directory assumptions
- no wrapper that shells out through `nu -c` just to preserve an old Nu test
  shape

`cargo test` remains reserved only for doctests and explicit nextest-unsupported
exceptions.

## Dependency Decision

This harness lane does not add new crates.

- dev-only crates reused:
  - `assert_cmd`
  - `tempfile`
  - `serde_json`
- in-house logic kept:
  - repo/runtime fixture shaping
  - typed `yzx` / `yzx_control` / `yzx_core` command builders
  - JSON envelope parsing helpers
- rejected alternatives:
  - `duct` or other subprocess wrapper crates, because `assert_cmd` already owns
    the child-process contract we need
  - snapshot-heavy crates such as `insta`, because this lane is about narrow
    contract assertions, not broad output snapshots
- packaging impact:
  - none, because the lane only reuses existing dev-dependencies in
    `rust_core/yazelix_core/Cargo.toml`

## Explicit Non-Goals

- rebuilding the current Nu omnibus runners in Rust
- preserving low-value log helpers or profile printers
- adding new Rust wrappers around shell-heavy integration flows before their
  owner cut exists

## Verification

- `nu nushell/scripts/dev/validate_specs.nu`
- later implementation lanes:
  - `nix develop -c cargo nextest run --profile ci --manifest-path rust_core/Cargo.toml -p yazelix_core`

## Traceability

- Bead: `yazelix-rdn7.4.5.15`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
- Informed by: `docs/specs/rust_owned_test_migration_budget.md`
- Informed by: `docs/specs/rust_test_hardening_tools_decision.md`
