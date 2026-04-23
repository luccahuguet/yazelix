# Runtime Dependency And Launch Preflight Contract

## Summary

Yazelix should define one shared runtime dependency contract for normal user-facing entrypoints and a narrower launch-preflight scope that checks only the fast, actionable prerequisites needed before launch. The same dependency story should be reusable by `yzx launch`, startup, `yzx doctor`, install smoke, and a future `Yazelix Core` discussion without forcing all of them to perform the same depth of checks.

## Why

The current code already distinguishes several kinds of checks, but the boundary is implicit:

- startup and launch fail fast on missing working directories, missing runtime scripts, missing generated layouts, and unavailable configured terminals
- config validation runs before launch, but it is a config-contract concern rather than a runtime dependency concern
- `yzx doctor` performs much heavier checks such as desktop-entry freshness, install-artifact staleness, version drift, Helix runtime conflicts, and plugin health
- install smoke validates even heavier installed-runtime invariants that are useful for packaging confidence but too expensive for normal launch

Without a written contract:

- launch can grow into a slow mini-doctor
- doctor can keep inventing checks that users assume launch will enforce
- later Core work will keep guessing which missing tools are launch blockers versus optional diagnostics
- error messages drift between launch, doctor, and install smoke

## Scope

- define the runtime dependency classes used by normal user-facing entrypoints
- define which checks belong in lightweight launch preflight
- define which checks belong in richer `yzx doctor` diagnostics
- define which checks belong only in install/package validation
- define how config-conditioned requirements should be expressed

## Contract Items

#### PRE-001
- Type: boundary
- Status: live
- Owner: shared runtime-preflight evaluation plus entrypoint-specific rendering
- Statement: Launch preflight is a fast, bounded dependency check for the
  selected entrypoint. It must not silently expand into full doctor or
  install-smoke behavior
- Verification: automated
  `nushell/scripts/dev/test_yzx_workspace_commands.nu`; automated
  `nushell/scripts/dev/test_yzx_doctor_commands.nu`

#### PRE-002
- Type: failure_mode
- Status: live
- Owner: launch/startup preflight owners
- Statement: Missing or invalid working directories, missing runtime entrypoint
  scripts, and unavailable configured terminals for new-window launch are
  immediate preflight blockers with direct recovery guidance
- Verification: automated
  `nushell/scripts/dev/test_yzx_workspace_commands.nu`; automated
  `nushell/scripts/dev/test_yzx_generated_configs.nu`

#### PRE-003
- Type: behavior
- Status: live
- Owner: startup materialization plus bounded preflight
- Statement: When startup depends on managed generated layouts, Yazelix
  materializes the missing managed layout before handoff. Unresolved custom
  layout overrides still fail clearly instead of being silently replaced
- Verification: automated
  `nushell/scripts/dev/test_yzx_workspace_commands.nu`

#### PRE-004
- Type: ownership
- Status: live
- Owner: `yzx doctor` and install/package validation
- Statement: Desktop-entry freshness, install-artifact staleness, version drift,
  Helix runtime conflicts, and broader install integrity remain doctor or
  install-smoke work rather than universal launch blockers
- Verification: automated
  `rust_core/yazelix_core/tests/yzx_control_runtime_surface.rs`; automated
  `cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_core --bin yzx_repo_validator -- validate-installed-runtime-contract`

#### PRE-005
- Type: boundary
- Status: live
- Owner: config-conditioned preflight selection
- Statement: Terminal availability is a preflight requirement only for entry
  paths that launch a new terminal window. Current-terminal entrypoints do not
  inherit detached-terminal requirements they do not use
- Verification: automated
  `nushell/scripts/dev/test_yzx_workspace_commands.nu`; automated
  `nushell/scripts/dev/test_yzx_generated_configs.nu`

## Behavior

- The runtime dependency contract is about what must be present or resolvable
  for a supported user-facing Yazelix entrypoint to work honestly.
- This contract is downstream of the live runtime and bridge contracts:
  - the runtime contracts define which entrypoints, managed artifacts, and
    owned roots exist
  - this contract defines what should be verified quickly before launch and what
    should remain in richer diagnostics
- Stale-config blocking is adjacent but separate:
  - entrypoint config validation should still run before launch
  - but config parsing and unsupported-field diagnostics are not themselves part of the runtime dependency checker scope

