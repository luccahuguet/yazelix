# Config Runtime Control-Plane Canonicalization Audit

## Summary

This audit covers the mixed config/runtime/control-plane seam that still spans
`config_surfaces.nu`, `runtime_env.nu`, `yzx_core_bridge.nu`, `common.nu`, the
dev-only `config_normalize_test_helpers.nu` probe, and the Rust `yzx_core` /
`yzx_control` owners that already replaced large parts of the old Nushell
logic.

The goal is not to delete code during the audit. The goal is to name the
retained behavior, identify which surviving Nushell layers are still real
owners versus temporary bridge debt, and create honest deletion-lane beads for
the remaining serious Nu cuts.

## 1. Subsystem Snapshot

- subsystem name: config, runtime-env, and control-plane bridge ownership
- purpose: resolve the active managed config surface, normalize config against
  the shipped contract, compute the canonical runtime env, decide whether
  generated state is stale, and transport those results into `yzx` commands and
  startup/launch orchestration
- user-visible entrypoints: `yzx config ...`, `yzx env`, `yzx run`, startup,
  launch, popup/editor helper launches, generated-state repair, and doctor
  paths that read config/runtime state
- primary source paths:
  - `nushell/scripts/utils/config_surfaces.nu`
  - `nushell/scripts/utils/runtime_env.nu`
  - `nushell/scripts/utils/yzx_core_bridge.nu`
  - `nushell/scripts/utils/common.nu`
  - `nushell/scripts/utils/environment_bootstrap.nu`
  - `nushell/scripts/dev/config_normalize_test_helpers.nu`
  - `rust_core/yazelix_core/src/active_config_surface.rs`
  - `rust_core/yazelix_core/src/runtime_env.rs`
  - `rust_core/yazelix_core/src/config_state.rs`
  - `rust_core/yazelix_core/src/control_plane.rs`
  - `rust_core/yazelix_core/src/config_commands.rs`
- upstream or external dependencies that matter:
  - `yazelix_default.toml`
  - `config_metadata/main_config_contract.toml`
  - `home_manager/module.nix`
  - `shells/posix/yazelix_nu.sh`
  - the packaged `libexec/yzx_core` helper and source-checkout helper fallback

## 2. Must-Not-Lose Behavior

| Behavior | Current contract or source | Current owner | Current verification | Candidate surviving owner |
| --- | --- | --- | --- | --- |
| Canonical managed config surface lives under `user_configs/`, rejects legacy-root conflicts, bootstraps missing `yazelix.toml`, and keeps managed Taplo support current | `docs/specs/config_surface_and_launch_profile_contract.md`; `CRCP-004` | Rust `active_config_surface.rs` is the canonical owner; Nu callers consume the resolved surface where shell UX still needs it | `nu nushell/scripts/dev/validate_config_surface_contract.nu`; `rust_core/yazelix_core/src/active_config_surface.rs`; `rust_core/yazelix_core/tests/yzx_core_config_normalize.rs` | Rust `active_config_surface.rs` with only caller-local Nu shell orchestration where file writes or editor launch UX still require it |
| Typed main-config normalization and diagnostic classification stay Rust-owned | `CRCP-001`; `docs/specs/rust_nushell_bridge_contract.md` | Rust `normalize_config`; the former `config_parser.nu` bridge owner is deleted | `rust_core/yazelix_core/tests/yzx_core_config_normalize.rs`; the remaining helper-resolution cases in `nu nushell/scripts/dev/test_yzx_generated_configs.nu` | same |
| Canonical runtime env policy stays in Rust and does not drift back into Nu | `CRCP-002`; `docs/specs/launch_bootstrap_rust_migration.md` | Rust `runtime_env.rs` plus the explicit Nu shell-exec seam in `runtime_env.nu` | `rust_core/yazelix_core/tests/yzx_core_runtime_env.rs`; `nu nushell/scripts/dev/test_helix_managed_config_contracts.nu`; `nu nushell/scripts/dev/test_yzx_popup_commands.nu` | Rust `runtime_env.rs` / `control_plane.rs` with Nu limited to shell env application and argv execution |
| Packaged helper resolution and source-checkout helper fallback fail fast without reviving deleted Nu semantics | `CRCP-003`; `docs/specs/runtime_root_contract.md` | Nu `yzx_core_bridge.nu` plus POSIX/runtime launcher wiring | Rust config/materialization tests; `yzx_repo_validator validate-installed-runtime-contract` | smaller helper bridge or later public Rust command owner, but not a wider Nu bridge |
| Generated-state refresh uses config/runtime hashes for the default managed surface and records state only for the canonical managed config | `docs/specs/config_surface_and_launch_profile_contract.md`; `docs/specs/runtime_activation_state_contract.md` | Rust `config_state.rs` and `control_plane.rs`, with Nu startup/launch consuming structured results | `rust_core/yazelix_core/src/config_state.rs`; `nu nushell/scripts/dev/test_yzx_generated_configs.nu`; `nu nushell/scripts/dev/validate_config_surface_contract.nu` | same |
| Home Manager defaults and shipped config defaults stay synchronized through maintained metadata, not duplicate owners or fallback behavior | `CRCP-004`; `docs/specs/config_metadata_centralization_plan.md` | config metadata plus validators, with both Nu and Rust reading the same runtime contract | `nu nushell/scripts/dev/validate_config_surface_contract.nu`; `nu nushell/scripts/dev/validate_upgrade_contract.nu` | same |

