# Runtime Shell Floor Contract

## Summary

The product/runtime Nushell floor is now the measured allowlist below:

| Family | Files | LOC |
| --- | ---: | ---: |
| Setup/bootstrap shell entry | `2` | `366` |
| Front-door presentation floor | `1` | `168` |
| Runtime helper seam | `5` | `488` |

Total tracked product/runtime Nu: `1,022` LOC across `8` files.

## Scope

In scope:

- the remaining runtime-side Nushell owners under `core/`, `setup/`, and
  `utils/`

Out of scope:

- maintainer command orchestration, which is Rust-owned through `yzx_control dev`

## Behavior

### Integration wrapper floor

No Zellij integration wrapper remains in Nushell. The managed left Yazi sidebar
is launched through the Rust-owned `yzx sidebar yazi` command, and popup, command
menu, and config UI floating panes are configured `yzpp` popups instead of Nu
wrapper trampolines.

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
- `nushell/scripts/utils/yzx_core_bridge.nu`

These survive only as the narrow remaining env/path/bridge helpers consumed by
the runtime shell owners above.

### Front-door presentation floor

The surviving presentation file is:

- `nushell/scripts/setup/welcome.nu`

This remains Nu-owned while it is mostly human-facing terminal presentation and
startup-shell UX. `yzx menu` moved to the Rust control plane, with `fzf`
remaining an external selector process.

## Non-goals

- broad Rust wrapper insertions that leave the same startup/process boundary in
  place
- reviving deleted config/logging helper files in Nushell

## Acceptance Cases

1. No runtime-side Nushell wrapper file remains
2. `environment.nu` reads as shellhook/env setup instead of a second welcome
   owner
3. The runtime helper allowlist is reviewed directly against the retained shell floor

## Verification

- `yzx_repo_validator validate-contracts`
- `yzx_repo_validator validate-nushell-syntax`

## Open Questions

- If a later runtime cut can delete `yzx_core_bridge.nu`, `runtime_paths.nu`, or
  the sidebar wrapper end-to-end instead of rewrapping them, update the retained
  shell floor from the new measured tree
