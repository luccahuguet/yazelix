# Provable Nushell Floor Budget

## Summary

This document replaces the softer "likely Nushell survivor" framing with a
harder rule: every surviving Nu line is dangerous by default and must justify
itself against deletion, Rust ownership, asset ownership, or fixed POSIX
helpers.

The current measured surface is `26,678` tracked lines of Nushell across
`112` `.nu` files under `nushell/scripts/`.

The canonical hard target for the current backlog is `4,200` Nu lines. That is
intentionally aggressive and it assumes:

- `0` governed Nu tests survive
- `0` deterministic Nu validators survive
- maintainer and `yzx dev` Nu collapses to shell-only orchestration
- the remaining product/runtime Nu is forced down to a narrow shell, host, TTY,
  and presentation floor

This is the current top-level budget document for the under-`5k` push.

## Current Measured Surface

Measured on `2026-04-22` from the tracked tree after deleting
`nushell/scripts/yzx/enter.nu` under `yazelix-w6sz.5.2`.

| Family | Current included surface | Current LOC | Hard target LOC | Main beads |
| --- | --- | ---: | ---: | --- |
| Governed Nu tests | `nushell/scripts/dev/test_*.nu` | `13,115` | `0` | `yazelix-rdn7.4.5`, `yazelix-rdn7.4.7` |
| Deterministic Nu validators | `nushell/scripts/dev/validate*.nu` | `1,088` | `0` | `yazelix-rdn7.4.6`, `yazelix-rdn7.4.7` |
| Maintainer and `yzx dev` shell orchestration | `nushell/scripts/maintainer/*.nu`, `nushell/scripts/yzx/dev.nu`, residual non-test dev orchestration | `4,077` | `1,200` | `yazelix-8ih0`, `yazelix-8ih0.7`, `yazelix-8ih0.8` |
| Integration and popup wrapper glue | `nushell/scripts/integrations/*.nu`, `nushell/scripts/zellij_wrappers/*.nu` | `1,328` | `300` | `yazelix-w6sz.2` |
| Setup and bootstrap shell entry | `setup/environment.nu`, `setup/initializers.nu`, `core/start_yazelix.nu`, `core/start_yazelix_inner.nu`, `core/launch_yazelix.nu` | `1,073` | `500` | `yazelix-w6sz.3`, `yazelix-nuj1`, `yazelix-p18h` |
| Front-door UX and public shell presentation | `setup/welcome.nu`, `utils/ascii_art.nu`, `utils/upgrade_summary.nu`, `yzx/menu.nu`, `yzx/screen.nu`, `yzx/tutor.nu`, `yzx/whats_new.nu`, `yzx/popup.nu`, `yzx/edit.nu`, `yzx/import.nu` | `2,299` | `950` | `yazelix-w6sz.4`, `yazelix-dejl` |
| Runtime helpers, bridges, and shared utilities | `utils/*.nu` except `ascii_art.nu` and `upgrade_summary.nu` | `3,126` | `1,050` | `yazelix-lnk6`, `yazelix-dejl`, `yazelix-nuj1`, `yazelix-p18h` |
| Session and desktop host integration | `core/yzx_session.nu`, `yzx/desktop.nu`, `yzx/launch.nu` | `572` | `200` | `yazelix-w6sz.5` |

Combined hard target: `4,200` Nu LOC

This table now partitions the full tracked Nushell tree. There is no unnamed
"misc" budget left to hide unexpected Nu growth.

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

## Hard Nu Allowlist

Only these survivor classes are allowlisted by default:

| Allowlisted class | What may stay in Nu | What is not allowlisted |
| --- | --- | --- |
| Shell/bootstrap entrypoints | startup env export, shell initializer generation, `with-env` execution, checked-in POSIX handoff | typed request construction, config normalization, generated-state decisions, reusable metadata tables |
| External tool adapters | direct `zellij`, `ya`, terminal, `gh`, `bd`, `git`, Nix, or XDG argv execution plus nearby human-facing error rendering | duplicated live state, duplicated config parsing, bridge-local policy that Rust or data files can own |
| Interactive presentation control | minimal playback, keypress waiting, TTY sizing, `fzf` interaction, and shell-owned screen refresh logic | large data tables, random pools, style policy, duplicated copy assembly, renderer stacks in parallel |
| Maintainer repo orchestration | fixed argv routing for release/update/build/sync operations | dynamic `nu -c` dispatch, deterministic validation logic, test helper libraries, broad helper registries |
| Tiny transport seams | small env/fact marshalling at an actual shell boundary | cross-surface typed logic, reusable command metadata, large report rendering helpers |

Anything outside this allowlist must move to Rust, assets, or checked-in POSIX
helpers, or it must be deleted.

## Mechanical Gate

The live no-growth gate is now tracked in `config_metadata/nushell_budget.toml`
and enforced by the Rust-owned validator command:

- `yzx_repo_validator validate-nushell-budget`

That manifest is intentionally stricter than the prose budget alone:

- every currently tolerated Nu file must be listed explicitly
- each family has an exact file-count ceiling and LOC ceiling
- any new Nu file, missing manifest update, or family growth is out of contract
- transitional exceptions must name the owning deletion bead directly in the
  manifest

Until the repo reaches the `4,200`-LOC hard floor, this gate enforces
no-growth against the current tracked ceilings rather than pretending the final
floor already landed.

## Exception Policy

Exceptions are intentionally hostile to new Nu growth:

1. A retained or new Nu surface must name its allowlisted class and its
   irreducibility proof
2. The owning bead or spec must explain why Rust, assets, or fixed POSIX
   helpers would be worse
3. The exception must declare the exact family budget it consumes and the LOC
   it keeps alive
4. Temporary bridge exceptions must name the follow-up deletion bead before the
   code lands
5. Governed tests and deterministic validators do not get exceptions; their
   target remains `0`
6. Inline quoted shell-program assembly does not get exceptions; use checked-in
   POSIX helpers or structured argv execution instead
7. Once `yazelix-w6sz.7.2` lands, anything outside the allowlist or over the
   family budget is out of contract by default

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
- every retained family surface must fit one of the allowlisted survivor
  classes above
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
- `nix develop -c cargo run --manifest-path rust_core/Cargo.toml --bin yzx_repo_validator -- validate-nushell-budget`

## Traceability

- Bead: `yazelix-w6sz.6`
- Bead: `yazelix-w6sz.1`
- Bead: `yazelix-w6sz.7.1`
- Bead: `yazelix-w6sz.7.2`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
- Informed by: `docs/specs/ranked_nu_deletion_budget.md`
- Informed by: `docs/specs/likely_nushell_survivor_owner_cut_decisions.md`
