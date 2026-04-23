# Second-Wave Nushell Deletion Map

## Summary

This map is the live post-cut state for the second hard Nu-deletion tranche.

Measured on `2026-04-23` after `yazelix-lj7z.4`, `yazelix-lj7z.8`, and
`yazelix-lj7z.9`, the tracked `nushell/scripts/**` surface is now `7,781` LOC
across `62` files. The hard floor remains `4,200` LOC.

The important change is not just the lower count. The biggest remaining Nu
families are now narrower:

- front-door presentation no longer owns `ascii_art.nu`,
  `upgrade_summary.nu`, `yzx/screen.nu`, `yzx/tutor.nu`, or
  `yzx/whats_new.nu`
- maintainer Nu no longer owns deterministic issue-sync or version-bump policy
- shell-heavy E2E runners no longer sprawl across four transitional files

## Current Budget Owners

| Family | Current LOC | Target LOC | Live owner |
| --- | ---: | ---: | --- |
| Manual E2E and sweep runners | `325` | `0` | `yazelix-lj7z.9` |
| Maintainer and `yzx dev` orchestration | `2,760` | `900` | `yazelix-lj7z.4` |
| Integration and wrappers | `1,198` | `300` | `yazelix-lj7z.7` |
| Setup and bootstrap | `621` | `500` | `yazelix-lj7z.6` |
| Front-door presentation | `841` | `500` | `yazelix-lj7z.8` |
| Runtime helpers | `2,036` | `1,050` | `yazelix-lj7z.5`, `yazelix-lj7z.10` |

## Landed Owner Cuts

### `yazelix-lj7z.8`

Rust now owns the large deterministic front-door surfaces:

- `yzx screen`
- `yzx tutor`
- `yzx whats_new`
- upgrade-summary loading, copy shaping, and seen-version state
- the retained welcome/screen renderer and Game of Life engine

Deleted Nu owners:

- `nushell/scripts/utils/ascii_art.nu`
- `nushell/scripts/utils/upgrade_summary.nu`
- `nushell/scripts/yzx/screen.nu`
- `nushell/scripts/yzx/tutor.nu`
- `nushell/scripts/yzx/whats_new.nu`

Remaining Nu front-door floor:

- `nushell/scripts/setup/welcome.nu`
- `nushell/scripts/utils/front_door_runtime.nu`
- `nushell/scripts/yzx/menu.nu`
- `nushell/scripts/yzx/edit.nu`
- `nushell/scripts/yzx/import.nu`

That remaining floor is no longer a renderer/data owner. It is now mostly shell
presentation, popup/editor/process handoff, and startup-shell integration.

### `yazelix-lj7z.4`

Rust now owns the deterministic maintainer policy for:

- version bump validation and release-note rotation
- GitHub/Beads lifecycle reconciliation
- canonical Beads comment sync

Deleted Nu owners:

- `nushell/scripts/maintainer/issue_bead_contract.nu`
- `nushell/scripts/maintainer/issue_sync.nu`
- `nushell/scripts/maintainer/repo_checkout.nu`
- `nushell/scripts/maintainer/version_bump.nu`

Remaining Nu maintainer floor:

- `nushell/scripts/maintainer/update_workflow.nu`
- `nushell/scripts/maintainer/plugin_build.nu`
- `nushell/scripts/yzx/dev.nu`
- demo, sweep, and test-helper shells under `nushell/scripts/dev/`

The remaining Nu is still too large, but it is now more honestly external-tool
and workflow heavy.

### `yazelix-lj7z.9`

The shell-heavy E2E runner cluster is no longer a five-file transitional
bucket.

Deleted Nu runners:

- `historical_upgrade_notes_e2e_runner.nu`
- `stale_config_diagnostics_e2e_runner.nu`
- `upgrade_contract_e2e_runner.nu`
- `upgrade_summary_e2e_runner.nu`

Rust replacements landed where the contracts were actually deterministic:

- `rust_core/yazelix_core/tests/repo_upgrade_contract.rs`
- `rust_core/yazelix_core/tests/yzx_control_front_door.rs`
- existing Rust `yzx_control` startup/runtime regressions

Remaining Nu runner:

- `nushell/scripts/dev/config_sweep_runner.nu`

That file survives only because it still owns real shell/TTY/matrix execution.
It is no longer defended as a placeholder bucket for unrelated deterministic
checks.

## Remaining Delete-First Order

1. Collapse the runtime-helper and bridge family again until `yzx_core` and
   adjacent Nu helpers stop owning broad request shaping
2. Delete more maintainer and `yzx dev` shell routing after the remaining
   update/plugin/sweep surfaces get narrower explicit owners
3. Re-evaluate the front-door shell floor file by file, especially
   `yzx/menu.nu`, `yzx/edit.nu`, and `yzx/import.nu`
4. Keep reducing live Nu budget ceilings in `config_metadata/nushell_budget.toml`
   instead of treating current survivors as permanent

## Stop Conditions

- Do not revive deleted Nu compatibility wrappers just to preserve an old file
  layout
- Do not move shell-heavy TTY/process execution into Rust unless it deletes a
  real Nu owner instead of adding another bridge
- Do not port weak or duplicate tests; only deterministic contract coverage
  should survive into Rust

## Verification

- `yzx_repo_validator validate-nushell-budget`
- `yzx_repo_validator validate-specs`
- `cargo test -p yazelix_core --manifest-path rust_core/Cargo.toml`

## Traceability

- Bead: `yazelix-lj7z`
- Bead: `yazelix-lj7z.1`
- Bead: `yazelix-lj7z.4`
- Bead: `yazelix-lj7z.8`
- Bead: `yazelix-lj7z.9`
- Defended by: `yzx_repo_validator validate-nushell-budget`
- Defended by: `yzx_repo_validator validate-specs`
