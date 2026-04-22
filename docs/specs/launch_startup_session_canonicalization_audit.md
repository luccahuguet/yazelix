# Launch Startup Session Canonicalization Audit

## Summary

This audit reviews the live launch, startup, and session-orchestration path for
Yazelix after the `config_state.nu` deletion and the `runtime_env.nu`
request-owner cut.

The subsystem still mixes real shell/process ownership with some remaining
request-shaping debt, but the honest next work is not a broad Rust rewrite.
The honest next work is to keep shell/process orchestration in Nu and POSIX,
collapse the remaining launch-time request owners into Rust where that deletes a
real seam, and refactor the quoted detached-launch shell trampoline into a
checked-in POSIX helper instead of keeping it embedded inside Nushell.

## 1. Subsystem Snapshot

- Subsystem: launch, startup, and session orchestration
- Purpose: turn user intent into a live Yazelix shell or window, materialize
  required runtime artifacts, hand off to Zellij, and restart an active
  session through the stable owner path
- User-visible entrypoints:
  - `yzx enter`
  - `yzx launch`
  - `yzx desktop launch`
  - `yzx restart`
  - POSIX launcher entry through `shells/posix/yzx_cli.sh`
- Primary source paths:
  - `nushell/scripts/yzx/enter.nu`
  - `nushell/scripts/yzx/launch.nu`
  - `nushell/scripts/yzx/desktop.nu`
  - `nushell/scripts/core/start_yazelix.nu`
  - `nushell/scripts/core/start_yazelix_inner.nu`
  - `nushell/scripts/core/launch_yazelix.nu`
  - `nushell/scripts/core/yzx_session.nu`
  - `nushell/scripts/setup/environment.nu`
  - `nushell/scripts/utils/terminal_launcher.nu`
  - `shells/posix/runtime_env.sh`
  - `shells/posix/start_yazelix.sh`
  - `shells/posix/yzx_cli.sh`
- External dependencies that matter:
  - POSIX shell and `bash` for detached launch
  - terminal emulators and platform-specific launch flags
  - Zellij process/session behavior
  - pane orchestrator session truth

## 2. Must-Not-Lose Behavior

| Behavior | Current contract or source | Current owner | Current verification | Candidate surviving owner |
| --- | --- | --- | --- | --- |
| Current-terminal startup bootstraps the managed runtime env, state dir, and logs dir before entering the live shell | `docs/specs/v15_trimmed_runtime_contract.md`; `docs/specs/runtime_root_contract.md` | POSIX `runtime_env.sh`; Nu `start_yazelix.nu`; Nu `setup/environment.nu` | `test_startup_bootstrap_runtime_env_exports_state_and_logs_dirs` in `nushell/scripts/dev/test_yzx_workspace_commands.nu`; `validate_installed_runtime_contract.nu` | same split, with Rust continuing to own typed env/config/materialization decisions |
| Launch and startup fail fast on missing working directories, missing runtime scripts, unavailable configured terminals, and unresolved custom layouts | `docs/specs/runtime_dependency_preflight_contract.md` | Rust `startup-launch-preflight.evaluate`; Nu `yzx/launch.nu`; Nu `start_yazelix.nu`; Nu `launch_yazelix.nu`; Nu `start_yazelix_inner.nu` | `test_startup_rejects_missing_working_dir`; `test_launch_rejects_file_working_dir`; `test_startup_materializes_missing_managed_layout_before_handoff`; `test_startup_custom_layout_override_fails_clearly` in `nushell/scripts/dev/test_yzx_workspace_commands.nu` | Rust helper keeps bounded preflight; Nu keeps entrypoint-specific rendering and shell handoff |
| Desktop and managed new-window launch clear hostile inherited activation state and use one explicit fast path without a hidden fallback launch | `docs/specs/runtime_activation_state_contract.md`; `docs/specs/runtime_root_contract.md` | POSIX `yzx_cli.sh`; Nu `yzx/desktop.nu`; Nu `yzx/launch.nu`; Nu `launch_yazelix.nu`; Nu `terminal_launcher.nu` | `test_yzx_cli_desktop_launch_ignores_hostile_shell_env`; `test_yzx_desktop_launch_uses_leaf_launch_module_with_clean_env`; `test_yzx_desktop_launch_propagates_fast_path_failures_without_fallback` in `nushell/scripts/dev/test_yzx_workspace_commands.nu` | same split |
| Persistent and non-persistent session semantics stay explicit, including restart scope and `--path` behavior | `docs/specs/persistent_window_session_contract.md`; `docs/specs/nonpersistent_window_session_contract.md`; `docs/specs/workspace_session_contract.md` | Nu `start_yazelix_inner.nu`; Nu `yzx_session.nu`; Zellij; pane orchestrator | `test_launch_here_path_uses_requested_directory_for_nonpersistent_sessions`; `test_launch_here_path_warns_when_existing_persistent_session_ignores_it` in `nushell/scripts/dev/test_yzx_workspace_commands.nu`; `test_home_manager_profile_restart_uses_owner_wrapper_without_manual_surfaces` in `nushell/scripts/dev/test_yzx_maintainer.nu` | same split, with pane/session truth remaining outside `rust_core` |
| Detached launch probing remains measurable, fast on success, and visible on early terminal death | `docs/specs/startup_profile_scenarios.md` | Nu `terminal_launcher.nu`; Nu `startup_profile.nu` | `test_startup_profile_records_detached_terminal_probe`; `test_detached_launch_probe_success_path_is_fast`; `test_detached_launch_probe_early_failure_is_visible` in `nushell/scripts/dev/test_yzx_maintainer.nu` | Nu launch orchestration plus a checked-in POSIX helper instead of an embedded shell body |
| Launch-time editor/runtime env handoff stays canonical for nested editor and popup flows | `CRCP-002`; `docs/specs/launch_bootstrap_rust_migration.md`; `docs/specs/workspace_session_contract.md` | Rust `runtime_env.rs`; Nu caller-local orchestration in `start_yazelix.nu`, `yzx/launch.nu`, `editor_launch_context.nu`, popup wrappers | `test_yzx_edit_resolves_managed_helix_wrapper_from_canonical_launch_env` in `nushell/scripts/dev/test_yzx_workspace_commands.nu`; `test_get_runtime_env_wraps_helix_with_managed_wrapper` and `test_get_runtime_env_exports_curated_toolbin_and_keeps_runtime_local_yzx` in `nushell/scripts/dev/test_helix_managed_config_contracts.nu`; popup tests in `nushell/scripts/dev/test_yzx_popup_commands.nu` | Rust `control_plane.rs` and `runtime_env.rs` plus caller-local Nu shell/env use |

