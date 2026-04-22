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
  `utils/config_schema.nu`

Out of scope:

- launch/bootstrap transport
- maintainer/dev shells
- deterministic validators and governed tests

## Current Measured Surface

Measured on `2026-04-22`:

| Surface | Current LOC | Hard target LOC | Notes |
| --- | ---: | ---: | --- |
| Front-door UX family | `2,442` | `950` | full front-door renderer and public shell presentation family |
| Data-heavy subset | `1,938` | `350` | subset only; counts are not additive because these files overlap the front-door and runtime-helper families |

The data-heavy subset is:

- `utils/ascii_art.nu`
- `utils/constants.nu`
- `utils/config_schema.nu`
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
- `utils/config_schema.nu`
  - keep only the smallest live contract bridge if one still exists
  - move schema/data shaping to Rust or data files
- `utils/upgrade_summary.nu`
  - keep only small render glue if needed
  - move static copy/data shaping out of Nu

Hard rule:

Data tables do not get to survive in Nu just because the surrounding file still
has some shell-owned rendering behavior.

Stop condition:

Do not preserve dead assets or style aliases for compatibility comfort. Keep
only the styles and payloads backed by the live contract in
`docs/specs/welcome_screen_style_contract.md`.

## Verification

- `nu nushell/scripts/dev/validate_specs.nu`
- later implementation beads must keep the governed front-door contracts and
  style validations green

## Traceability

- Bead: `yazelix-w6sz.4.1`
- Bead: `yazelix-dejl.1`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
- Informed by: `docs/specs/setup_shellhook_welcome_terminal_canonicalization_audit.md`
- Informed by: `docs/specs/welcome_screen_style_contract.md`
- Informed by: `docs/specs/provable_nushell_floor_budget.md`
