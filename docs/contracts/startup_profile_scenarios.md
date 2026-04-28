# Startup Profile Scenarios

## Summary

`yzx dev profile` must produce comparable structured startup reports for current-terminal entry, cold entry, desktop-entry launch, and managed new-window launch.

The report format stays one JSONL stream with one run header and step records. Scenario support must not introduce a second profiler or a second schema.

## Why

Desktop entry launch and managed new-window launch contain real user-waited work before the interactive Yazelix session appears: wrapper dispatch, runtime resolution, config-state computation, terminal selection, terminal config repair, Ghostty random cursor rerolls, detached terminal spawn/probe, shellHook setup, generated runtime materialization, and the final Zellij handoff boundary.

The Rust rewrite needs this baseline to preserve or improve startup behavior instead of making launch performance opaque.

## Scope

- `yzx dev profile`
- Current-terminal enter profiling
- Cold enter profiling
- Desktop-entry fast-path profiling
- Managed new-window launch profiling
- Startup profile JSONL report records under the Yazelix state directory
- Detached terminal spawn/probe timing before the interactive session
- Local saved-report comparison and named baselines

## Contract Items

#### PROF-001
- Type: behavior
- Status: live
- Owner: `yzx dev profile` and startup-profile report writers
- Statement: All supported startup scenarios write the same JSONL report schema
  with one run header plus step records. Scenario support must not fork into a
  second profiler or schema
- Verification: automated `nu nushell/scripts/dev/test_yzx_maintainer.nu`

#### PROF-002
- Type: ownership
- Status: live
- Owner: desktop and launch profile dispatch
- Statement: Desktop profiling invokes the real `yzx desktop launch` leaf
  command, and managed-launch profiling invokes the real `yzx launch` leaf
  command instead of a parallel profiling-only launcher
- Verification: automated `nu nushell/scripts/dev/test_yzx_maintainer.nu`

#### PROF-003
- Type: invariant
- Status: live
- Owner: detached-scenario summary boundary
- Statement: Desktop and managed-launch profiling wait for
  `inner.zellij_handoff_ready` before summarizing so detached startup work is
  not reported early
- Verification: automated `nu nushell/scripts/dev/test_yzx_maintainer.nu`

#### PROF-004
- Type: invariant
- Status: live
- Owner: detached-launch profiler instrumentation
- Statement: Detached terminal spawn/probe timing appears as the first-class
  step `terminal_launcher.detached_launch_probe`
- Verification: automated `nu nushell/scripts/dev/test_yzx_maintainer.nu`

#### PROF-005
- Type: behavior
- Status: live
- Owner: startup profile comparison tooling
- Statement: Saved startup profile reports can be compared locally without
  rerunning startup, including total wall-time delta and per-step deltas
- Verification: automated `yzx dev rust test core compare_profile_summaries_reports_total_and_step_deltas`

## Behavior

- `yzx dev profile` with no scenario flag profiles the current-terminal startup path.
- `yzx dev profile --cold` profiles cold current-terminal startup from outside Yazelix.
- `yzx dev profile --desktop` invokes the real `yzx desktop launch` leaf command.
- `yzx dev profile --launch` invokes the real `yzx launch` leaf command.
- `yzx dev profile --launch --terminal <name>` passes the terminal override to `yzx launch`.
- `yzx dev profile compare <baseline-report> <candidate-report>` compares two saved JSONL reports under the local startup profile directory or any explicit report paths.
- `yzx dev profile save-baseline <name> <report>` copies a saved report into the local startup profile baseline directory.
- `yzx dev profile compare-baseline <name> <candidate-report>` compares a named local baseline against another saved report.
- Profiling must work from either a repo checkout or the active installed Yazelix runtime. A writable repo checkout is not required for profiling an installed runtime.
- Desktop and managed-launch profiling must propagate the same startup-profile environment into the spawned terminal process.
- Desktop and managed-launch profiling must wait for `inner.zellij_handoff_ready` before rendering the summary, so detached startup reports are not summarized early.
- Terminal launch probing must be visible as a first-class `terminal_launcher.detached_launch_probe` step.
- Reports must keep the existing schema version, run header shape, step record shape, and summary renderer.
- Report comparison is a local maintainer evidence tool and must not become a noisy hosted-CI wall-clock gate.

## Non-goals

- Measuring the interactive Zellij session after handoff
- Adding a second report format or profiler module
- Wrapping every internal helper with timing events
- Changing normal non-profile launch behavior
- Treating profiling as a replacement for launch health checks or doctor diagnostics
- Adding a hosted CI timing gate for startup performance

## Acceptance Cases

1. Desktop profiling dispatches through `nu -c 'use <desktop.nu> *; yzx desktop launch'` and records a structured report.
2. Managed-launch profiling dispatches through `nu -c 'use <launch.nu> *; yzx launch ...'`, preserves terminal flags, and records a structured report.
3. Detached desktop and launch reports are not summarized until `inner.zellij_handoff_ready` appears or a bounded timeout fails loudly.
4. The detached terminal spawn/probe wait appears as `terminal_launcher.detached_launch_probe`.
5. The current-terminal and cold profile paths continue to use the same report schema and summary renderer.
6. Running `yzx dev profile` from outside the repo still works when the active installed runtime is valid.
7. Comparing two saved reports prints total wall-time delta plus per-step deltas without launching Yazelix.
8. A report can be copied to a named local baseline and compared with a later saved report.

## Verification

- `nu -c 'source nushell/scripts/dev/test_yzx_maintainer.nu; test_dev_profile_desktop_invokes_leaf_command_and_waits_for_handoff'`
- `nu -c 'source nushell/scripts/dev/test_yzx_maintainer.nu; test_dev_profile_launch_invokes_leaf_command_with_flags'`
- `nu -c 'source nushell/scripts/dev/test_yzx_maintainer.nu; test_startup_profile_records_detached_terminal_probe'`
- `nu -c 'source nushell/scripts/dev/test_yzx_maintainer.nu; test_startup_profile_harness_records_real_startup_boundaries'`
- `yzx dev rust test core compare_profile_summaries_reports_total_and_step_deltas`
- `yzx_repo_validator validate-contracts`

## Traceability
- Defended by: `nushell/scripts/dev/test_yzx_maintainer.nu::test_dev_profile_desktop_invokes_leaf_command_and_waits_for_handoff`
- Defended by: `nushell/scripts/dev/test_yzx_maintainer.nu::test_dev_profile_launch_invokes_leaf_command_with_flags`
- Defended by: `nushell/scripts/dev/test_yzx_maintainer.nu::test_startup_profile_records_detached_terminal_probe`
- Defended by: `nushell/scripts/dev/test_yzx_maintainer.nu::test_startup_profile_harness_records_real_startup_boundaries`
- Defended by: `rust_core/yazelix_core/src/profile_commands.rs::compare_profile_summaries_reports_total_and_step_deltas`
- Defended by: `yzx_repo_validator validate-contracts`

## Open Questions

- Should profile scenario flags become a single `--scenario` enum after the Rust rewrite if more startup scenarios are added?
- Should the timeout used by detached scenario profiling become configurable for unusually slow terminals or cold machines?
