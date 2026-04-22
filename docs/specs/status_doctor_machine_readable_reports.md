# Status And Doctor Machine-Readable Reports

## Summary

`yzx status` and `yzx doctor` should build structured reports first and keep human rendering as a separate Nushell layer.

This gives Yazelix one machine-readable inspection/report seam for the mixed `status` and `doctor` family without pretending the whole command family is ready to move into Rust yet.

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
- Owner: Rust `status.compute` plus Nushell human renderer
- Statement: `yzx status --json` emits one typed JSON report whose `summary`
  contains the same runtime summary that the human table renders
- Verification: automated
  `nushell/scripts/dev/test_yzx_core_commands.nu`

#### SDR-002
- Type: behavior
- Status: live
- Owner: public doctor report owner plus human renderer
- Statement: `yzx doctor --json` emits one JSON report with `results` and a
  typed `summary`, and the default human doctor output renders from that report
  instead of recomputing result groups separately
- Verification: automated
  `nushell/scripts/dev/test_yzx_doctor_commands.nu`

#### SDR-003
- Type: failure_mode
- Status: live
- Owner: doctor CLI boundary
- Statement: `yzx doctor --json` is read-only. `yzx doctor --json --fix` is
  rejected clearly instead of mixing machine-readable reporting with repairs
- Verification: automated
  `nushell/scripts/dev/test_yzx_doctor_commands.nu`

#### SDR-004
- Type: boundary
- Status: live
- Owner: status/doctor default command surfaces
- Statement: Default `yzx status` and `yzx doctor` behavior remains
  human-oriented. The JSON surface narrows report ownership without pretending
  that the whole family is already pure Rust or pure machine mode
- Verification: automated
  `nushell/scripts/dev/test_yzx_core_commands.nu`; automated
  `nushell/scripts/dev/test_yzx_doctor_commands.nu`

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
- default shell
- terminal list
- optional Helix runtime override
- persistent-session boolean
- optional session name

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

`summary` includes:

- `error_count`
- `warning_count`
- `info_count`
- `ok_count`
- `fixable_count`
- `healthy`

The default human doctor output renders from that report instead of recomputing counts and result groups separately.

### Fix Boundary

`yzx doctor --json` is read-only in the current v15 contract.

`yzx doctor --json --fix` is rejected with a clear error instead of mixing the machine-readable report surface with the side-effecting repair flow.

`yzx doctor --fix` remains the current human-oriented repair path.

## Non-goals

- moving the whole `status` / `doctor` family into Rust now
- changing the existing human doctor prose by default
- making `yzx doctor --json` perform repairs
- treating the JSON surface as a promise that every result field is frozen forever

## Acceptance Cases

1. `yzx status --json` exposes the same core runtime summary that the human table renders
2. `yzx doctor --json` exposes collected findings and summary counts without depending on the human renderer
3. Default `yzx status` and `yzx doctor` behavior remains human-oriented
4. `yzx doctor --json --fix` fails clearly instead of mixing read-only reporting with repairs
5. Future Rust CLI planning can point at this report seam instead of treating `status` and `doctor` as prose-only commands

## Verification

- `nu -c 'source nushell/scripts/dev/test_yzx_core_commands.nu; let results = [(test_yzx_status_reports_basic_runtime_summary) (test_yzx_status_json_reports_typed_summary)]; if ($results | all {|result| $result}) { print "ok" } else { error make {msg: "status command tests failed"} }'`
- `nu -c 'source nushell/scripts/dev/test_yzx_doctor_commands.nu; if ((run_doctor_canonical_tests) | all {|result| $result}) { print "ok" } else { error make {msg: "doctor canonical tests failed"} }'`
- `nu nushell/scripts/dev/validate_specs.nu`

## Traceability

- Bead: `yazelix-51yr`
- Defended by: `nu -c 'source nushell/scripts/dev/test_yzx_core_commands.nu; let results = [(test_yzx_status_reports_basic_runtime_summary) (test_yzx_status_json_reports_typed_summary)]; if ($results | all {|result| $result}) { print "ok" } else { error make {msg: "status command tests failed"} }'`
- Defended by: `nu -c 'source nushell/scripts/dev/test_yzx_doctor_commands.nu; if ((run_doctor_canonical_tests) | all {|result| $result}) { print "ok" } else { error make {msg: "doctor canonical tests failed"} }'`
