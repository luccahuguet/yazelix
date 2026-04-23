# Front-Door And Data Nushell Budget

## Summary

This document is the live delete-first budget for the surviving front-door Nu
surface after `yazelix-lj7z.8`.

The oversized data-heavy Nu owners are gone:

- `utils/ascii_art.nu`
- `utils/upgrade_summary.nu`
- `yzx/screen.nu`
- `yzx/tutor.nu`
- `yzx/whats_new.nu`

Rust now owns the retained renderer, Game of Life logic, upgrade-summary copy,
and those three public command bodies. The remaining Nu front-door floor is
`841` LOC across `5` files.

## Scope

In scope:

- `setup/welcome.nu`
- `utils/front_door_runtime.nu`
- `yzx/menu.nu`
- `yzx/edit.nu`
- `yzx/import.nu`

Out of scope:

- launch/bootstrap transport outside front-door presentation
- maintainer and sweep shells
- the already-landed Rust front-door owners in
  `front_door_render.rs`, `front_door_commands.rs`, and
  `upgrade_summary.rs`

## Current Measured Surface

Measured on `2026-04-23`:

| Surface | Current LOC | Hard target LOC | Notes |
| --- | ---: | ---: | --- |
| Front-door Nu floor | `841` | `500` | surviving shell presentation and process-handoff surface |
| Large front-door data owners in Nu | `0` | `0` | static art/spec tables and upgrade-summary shaping already moved out |

## Current Owner Split

### Rust-owned now

- welcome and `yzx screen` style resolution
- the retained random Game of Life pool
- Game of Life evolution and width-aware frame rendering
- `yzx screen`, `yzx tutor`, and `yzx whats_new`
- upgrade-summary loading, rendering, and last-seen state

### Nu-owned now

- startup-shell welcome sequencing and prompt gating in `setup/welcome.nu`
- tiny runtime handoff helpers in `utils/front_door_runtime.nu`
- `fzf`/popup/editor/process-heavy surfaces in `yzx/menu.nu`,
  `yzx/edit.nu`, and `yzx/import.nu`

## Remaining Deletion Budget

`yazelix-lj7z.8` is complete, but the family is not exempt from further cuts.
The next valid front-door deletions must focus on these seams:

1. `setup/welcome.nu`
   - keep only startup-shell sequencing, skip/logging behavior, and the final
     prompt-to-launch boundary
   - do not let it regain renderer, data, or summary ownership
2. `utils/front_door_runtime.nu`
   - keep only the smallest runtime bridge needed by welcome/startup callers
   - fold it away if a direct Rust command call deletes the file cleanly
3. `yzx/menu.nu`, `yzx/edit.nu`, `yzx/import.nu`
   - move deterministic planning and report shaping to Rust if that deletes the
     Nu owner end to end
   - keep Nu only where the surface is honestly `fzf`, popup, editor, or shell
     process orchestration

## Hard Rules

- Do not recreate a second renderer stack in Nu
- Do not move large static art or prose tables back into shell files
- Do not port shell-heavy popup/editor transport into Rust unless the result
  deletes the surviving Nu owner instead of wrapping it
- Keep the retained style contract in
  `docs/specs/welcome_screen_style_contract.md` authoritative

## Verification

- `yzx_repo_validator validate-specs`
- `yzx_repo_validator validate-nushell-budget`
- `cargo test -p yazelix_core --manifest-path rust_core/Cargo.toml`

## Traceability

- Bead: `yazelix-lj7z.8`
- Defended by: `yzx_repo_validator validate-specs`
- Defended by: `yzx_repo_validator validate-nushell-budget`
- Defended by: `cargo test -p yazelix_core --manifest-path rust_core/Cargo.toml`
