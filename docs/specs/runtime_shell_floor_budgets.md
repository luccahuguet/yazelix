# Runtime Shell Floor Budgets

## Summary

This document defines the hard shell-floor budgets for the remaining
product/runtime Nushell families after the full-config owner cut and the recent
bridge collapses.

The goal is not a fake broad Rust rewrite. The goal is to keep Nu only where it
still directly owns shell, process, TTY, or host-integration behavior, while
forcing typed request construction, duplicated state reads, and helper policy
out of Nu.

## Scope

In scope:

- integration orchestration and wrapper transport
- setup/bootstrap shell entry
- session, launch, and desktop host integration
- runtime helpers and shared utility seams
- launch-time terminal and Ghostty request assembly
- detached terminal-probe shell transport

Out of scope:

- governed tests and validators
- maintainer/dev harnesses
- front-door renderer/data surfaces

## Current Measured Surface

Measured on `2026-04-24` after the Rust `yzx_control zellij` integration cut:

| Family | Current included surface | Current LOC | Hard target LOC |
| --- | --- | ---: | ---: |
| Integration orchestration | surviving `nushell/scripts/zellij_wrappers/*.nu` only | `90` | `90` |
| Setup/bootstrap shell entry | `setup/environment.nu`, `core/start_yazelix_inner.nu` | `395` | `300` |
| Runtime helpers and shared utility seams | surviving `utils/*.nu` except `front_door_runtime.nu` | `886` | `800` |

## `yazelix-lj7z.7` Integration Orchestration Budget

Retain only the honest external adapter seams:

- direct `zellij` command execution
- direct `ya` execution
- configured editor execution
- popup/program wrapper env handoff

Delete or move:

- tiny wrappers whose only job is to pipe one pane-orchestrator command
- duplicated config reads
- duplicated session/sidebar truth reconstruction
- bridge-local policy that Rust or the pane orchestrator already owns

Candidate surviving owners:

- `zellij_wrappers/launch_sidebar_yazi.nu`
- `zellij_wrappers/yzx_menu_popup.nu`
- `zellij_wrappers/yzx_popup_program.nu`

Stop condition:

Do not reintroduce a second Nu integration owner after the Rust `yzx_control
zellij` cut. The remaining wrappers survive only while they still own direct
Yazi or popup shell execution.

## `yazelix-lj7z.6` Setup And Bootstrap Budget

Retain only the honest shell/bootstrap owners:

- shell initializer generation
- startup env export and `with-env` orchestration
- current-shell and detached-launch handoff

Delete or move:

- Rust-owned request construction that still lives in Nu
- launch-time materialization request assembly
- duplicated config facts that are already available through Rust-owned helpers

Candidate surviving owners:

- smaller `setup/environment.nu`
- smaller `setup/initializers.nu`
- smaller `core/start_yazelix.nu`
- smaller `core/start_yazelix_inner.nu`
- smaller `core/launch_yazelix.nu`

Stop condition:

Do not port these files to Rust unless the cut deletes the Nu owner end to end
rather than hiding shell/bootstrap behavior behind Rust wrappers.

## `yazelix-lj7z.6` Session And Desktop Budget

Retain only the honest host/session boundaries:

- Zellij session discovery, restart, kill, and reattach orchestration
- desktop-entry install/uninstall side effects
- desktop launch env cleanup and delegation

Delete or move:

- typed install-ownership decisions already handled in Rust
- repeated launch/env fact construction
- helper tables or rendering policy that do not need to live beside host
  integration code

Candidate surviving owners:

- smaller `core/yzx_session.nu`
- smaller `yzx/desktop.nu`
- smaller `yzx/launch.nu`
- `yzx enter` routed through the existing startup owner in `core/start_yazelix.nu`

Stop condition:

Do not add a new Rust restart or desktop wrapper unless it deletes a real Nu
owner instead of rewrapping XDG or Zellij shell behavior.

## `yazelix-lj7z.5` Runtime Helper Boundary

The minimum surviving helper floor should be:

- terminal launch transport in `utils/terminal_launcher.nu`
- startup profile transport in `utils/startup_profile.nu`
- tiny shell-bound env/fact helpers such as:
  - `utils/runtime_env.nu`
  - `utils/shell_user_hooks.nu`
  - `utils/integration_facts.nu`
  - `utils/startup_facts.nu`
  - `utils/transient_pane_facts.nu`
- only the smallest shared helpers that still sit at a real Nu boundary

These are not allowlisted as broad long-term owners:

- `utils/yzx_core_bridge.nu` in its current size and shape
- large duplicated helpers in `utils/common.nu`
- report or config rendering helpers that Rust or data files can own
- `utils/version_info.nu`, which is deleted under `yazelix-lnk6.4`

Stop condition:

If a helper is only formatting requests, rendering duplicated reports, or
holding tables, it should not survive in the runtime helper floor.

## Launch-Time Request-Assembly Cut Under `yazelix-lj7z.6`

Retained behavior:

- launch still picks the right terminal path
- generated terminal and Ghostty configs still materialize correctly
- caller-local launch rendering stays in Nu

Deletion target:

- terminal-materialization and Ghostty request construction should stop living
  in `core/launch_yazelix.nu`

Candidate surviving owner:

- Rust terminal and Ghostty materialization owners plus caller-local Nu shell
  execution only

Stop condition:

Keep terminal selection and launch execution in Nu if the only alternative is a
fake Rust launch wrapper.

## Detached Launch Probe Contract Under `yazelix-lj7z.6`

Retained behavior:

- detached launch probing stays measurable, fast on success, and visible on
  early failure

Required contract:

- the shell program body must live in one checked-in POSIX helper
- dynamic values must flow through argv or environment variables
- Nu may call that helper, but it may not assemble a quoted multi-line shell
  program inline

Candidate surviving owner:

- a checked-in helper under `shells/posix/` plus caller-local Nu orchestration

## Shared Runtime-Helper Stop Conditions Under `yazelix-lj7z.5`

Landed deletions in this lane:

- `utils/version_info.nu` is deleted under `yazelix-lnk6.4`
- dead or purely layering-only common helpers such as
  `get_materialized_state_path` and the extra Helix directory wrappers should
  not survive in `common.nu`

Retained `common.nu` responsibilities stay Nu-owned only while they are still
directly coupled to live shell/runtime boundaries:

- runtime/config/state-root resolution that still feeds shell entrypoints,
  startup, terminal launch, and import flows
- external command discovery and current-`nu` fallback for `resolve_yazelix_nu_bin`
- shell-local default-shell rewriting for Zellij startup

Stop conditions:

- do not move runtime/config/state-root discovery into Rust while startup,
  launch, and import still consume those paths inside Nu shell/process owners
- do not add a Rust wrapper just to answer `which`/PATH questions or to
  re-expose the current Nushell binary path
- reopen a broader `common.nu` cut only after `yazelix-w6sz.3.2` or a later
  owner cut deletes the surrounding shell entrypoints substantially end to end

## Verification

- `yzx_repo_validator validate-specs`
- later implementation beads must keep the existing launch/session/integration
  regressions green

## Traceability

- Bead: `yazelix-w6sz.2.1`
- Bead: `yazelix-w6sz.3.1`
- Bead: `yazelix-w6sz.5.1`
- Bead: `yazelix-lnk6.1`
- Bead: `yazelix-nuj1.1`
- Bead: `yazelix-p18h.1`
- Bead: `yazelix-lnk6.4`
- Bead: `yazelix-lj7z.5`
- Bead: `yazelix-lj7z.6`
- Bead: `yazelix-lj7z.7`
- Bead: `yazelix-lj7z.10`
- Defended by: `yzx_repo_validator validate-specs`
- Informed by: `docs/specs/integration_glue_canonicalization_audit.md`
- Informed by: `docs/specs/launch_startup_session_canonicalization_audit.md`
- Informed by: `docs/specs/provable_nushell_floor_budget.md`