## 3. Canonical Owner Map

| Concern | Current owner or split boundary | Split kind | Audit judgment |
| --- | --- | --- | --- |
| User-visible config/runtime behavior | Nu `yzx` commands and startup/launch frontends calling Rust helpers | intentional | The public surface is still Nu-first in v15.4, but typed decisions should keep shrinking out of Nu |
| Typed or deterministic logic | Rust `active_config_surface.rs`, `runtime_env.rs`, `config_state.rs`, `config_commands.rs` | intentional | This is already the canonical owner set |
| Active config-surface resolution and Taplo synchronization | Rust `active_config_surface.rs`; Nu callers consume the structured result via `yzx_core_bridge.nu` | landed owner cut | Rust now owns the bootstrap and Taplo rules; surviving Nu callers no longer need a separate parser owner |
| Runtime-env request construction | Rust `control_plane.rs` feeding `runtime-env.compute`; Nu callers use `compute_runtime_env_via_yzx_core` | landed owner cut | Request assembly no longer belongs to Nu; the surviving Nu seam is env application and argv execution only |
| Generated-state request construction and recording gate | Rust `control_plane.rs` feeding `config-state.compute` / `config-state.record` | landed owner cut | `config_state.nu` is deleted and the state hash logic plus request construction are Rust-owned |
| Shell or process orchestration | Nu `start_yazelix.nu`, `launch_yazelix.nu`, popup wrappers, `runtime_env.nu::run_runtime_argv`, POSIX wrappers | intentional | This is a real shell boundary until the launch/startup audit proves otherwise |
| Root/config/state dir resolution and home expansion | Nu `common.nu` and Rust `control_plane.rs` | accidental duplication | This seam is real, but it is blocked on the launch/startup audit because Nu still needs some runtime-root and shell-path discovery |
| Helper discovery, JSON envelope parsing, and final helper error rendering | Nu `yzx_core_bridge.nu` | temporary bridge debt | Still needed today, but too central to count as a purely transport-only seam |
| Final human-facing rendering | mostly Nu renderers such as `config_report_rendering.nu` and bridge-owned prose | intentional | Final prose can stay caller-local even when typed logic moves to Rust |

## 4. Survivor Reasons

- Rust `active_config_surface.rs`: `canonical_owner`
- Rust `runtime_env.rs`: `canonical_owner`
- Rust `config_state.rs`: `canonical_owner`
- Rust `control_plane.rs`: `canonical_owner`
- Nu `environment_bootstrap.nu`: `irreducible_shell_boundary`
- Nu startup/launch entrypoints in `core/start_yazelix.nu` and `core/launch_yazelix.nu`: `irreducible_shell_boundary`
- Nu `runtime_env.nu::run_runtime_argv`: `irreducible_shell_boundary`
- Nu `config_surfaces.nu`: `temporary_bridge_debt`
- deleted `config_parser.nu`: `landed_deletion`
- Nu `runtime_env.nu`: `irreducible_shell_boundary`
- Nu `yzx_core_bridge.nu`: `temporary_bridge_debt`
- Nu `common.nu` path/root helpers:
  - `resolve_yazelix_nu_bin` and shell-wrapper helpers: `irreducible_shell_boundary`
  - config/runtime/state-dir and path-expansion duplicates mirrored in Rust: `historical_debt`

If a future lane cannot keep one of those reason classes honest, the layer
should be deleted or narrowed.

## 5. Delete-First Findings

### Delete Now

- No product-code deletion is honest yet from this audit alone
- The immediately stale thing is planning prose, not live behavior:
  `docs/subsystem_code_inventory.md` and
  `docs/specs/rust_migration_matrix.md` still read as if the old
  config/state/env/preflight cleanup is materially finished even though live
  duplicate owners remain

### Bridge Layer To Collapse

- `runtime_env.nu` request construction is already gone; the remaining question
  is whether the explicit shell-exec seam can shrink further without hiding
  real process ownership
- `config_state.nu` is deleted; do not recreate a second Nu owner for
  config-state request assembly or canonical managed-surface resolution
- `yzx_core_bridge.nu` is still oversized bridge debt, but this audit records it
  as a later lane because its helper-resolution and error-surface contract still
  spans more than the local config/runtime callers

### Full-Owner Migration

- A real full-owner migration remains for the helper-resolution and bridge-error
  cluster around `yzx_core_bridge.nu`, but it should follow the public command
  surface and launch/startup audits rather than hiding inside this narrower
  config/runtime cut

### Likely Survivors

- Rust typed owners in `active_config_surface.rs`, `runtime_env.rs`,
  `config_state.rs`, and `control_plane.rs`