### Dependency Classes

- Always-required entrypoint prerequisites:
  - a valid Yazelix runtime root
  - required runtime scripts for the chosen entrypoint
  - required generated layout/config paths for startup where those artifacts are part of the supported runtime model
  - a valid working directory when the entrypoint accepts one
- Config-conditioned requirements:
  - the configured terminal must be available for `yzx launch` when launching a new terminal window
  - terminal availability depends on whether Yazelix is managing terminals or relying on host-installed terminals
  - entrypoints that stay in the current terminal, such as `yzx enter`, do not require a detached terminal candidate
- Runtime-owned assets versus host tools:
  - runtime-owned scripts, generated layouts, and shipped assets should be treated as runtime contract dependencies
  - host or externally resolved tools should only be treated as required when the current entrypoint truly needs them to proceed
- Optional or doctor-only diagnostics:
  - version drift warnings
  - desktop-entry freshness
  - install-artifact staleness
  - Helix runtime conflicts and deeper health checks
  - session-local plugin health
  - these may matter a lot, but they should not all be launch blockers by default

### First-Pass Ownership Matrix

This matrix is intentionally concrete. It exists to stop runtime checks from drifting between launch, doctor, config migration preflight, and install smoke.

| Check or condition | Owner | Why |
| --- | --- | --- |
| Missing or nondirectory working directory for launch/startup | Launch preflight | The chosen entrypoint will fail immediately and the recovery is local and fast. |
| Missing runtime entrypoint script such as `start_yazelix_inner.nu` or `launch_yazelix.nu` | Launch preflight | This is a hard runtime integrity blocker for the selected path, not a deep diagnostic. |
| Missing managed generated layout required for startup | Startup materialization plus bounded preflight | Startup should repair Yazelix-owned generated layouts before failing, while custom layout overrides still fail clearly if they remain unresolved. |
| No suitable configured/requested terminal available for new-window launch | Launch preflight | Detached launch should fail clearly before attempting terminal startup. |
| Unsupported config follow-up before entrypoint execution | Adjacent config-surface validation | This can block entrypoints, but it belongs to config-surface ownership rather than runtime dependency checking. |
| Stale desktop entry or launcher-shadowing install artifact | `yzx doctor` | Important health signal, but not a universal startup blocker for every entrypoint. |
| Installed runtime pointer correctness or stable launcher shim correctness | Install/package validation and `yzx doctor` | These defend packaging/install integrity and may be inspected by doctor, but are too heavy for routine launch. |
| Minimal-PATH POSIX launcher viability and shell-enter contract | Install/package validation | Heavy install-smoke concerns, not routine preflight checks. |
| Version drift, Helix runtime conflicts, plugin/session-local health | `yzx doctor` | Rich diagnostics that should not silently expand launch into a slow environment audit. |

### Shared Evaluation Boundary

- The shared runtime-preflight reasoning should live behind one structured evaluation boundary instead of being recomputed independently in every Nushell surface.
- In the current v15.x bridge, that boundary is the packaged/helper command `yzx_core runtime-contract.evaluate`.
- Startup, new-window launch, and the shared doctor-preflight surface should submit one batch request per surface and receive a machine-readable list of findings.
- Nushell still owns:
  - surface-specific rendering and prose
  - whether a finding is fatal for that entrypoint
  - `yzx doctor --fix` actions and install-audit follow-ups
- The first Rust-backed parity set is:
  - working-directory validation
  - required runtime-script existence
  - generated-layout existence
  - new-window terminal availability and candidate selection
  - the Linux Ghostty desktop-launch graphics warning
- Doctor-only install-artifact audits, Helix runtime diagnostics, distribution-tier reporting, and session-local/plugin-local health remain outside this helper until later beads.

### Launch Preflight Scope

- Launch preflight should be fast and bounded.
- It should check only the dependencies whose absence makes the selected launch path fail immediately or misleadingly.
- For normal startup/launch flows, that includes at least:
  - requested working directory exists and is a directory
  - the active runtime root resolves
  - entrypoint runtime scripts required for startup exist
  - managed generated layouts can be materialized, and the selected layout path exists before asking Zellij to use it
  - when launching a new terminal, at least one suitable configured/requested terminal candidate is available for the current terminal-management mode
