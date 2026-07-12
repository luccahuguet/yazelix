# Status And Doctor Machine-Readable Reports

## Summary

`yzx status` and `yzx doctor` build structured Rust reports first and keep human rendering as a separate presentation layer.

This gives Yazelix one machine-readable inspection/report seam while preserving friendly default terminal output.

## Why

The v16 Rust CLI evaluation concluded that `status` and `doctor` are one of the main mixed families that must narrow before a broader public Rust CLI could delete a real Nushell owner.

Before this change:

- `yzx status` assembled only a human table
- `yzx doctor` collected, summarized, and rendered findings inline
- there was no stable machine-readable report surface for the family

That shape made future deletion harder because the data and the prose were bundled together in the same command owners.

## Scope

- define the structured report shape for `yzx status --json`
- define the structured report shape for `yzx doctor --json`
- define the default human-rendered behavior
- define the current `--json` versus `--fix` boundary for doctor

## Contract Items

#### SDR-001
- Type: behavior
- Status: live
- Owner: Rust `status_report` plus the public `yzx status` renderer
- Statement: `yzx status --json` emits one typed JSON report whose `summary`
  contains the same runtime summary that the human table renders
- Verification: automated
  `rust_core/yazelix_core/tests/yzx_control_runtime_surface.rs`

#### SDR-002
- Type: behavior
- Status: live
- Owner: public doctor report owner plus human renderer
- Statement: `yzx doctor --json` emits one JSON report with `results` and a
  typed `summary`, and the default human doctor output renders from that report
  instead of recomputing result groups separately
- Verification: automated
  `rust_core/yazelix_core/tests/yzx_control_runtime_surface.rs`

#### SDR-003
- Type: failure_mode
- Status: live
- Owner: doctor CLI boundary
- Statement: `yzx doctor --json` is read-only. `yzx doctor --json --fix` is
  rejected clearly instead of mixing machine-readable reporting with repairs
- Verification: automated
  `rust_core/yazelix_core/tests/yzx_control_runtime_surface.rs`

#### SDR-004
- Type: boundary
- Status: live
- Owner: status/doctor default command surfaces
- Statement: Default `yzx status` and `yzx doctor` behavior remains
  human-oriented even though collection and rendering are Rust-owned. Machine
  mode remains an explicit opt-in
- Verification: automated
  `rust_core/yazelix_core/tests/yzx_control_runtime_surface.rs`; automated
  `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core doctor_commands`

## Behavior

### `yzx status`

Default behavior remains human-rendered.

`yzx status --json` emits one JSON object to stdout with:

- `title`
- `summary`
- optional `versions` when `--versions` is also passed

`summary` is the typed status payload. It includes:

- version and description
- config file and runtime/log paths
- generated-state repair flag
- generated-state materialization status, reason, input/cache freshness, and missing-artifact list
- default shell
- terminal list
- active session config snapshot status, path, source config, and readable snapshot error details when present

The human table is rendered from that report rather than being the primary source of truth.

### `yzx doctor`

Default behavior remains human-rendered.

`yzx doctor --json` emits one JSON object to stdout with:

- `title`
- `results`
- `summary`

`results` is the collected list of doctor findings. Each result keeps the existing structured check fields such as:

- `status`
- `message`
- `details`
- `fix_available`
- any check-specific metadata already attached by the collector

Doctor reports the generated runtime materialization plan as its own
`generated_state_check = "runtime_materialization_plan"` finding. That finding
covers stale config/runtime input hashes and missing generated artifacts from the
shared materialization plan. The generated workspace asset check remains a
separate finding for concrete Zellij config/layout/plugin file freshness. A
fresh workspace asset finding does not imply the materialization-plan cache is
fresh, and a stale materialization-plan finding does not imply generated files
are byte-stale.

`summary` includes:

- `error_count`
- `warning_count`
- `info_count`
- `ok_count`
- `fixable_count`
- `healthy`

The default human doctor output renders from that report instead of recomputing counts and result groups separately.

### Fix Boundary

`yzx doctor --json` is read-only in the current trimmed contract.

`yzx doctor --json --fix` is rejected with a clear error instead of mixing the machine-readable report surface with the side-effecting repair flow.

`yzx doctor --fix` remains the current human-oriented repair path.

## Non-goals

- changing the existing human doctor prose by default
- making `yzx doctor --json` perform repairs
- treating the JSON surface as a promise that every result field is frozen forever

## Acceptance Cases

1. `yzx status --json` exposes the same core runtime summary that the human table renders
2. `yzx doctor --json` exposes collected findings and summary counts without depending on the human renderer
3. Default `yzx status` and `yzx doctor` behavior remains human-oriented
4. `yzx doctor --json --fix` fails clearly instead of mixing read-only reporting with repairs
5. Human and JSON output consume the same Rust-owned report instead of maintaining parallel truth

## Verification

- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_control_runtime_surface status`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core doctor_commands`
- `yzx_repo_validator validate-contracts`

## Traceability
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_control_runtime_surface status`
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core doctor_commands`
