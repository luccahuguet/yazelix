# Provable Nushell Floor Budget

## Summary

This document replaces the softer "likely Nushell survivor" framing with a
harder rule: every surviving Nu line is dangerous by default and must justify
itself against deletion, Rust ownership, asset ownership, or fixed POSIX
helpers.

The current measured surface is `29,005` lines of Nushell across
`nushell/scripts/`.

The canonical target floor for the current backlog is `3,950` Nu lines. That
is intentionally aggressive, but it still leaves room for the small shell,
TTY, host-integration, and presentation seams that have not yet been proven
movable without worse wrapper debt.

This is the current top-level budget document for the under-`5k` push.

## Current Measured Surface

Measured on `2026-04-22` from the tracked tree:

| Family | Current included surface | Current LOC | Hard target LOC | Main beads |
| --- | --- | ---: | ---: | --- |
| Governed tests and deterministic validators | `nushell/scripts/dev/test_*.nu`, `validate*.nu`, `validate_*.nu` | `15,792` | `0` | `yazelix-rdn7.4.5`, `yazelix-rdn7.4.6`, `yazelix-rdn7.4.7` |
| Maintainer and `yzx dev` shell orchestration | `nushell/scripts/maintainer/*.nu`, `nushell/scripts/yzx/dev.nu`, residual non-test dev orchestration | `4,224` | `700` | `yazelix-8ih0`, `yazelix-8ih0.7`, `yazelix-8ih0.8` |
| Integration and popup wrapper glue | `nushell/scripts/integrations/*.nu`, `nushell/scripts/zellij_wrappers/*.nu` | `1,340` | `350` | `yazelix-w6sz.2` |
| Setup and bootstrap shell entry | `nushell/scripts/setup/*.nu`, `core/start_yazelix.nu`, `core/start_yazelix_inner.nu`, `core/launch_yazelix.nu` | `1,289` | `600` | `yazelix-w6sz.3`, `yazelix-nuj1`, `yazelix-p18h` |
| Front-door UX and public shell presentation | `utils/ascii_art.nu`, `utils/upgrade_summary.nu`, `yzx/menu.nu`, `yzx/screen.nu`, `yzx/tutor.nu`, `yzx/whats_new.nu`, `yzx/popup.nu`, `yzx/edit.nu`, `yzx/import.nu` | `2,263` | `1,100` | `yazelix-w6sz.4`, `yazelix-dejl` |
| Runtime helpers, bridges, and shared data-shaped utilities | `utils/yzx_core_bridge.nu`, `utils/common.nu`, `utils/terminal_launcher.nu`, `utils/version_info.nu`, `utils/constants.nu`, `utils/config_schema.nu`, `utils/startup_profile.nu`, `utils/doctor_fix.nu` | `2,180` | `950` | `yazelix-lnk6`, `yazelix-dejl`, `yazelix-p18h` |
| Session and desktop host integration | `core/yzx_session.nu`, `yzx/desktop.nu`, `yzx/launch.nu` | `572` | `250` | `yazelix-w6sz.5` |

Combined hard target: `3,950` Nu LOC

## Rust-First Proof Standard

Use this decision order for every remaining Nu surface:

1. Delete it outright
2. Move the retained logic to Rust
3. Move static payloads or tables to assets or data files
4. Move fixed shell bodies to checked-in POSIX helpers
5. Keep a narrow Nu remainder only if the file, export, or branch has a
   concrete irreducibility proof

Accepted irreducibility proofs are narrow:

- direct shell initialization or shell startup integration
- direct TTY or external command interaction that would only be re-wrapped
- direct host integration such as XDG desktop entry side effects
- direct presentation logic whose retained value is still the renderer itself,
  not the data it renders

Rejected "proofs" are broad or lazy:

- "this area is UX-heavy"
- "this area is shell-heavy"
- "a broad Rust rewrite is not honest"
- "the file is smaller now"
- "the current Nu path already works"