- Launch preflight should fail fast with explicit recovery guidance.
- Launch preflight should not:
  - perform deep freshness audits of desktop entries or other install artifacts
  - perform full install integrity checks
  - perform slow environment-wide health analysis just because doctor can

### Doctor Scope

- `yzx doctor` should consume the same dependency story, but it may check more than launch preflight.
- Doctor is the place for:
  - stale or broken install artifacts
  - desktop-entry freshness
  - version drift reporting
  - Helix runtime conflicts and deeper runtime health
  - plugin/session-local health
  - fixable repair surfaces such as `yzx doctor --fix`
- Doctor may report warnings that launch tolerates, as long as that distinction is explicit.

### Install-Smoke Scope

- install/package smoke checks may validate heavier invariants than normal launch or doctor.
- Examples:
  - installed runtime pointer correctness
  - installed `yzx` shim correctness
  - POSIX launcher behavior under minimal environment
  - runtime-local tool resolution
  - shell-enter command viability
- These checks defend packaging and install contracts, not the everyday preflight path.

## Non-goals

- redefining the remaining delete-first control-plane ownership already covered
  by the runtime and Rust migration docs
- redefining config validation or stale-config rules
- making `yzx doctor` and launch run the exact same set of checks
- turning launch into a slow environment audit
- deciding the final `Yazelix Core` product boundary in this spec

## Acceptance Cases

1. When `yzx launch --path` receives a missing or nondirectory path, launch preflight fails before a deeper launch attempt with a direct recovery message.
2. When startup or new-window launch depends on a missing runtime script, launch fails clearly as a runtime/generated-state problem instead of surfacing a generic downstream tool failure.
3. When a new-terminal launch is requested and the configured terminal is unavailable for the current management mode, launch fails quickly with terminal-specific guidance instead of falling through into unrelated errors.
4. When startup depends on a missing managed generated layout, startup materializes it before asking Zellij to use that path, and unresolved custom layout overrides still fail clearly.
5. When desktop entries or installed runtime links are stale, `yzx doctor` may report them, but normal launch preflight does not have to run the full install-audit surface first.
6. When a later Core discussion asks which dependencies are true launch blockers versus richer diagnostics, the answer can be taken from this contract instead of inferred ad hoc from current implementation details.

## Verification

- manual review against:
  - [runtime_root_contract.md](./runtime_root_contract.md)
  - [rust_migration_matrix.md](./rust_migration_matrix.md)
  - [v15_trimmed_runtime_contract.md](./v15_trimmed_runtime_contract.md)
  - [stale_config_diagnostics.md](./stale_config_diagnostics.md)
- manual review of the current runtime-check code paths:
  - `nushell/scripts/core/start_yazelix.nu`
  - `nushell/scripts/core/start_yazelix_inner.nu`
  - `nushell/scripts/core/launch_yazelix.nu`
  - `nushell/scripts/utils/terminal_launcher.nu`
  - `nushell/scripts/utils/doctor_fix.nu`
  - `yzx_repo_validator validate-installed-runtime-contract`
  - `nushell/scripts/dev/validate_flake_install.nu`
- integration tests:
  - `nu nushell/scripts/dev/test_yzx_workspace_commands.nu`
  - `nu nushell/scripts/dev/test_yzx_doctor_commands.nu`
  - `nu nushell/scripts/dev/test_yzx_core_commands.nu`
- helper tests:
  - `cargo test --manifest-path rust_core/Cargo.toml`
- CI/spec check: `yzx_repo_validator validate-specs`

## Traceability

- Bead: `yazelix-kt5.3`
- Defended by:
  - `yzx_repo_validator validate-specs`
  - `cargo test --manifest-path rust_core/Cargo.toml`

## Open Questions

- Should generated Yazi config preparation remain implicitly part of normal startup readiness, or should some of that surface move into a narrower runtime preflight helper that can classify failures more explicitly?
- Should launch preflight eventually expose structured failure classes directly so `yzx doctor`, install smoke, and CLI entrypoints stop duplicating recovery text?
- For a future `Yazelix Core` edition, which host-managed tools should remain doctor-only warnings versus true launch blockers?
