# Runtime Shell Floor Contract

## Summary

The product/runtime Nushell floor is now the measured allowlist below:

| Family | Files | LOC |
| --- | ---: | ---: |
| Integration wrapper floor | `3` | `90` |
| Setup/bootstrap shell entry | `2` | `345` |
| Runtime helper seam | `6` | `611` |

Total product/runtime Nu outside `yzx/dev.nu`: `1,046` LOC across `11` files.

## Scope

In scope:

- the remaining runtime-side Nushell owners under `core/`, `setup/`, `utils/`,
  and `zellij_wrappers/`

Out of scope:

- `yzx/dev.nu`
- front-door Nu presentation (`welcome.nu`, `menu.nu`)

## Behavior

### Integration wrapper floor

Three wrappers remain:

- `nushell/scripts/zellij_wrappers/launch_sidebar_yazi.nu`
- `nushell/scripts/zellij_wrappers/yzx_popup_program.nu`
- `nushell/scripts/zellij_wrappers/yzx_menu_popup.nu`

Popup and menu launch trampolines remain Nu-owned as tiny wrapper seams because
Yazelix already depends on Nushell at that boundary and the Nu versions are the
clearer shipped owner.

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
- reviving deleted config/logging helper files in Nushell

## Acceptance Cases

1. The runtime-side wrapper floor is only the sidebar, popup, and menu wrappers in Nu
2. `environment.nu` reads as shellhook/env setup instead of a second welcome
   owner
3. The runtime helper allowlist matches the canonical budget exactly

## Verification

- `yzx_repo_validator validate-contracts`
- `yzx_repo_validator validate-nushell-budget`
- `yzx_repo_validator validate-nushell-syntax`

## Verification

- `yzx_repo_validator validate-contracts`
- `yzx_repo_validator validate-nushell-budget`
- `yzx_repo_validator validate-nushell-syntax`

## Open Questions

- If a later runtime cut can delete `yzx_core_bridge.nu`, `runtime_paths.nu`, or
  the sidebar wrapper end-to-end instead of rewrapping them, ratchet the family
  again from the new measured tree