Those arguments can justify not doing a fake wrapper rewrite. They cannot
justify leaving large mixed Nu owners unchallenged.

## Superseded Assumptions

These earlier assumptions are no longer sufficient on their own:

| Earlier assumption | Status now | Current rule | Follow-up lane |
| --- | --- | --- | --- |
| setup, welcome, and bootstrap did not have an honest broad Rust owner cut | superseded as a stopping rule | lack of a broad Rust port does not excuse large surviving Nu; the family still has to collapse to the smallest provable shell floor | `yazelix-w6sz.3` |
| front-door UX did not have an honest broad Rust owner cut | superseded as a stopping rule | lack of a broad Rust port does not excuse large renderer Nu; branches, copy, and data still have to collapse aggressively | `yazelix-w6sz.4`, `yazelix-dejl` |
| session and desktop command bodies were still shell-heavy | superseded as a stopping rule | shell- and host-heavy code still must collapse to the smallest provable host-integration floor | `yazelix-w6sz.5` |
| maintainer and dev paths were allowed to remain broadly Nu because they are operational | superseded | only direct shell orchestration may survive; metadata, policy, routing, and deterministic surfaces must leave Nu | `yazelix-8ih0` |
| launch/runtime helpers needed only smaller cuts | reaffirmed but hardened | the smaller cuts still stand, but the surviving helper floor now needs proof, not just a softer "honest survivor" note | `yazelix-lnk6`, `yazelix-nuj1`, `yazelix-p18h` |

## Hard Budget Rules

- `0` governed Nu tests survive
- `0` deterministic Nu validators survive as a long-term owned surface
- every family budget above should bias toward deleting whole files rather than
  trimming the same ownership across more files
- any bead that cannot meet its family target must explain why in terms of a
  retained irreducibility proof, not in terms of comfort or historical habit
- new Nu growth is out of contract once `yazelix-w6sz.7` lands unless it has an
  explicit allowlisted exception

## Cut Order

The current aggressive order is:

1. lock the proof standard and hard family budgets
   - `yazelix-w6sz.6`
   - `yazelix-w6sz.1`
2. lock the Rust test runner default so Nu tests and mixed harness debt do not
   leak back in
   - `yazelix-rdn7.4.7`
3. remove governed tests and deterministic validators from Nu
   - `yazelix-rdn7.4.5`
   - `yazelix-rdn7.4.6`
4. shrink maintainer and `yzx dev` to shell orchestration only
   - `yazelix-8ih0`
5. collapse integration glue and wrapper owners
   - `yazelix-w6sz.2`
6. collapse setup/bootstrap and the launch/runtime helper floor together
   - `yazelix-w6sz.3`
   - `yazelix-lnk6`
   - `yazelix-nuj1`
   - `yazelix-p18h`
7. collapse front-door UX and delete data-heavy Nu
   - `yazelix-w6sz.4`
   - `yazelix-dejl`
8. collapse session and desktop host integration
   - `yazelix-w6sz.5`
9. enforce the floor mechanically
   - `yazelix-w6sz.7`

## Why The Floor Is Not Zero

The current backlog does not yet prove that every last surviving Nu surface can
leave the repo without worse complexity.

The under-`5k` target is therefore the current provable floor, not the
philosophical lower bound. It is intentionally small enough that any remaining
Nu must be narrow, obvious, and hard to challenge. If later beads prove even
those remainders movable, the floor should fall again.

## Verification

- `nu nushell/scripts/dev/validate_specs.nu`
- manual LOC measurement with `find`, `wc`, and family-specific file lists

## Traceability

- Bead: `yazelix-w6sz.6`
- Bead: `yazelix-w6sz.1`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
- Informed by: `docs/specs/ranked_nu_deletion_budget.md`
- Informed by: `docs/specs/likely_nushell_survivor_owner_cut_decisions.md`