## 3. Canonical Owner Map

| Concern | Current owner or split boundary | Split kind | Audit judgment |
| --- | --- | --- | --- |
| User-visible launch/start/restart behavior | Nu `yzx/enter.nu`, `yzx/launch.nu`, `yzx/desktop.nu`, `core/yzx_session.nu` | intentional | Public command UX is still a Nu surface |
| Typed or deterministic logic | Rust `runtime-env.compute`, `config-state.compute`, `startup-launch-preflight.evaluate`, `runtime-materialization.*`, install-ownership evaluation | intentional | Rust already owns the reusable decision layers that are truly shared |
| Shell or process orchestration | POSIX launchers; Nu `start_yazelix.nu`, `start_yazelix_inner.nu`, `launch_yazelix.nu`, `terminal_launcher.nu`, `yzx_session.nu`, `setup/environment.nu` | intentional with local bridge debt | This is the real shell/process boundary, but some launch-time request shaping still lives beside it |
| Generated-state writes | Rust runtime, terminal, Ghostty, Yazi, Zellij, and Helix materialization; Nu shell initializer generation in `setup/environment.nu` | mixed | Rust is the canonical owner for generated runtime artifacts; Nu still owns shell initializer generation and shellhook bridge sync |
| Live session or plugin state | Zellij and pane orchestrator | intentional | This should stay outside a broad `rust_core` launch rewrite |
| Final human-facing rendering | Nu launch/start/session modules | intentional | Caller-local prose and recovery text belong here |
| Launch-time terminal materialization request construction | Nu `launch_yazelix.nu` plus Rust `terminal_materialization.rs` / `ghostty_materialization.rs` | temporary bridge debt | This is the cleanest remaining Rust owner-cut inside launch code |
| Detached launch probe trampoline | Nu `terminal_launcher.nu` inline shell body | temporary bridge debt | The behavior is shell-bound, but the current embedding shape is weaker than a checked-in POSIX helper |

## 4. Survivor Reasons

- POSIX `runtime_env.sh`: `external_tool_adapter`
- POSIX `start_yazelix.sh`: `external_tool_adapter`
- POSIX `yzx_cli.sh`: `external_tool_adapter`
- Nu `start_yazelix.nu`: `irreducible_shell_boundary`
- Nu `start_yazelix_inner.nu`: `irreducible_shell_boundary`
- Nu `launch_yazelix.nu`: `irreducible_shell_boundary`
  - launch-time terminal and Ghostty request construction inside it: `temporary_bridge_debt`
- Nu `terminal_launcher.nu`: `external_tool_adapter`
  - embedded detached-launch probe shell body inside it: `temporary_bridge_debt`
- Nu `setup/environment.nu`: `irreducible_shell_boundary`
- Nu `yzx_session.nu`: `irreducible_shell_boundary`
- Rust launch/runtime/materialization helpers: `canonical_owner`
- Zellij and pane orchestrator: `canonical_owner`

## 5. Delete-First Findings

### Delete Now

- No broad product-code deletion is honest from this audit alone
- The work now is boundary sharpening, not another speculative full rewrite

### Bridge Layer To Collapse

- `launch_yazelix.nu` still owns terminal-materialization and Ghostty
  request assembly even though Rust already owns the typed materialization
  logic
- `terminal_launcher.nu` still embeds a quoted multi-line shell trampoline for
  detached launch probing instead of calling a checked-in POSIX helper

