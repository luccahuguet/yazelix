# Runtime Shell Floor Budgets

## Summary

The product/runtime Nushell floor is now the measured allowlist below:

| Family | Files | LOC |
| --- | ---: | ---: |
| Integration wrapper floor | `1` | `54` |
| Setup/bootstrap shell entry | `2` | `345` |
| Runtime helper seam | `6` | `611` |

Total product/runtime Nu outside `yzx/dev.nu`: `1,010` LOC across `9` files.

## Scope

In scope:

- the remaining runtime-side Nushell owners under `core/`, `setup/`, `utils/`,
  and `zellij_wrappers/`

Out of scope:

- `yzx/dev.nu`
- front-door Nu presentation (`welcome.nu`, `menu.nu`)

## Behavior

### Integration wrapper floor

Only one wrapper remains:

- `nushell/scripts/zellij_wrappers/launch_sidebar_yazi.nu`

Popup and menu launch trampolines are no longer Nu-owned. They now live in
checked-in POSIX helpers.

### Setup and bootstrap floor

The remaining shell entry files are:

- `nushell/scripts/core/start_yazelix_inner.nu`
- `nushell/scripts/setup/environment.nu`

They still own:

- shellhook env mutation and initializer generation
- welcome/startup sequencing
- the final `zellij` exec boundary

They no longer own the extra welcome/display path that had drifted back into
`environment.nu`.

### Runtime helper seam

The surviving runtime-side helper files are:

- `nushell/scripts/utils/constants.nu`
- `nushell/scripts/utils/runtime_commands.nu`
- `nushell/scripts/utils/runtime_defaults.nu`
- `nushell/scripts/utils/runtime_paths.nu`
- `nushell/scripts/utils/transient_pane_contract.nu`
- `nushell/scripts/utils/yzx_core_bridge.nu`

These survive only as the narrow remaining env/path/bridge helpers consumed by
the runtime shell owners above.

## Non-goals

- broad Rust wrapper insertions that leave the same startup/process boundary in
  place
- reviving deleted popup/menu/config/logging helper files in Nushell

## Acceptance Cases

1. The runtime-side wrapper floor is only the sidebar Yazi launcher in Nu
2. `environment.nu` reads as shellhook/env setup instead of a second welcome
   owner
3. The runtime helper allowlist matches the canonical budget exactly

## Verification

- `yzx_repo_validator validate-specs`
- `yzx_repo_validator validate-nushell-budget`
- `yzx_repo_validator validate-nushell-syntax`

## Traceability

- Bead: `yazelix-pw9j.3`
- Bead: `yazelix-pw9j.5`
- Bead: `yazelix-pw9j.6.2`
- Defended by: `yzx_repo_validator validate-specs`
- Defended by: `yzx_repo_validator validate-nushell-budget`
- Defended by: `yzx_repo_validator validate-nushell-syntax`

## Open Questions

- If a later runtime cut can delete `yzx_core_bridge.nu`, `runtime_paths.nu`, or
  the sidebar wrapper end-to-end instead of rewrapping them, ratchet the family
  again from the new measured tree