- Nu startup, launch, popup, and editor-exec boundaries that actually spawn
  processes or stage live shell env
- config metadata plus parity validators

### No-Go Deletions

- `common.nu` cannot lose all root/config/state path resolution yet
  Stop condition: `yazelix-rdn7.5.3` must first decide which launch/startup
  paths remain irreducibly Nu-owned and which env/root resolution belongs in a
  shared Rust owner
- `yzx_core_bridge.nu` cannot be deleted by this audit
  Stop condition: a later lane must preserve `CRCP-003` helper resolution and
  fail-fast error rendering for both packaged runtimes and source checkouts
- `runtime_env.nu` cannot disappear wholesale in this lane
  Stop condition: the surviving shell-exec boundary for popup/startup/launch
  flows must stay explicit instead of reappearing as ambient shell behavior

## 6. Quality Findings

- duplicate owners:
  - `config_surfaces.nu` and `active_config_surface.rs` both own canonical
    `user_configs` resolution, legacy-root rejection, default bootstrap, and
    Taplo synchronization
  - `runtime_env.nu` and Rust control-plane code both assemble runtime-env
    request inputs from runtime dir, home, PATH, and normalized config
  - `common.nu` and `control_plane.rs` both resolve config/state roots and
    `HOME`-derived defaults
- missing layer problems:
  - there is no dedicated spec for `active_config_surface.rs` as the canonical
    owner even though Rust already serves config, doctor, and control-plane
    callers with it
  - there is no sharply named contract for the current config-state request
    boundary; the state semantics are documented, but the surviving shim seam is
    still implicit
- extra layer problems:
- `yzx_core_bridge.nu` still centralizes helper discovery, JSON parsing, and
  error rendering for callers that could eventually consume a narrower Rust or
  caller-local surface
- DRY opportunities:
  - path expansion and XDG/HOME defaulting are duplicated between `common.nu`
    and `control_plane.rs`
  - contract-path and managed-surface derivation are rebuilt in multiple Nu
    wrappers instead of coming from one Rust-owned request source
- weak or orphan tests:
  - some default-lane config/runtime tests still rely on broad suite-level
    traceability rather than narrow contract IDs; the new protocol/quarantine
    machinery should tighten those before more deletions land
- only-known executable-defense tests:
  - `nu nushell/scripts/dev/validate_config_surface_contract.nu` for default
    config, Home Manager parity, and managed-surface behavior
  - helper-resolution cases in
    `nu nushell/scripts/dev/test_yzx_generated_configs.nu`
  - `rust_core/yazelix_core/src/config_state.rs` tests for rebuild-hash and
    record behavior
  - `nu nushell/scripts/dev/test_helix_managed_config_contracts.nu` and
    `nu nushell/scripts/dev/test_yzx_popup_commands.nu` for runtime-env
    propagation behavior
- spec gaps:
  - no single live spec says when Nu-side request construction is still allowed
    versus when it becomes a second owner
- docs drift:
  - `docs/subsystem_code_inventory.md` and
    `docs/specs/rust_migration_matrix.md` understate the remaining
    config/runtime bridge debt

## 7. Deletion Classes And Follow-Up Beads

| Bead | Retained behavior | Deletion class | Candidate surviving owner | Verification that must still pass | Explicit stop condition |
| --- | --- | --- | --- | --- | --- |
| `yazelix-izwm` | canonical `user_configs` ownership, default bootstrap, Taplo sync, legacy-root fail-fast, HM/default parity | `bridge_collapse` | Rust `active_config_surface.rs` plus caller-local Nu shell UX only where needed | `nu nushell/scripts/dev/validate_config_surface_contract.nu`; `nu nushell/scripts/dev/test_yzx_core_commands.nu` | stop if a surviving Nu caller still needs shell-side file bootstrap behavior that Rust cannot own without taking over user-managed external config |
| `yazelix-ekfc` | runtime-env parity and generated-state hash/record behavior without reviving ambient host inference | `bridge_collapse` | Rust `runtime_env.rs`, `config_state.rs`, and `control_plane.rs` plus explicit Nu shell-exec boundaries only | `rust_core/yazelix_core/tests/yzx_core_runtime_env.rs`; `rust_core/yazelix_core/src/config_state.rs`; `nu nushell/scripts/dev/test_helix_managed_config_contracts.nu`; `nu nushell/scripts/dev/test_yzx_generated_configs.nu`; `nu nushell/scripts/dev/validate_config_surface_contract.nu` | stop if a caller still depends on Nu-owned shell/process state that cannot be expressed as explicit Rust inputs without changing live behavior |

Follow-up lanes deliberately not created here:

- a `common.nu` root/config/state-dir collapse lane
  - blocked on `yazelix-rdn7.5.3`
- a `yzx_core_bridge.nu` end-to-end deletion lane
  - blocked on later public-command-surface and launcher audits that can keep
    `CRCP-003` honest

## Traceability

- Bead: `yazelix-rdn7.5.1`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
- Defended by: manual review of the cited Nushell, Rust, and spec paths in this audit
