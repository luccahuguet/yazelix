# Front-Door And Data Nushell Budget

## Summary

This document defines the delete-first budget for the remaining front-door UX
and data-heavy Nushell surfaces.

The retained value here is the actual interactive shell presentation, not large
data tables, random pools, duplicated style policy, or copied prose assembly.
Those oversized deterministic owners should move to assets, Rust-owned
metadata, or much smaller retained render glue.

## Scope

In scope:

- `setup/welcome.nu`
- `utils/ascii_art.nu`
- `utils/upgrade_summary.nu`
- `yzx/menu.nu`
- `yzx/screen.nu`
- `yzx/tutor.nu`
- `yzx/whats_new.nu`
- `yzx/popup.nu`
- `yzx/edit.nu`
- `yzx/import.nu`
- the large data-bearing subsets inside `utils/constants.nu` and
  the surviving front-door/runtime presentation helpers

Out of scope:

- launch/bootstrap transport
- maintainer/dev shells
- deterministic validators and governed tests

## Current Measured Surface

Measured on `2026-04-22`:

| Surface | Current LOC | Hard target LOC | Notes |
| --- | ---: | ---: | --- |
| Front-door UX family | `2,281` | `950` | full front-door renderer and public shell presentation family |
| Data-heavy subset | `1,262` | `350` | subset only; counts are not additive because these files overlap the front-door and runtime-helper families |

The data-heavy subset is:

- `utils/ascii_art.nu`
- `utils/constants.nu`
- `utils/upgrade_summary.nu`

## `yazelix-w6sz.4.1` Front-Door UX Budget

Retain only the honest shell presentation seams:

- keypress waiting and width-aware playback
- `fzf` interaction in `yzx/menu.nu`
- minimal screen-entry and popup-entry transport
- the smallest copy/render logic that is still inseparable from the live shell
  UX

Delete or move:

- stale or weakly-defended styles
- random-pool policy outside the canonical style contract
- duplicated welcome-message assembly
- parallel renderer stacks
- large static frame data or tables

Candidate surviving owners:

- smaller `setup/welcome.nu`
- smaller `yzx/menu.nu`
- smaller `yzx/screen.nu`
- tiny retained presentation wrappers for `tutor`, `whats_new`, `popup`,
  `edit`, and `import`

Stop condition:

Do not build a parallel Rust renderer unless it deletes the oversized Nu owner
end to end. Delete stale styles and data first.

## `yazelix-dejl.1` Data-Heavy Budget

The data-heavy lane is a subset deletion budget inside the front-door and
runtime-helper families.

Target outcomes:

- `utils/ascii_art.nu`
  - delete stale style/data branches
  - move large frame or deterministic pattern payloads out of broad Nu owner
    code
- `utils/constants.nu`
  - keep only small irreducible runtime constants
  - move large static tables or policy maps out of Nu
- `utils/upgrade_summary.nu`
  - keep only small render glue if needed
  - move static copy/data shaping out of Nu

Completed cuts:

- `yazelix-w6sz.3.2` removed the dead `setup/environment.nu` import of
  `utils/config_schema.nu`
- `yazelix-dejl.4` deleted `utils/config_schema.nu`; Rust
  `config_normalize.rs` and `doctor_config_report.rs` own the retained schema
  diagnostics
- `yazelix-w6sz.4.2` removed the separate `utils/upgrade_notes.nu` series
  lookup and kept welcome release copy on the existing `upgrade_summary.nu`
  path

Hard rule:

Data tables do not get to survive in Nu just because the surrounding file still
has some shell-owned rendering behavior.

Stop condition:

Do not preserve dead assets or style aliases for compatibility comfort. Keep
only the styles and payloads backed by the live contract in
`docs/specs/welcome_screen_style_contract.md`.

## `yazelix-dejl.5` Ascii-Art Engine Decision

Decision after `yazelix-dejl.2`:

- static style tables, welcome copy, and Game of Life seed shapes should leave
  `ascii_art.nu`
- the remaining `ascii_art.nu` engine does not yet have an honest Rust owner
  cut

Reason:

- the surviving code is now mostly width-aware frame composition, ANSI-aware
  rendering, and live Game of Life state evolution that is consumed directly by
  the shell-owned `setup/welcome.nu` and `yzx/screen.nu` surfaces
- moving that code to Rust right now would not delete the shell-owned playback,
  interruptibility, and terminal-size boundaries
- without a narrower retained renderer contract, a Rust port would mostly
  recreate the same front-door engine behind another bridge

Explicit stop condition:

- do not start a broad Rust port of the remaining `ascii_art.nu` engine until
  a later front-door owner cut deletes `welcome.nu` plus `ascii_art.nu`
  materially end to end, or until the retained style surface is narrowed enough
  that the surviving renderer becomes a clearly smaller typed owner

Retained Nu floor for now:

- style resolution against the canonical welcome/screen contract
- width-aware line composition and ANSI coloring
- live Game of Life state stepping and rendering

Follow-up expectation:

- keep deleting data and duplicated style policy first
- only reopen a Rust engine lane if it deletes the remaining front-door Nu
  owner instead of wrapping it

## Verification

- `yzx_repo_validator validate-specs`
- later implementation beads must keep the governed front-door contracts and
  style validations green

## Traceability

- Bead: `yazelix-w6sz.4.1`
- Bead: `yazelix-w6sz.4.2`
- Bead: `yazelix-dejl.1`
- Bead: `yazelix-dejl.4`
- Defended by: `yzx_repo_validator validate-specs`
- Informed by: `docs/specs/setup_shellhook_welcome_terminal_canonicalization_audit.md`
- Informed by: `docs/specs/welcome_screen_style_contract.md`
- Informed by: `docs/specs/provable_nushell_floor_budget.md`