### Full-Owner Migration

- No broad launch/startup/session full-owner Rust migration is honest today
- The shell/process/Zellij/session boundary is still the dominant owner, so a
  broad Rust rewrite would mostly add wrappers under real orchestration code

### Likely Survivors

- POSIX bootstrap wrappers
- `start_yazelix_inner.nu`
- `setup/environment.nu`
- `yzx_session.nu`
- caller-local Nu rendering in `yzx/launch.nu`, `yzx/desktop.nu`, and related
  start/session modules

### No-Go Deletions

- Broad Rust rewrite of launch/startup/session orchestration
  - Stop condition: reopen only if a future cut deletes a whole Nu owner instead
    of wrapping terminal string assembly, shell env setup, or Zellij handoff
- Deleting `setup/environment.nu`
  - Stop condition: only honest if shell initializer generation, bridge sync,
    executable-bit repair, and welcome gating get one clearer owner instead of
    scattering across more wrappers
- Deleting `yzx_session.nu`
  - Stop condition: only honest if restart becomes a narrower, explicit session
    transition owner rather than moving the same relaunch/kill orchestration
    behind Rust wrappers

## 6. Quality Findings

- Duplicate owners:
  - launch-time config/state/root resolution in `launch_yazelix.nu` still
    mirrors data Rust can already derive for materialization helpers
  - `common.nu` still mirrors some config/runtime/state root logic that also
    exists in Rust, but the remaining launch/startup surfaces still need local
    shell-facing path resolution
- Missing layer problems:
  - there is no checked-in POSIX helper for the detached terminal probe even
    though the repo now forbids adding new inline quoted shell-script bodies as
    the default pattern
- Extra layer problems:
  - `launch_yazelix.nu` still mixes execution, terminal fallthrough policy,
    request assembly, and launch-time progress rendering in one file
- DRY opportunities:
  - the terminal-materialization and Ghostty helper calls should stop carrying
    their own root/path request assembly in Nu
  - detached launch probing already has strong tests and profiling contracts,
    so the fixed shell trampoline can move out of Nushell without changing the
    user-visible launch logic
- Weak or orphan tests:
  - none obvious in this slice, but many launch/session assertions live inside
    large omnibus files rather than small subsystem-focused files
- Only-known-executable-defense tests:
  - `test_detached_launch_probe_success_path_is_fast`
  - `test_detached_launch_probe_early_failure_is_visible`
  - `test_home_manager_profile_restart_uses_owner_wrapper_without_manual_surfaces`
- Spec gaps:
  - there is no dedicated live spec for the detached-launch probe seam itself
  - restart bootstrap-file semantics are covered indirectly by session contracts
    and tests, not by one focused contract
- Docs drift:
  - historical launch/bootstrap notes remain useful, but the live current owner
    map now belongs in this audit rather than in transition prose alone

## 7. Deletion Classes And Follow-Up Beads

- `yazelix-nuj1`
  - retained behavior: launch still generates bundled terminal configs and
    Ghostty rerolls correctly before detached startup
  - deletion class: `bridge_collapse`
  - candidate surviving owner: Rust `terminal_materialization.rs`,
    `ghostty_materialization.rs`, and `control_plane.rs`
  - verification: `test_yzx_generated_configs.nu`,
    `test_yzx_workspace_commands.nu`, `validate_flake_install.nu`
  - stop condition: keep caller-local terminal selection and execution in Nu if
    the only alternative is a fake Rust launch wrapper
- `yazelix-p18h`
  - retained behavior: detached terminal launch probing stays measurable, fast
    on success, and visible on early failure
  - deletion class: `bridge_collapse`
  - candidate surviving owner: a checked-in POSIX helper under `shells/posix/`
    plus Nu launch orchestration
  - verification: `test_startup_profile_records_detached_terminal_probe`,
    `test_detached_launch_probe_success_path_is_fast`,
    `test_detached_launch_probe_early_failure_is_visible`
  - stop condition: if terminal-specific command shapes still require string
    assembly in Nu, move only the fixed shell trampoline out of Nushell

## Verification

- `nu nushell/scripts/dev/validate_specs.nu`
- manual review of:
  - `nushell/scripts/core/start_yazelix.nu`
  - `nushell/scripts/core/start_yazelix_inner.nu`
  - `nushell/scripts/core/launch_yazelix.nu`
  - `nushell/scripts/core/yzx_session.nu`
  - `nushell/scripts/setup/environment.nu`
  - `nushell/scripts/utils/terminal_launcher.nu`
  - `shells/posix/runtime_env.sh`
  - `shells/posix/start_yazelix.sh`
  - `shells/posix/yzx_cli.sh`

## Traceability

- Bead: `yazelix-rdn7.5.3`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
- Informed by: `docs/specs/runtime_dependency_preflight_contract.md`
- Informed by: `docs/specs/persistent_window_session_contract.md`
- Informed by: `docs/specs/nonpersistent_window_session_contract.md`
- Informed by: `docs/specs/startup_profile_scenarios.md`
