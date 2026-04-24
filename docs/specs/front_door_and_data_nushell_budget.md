# Front-Door And Data Nushell Budget

## Summary

The front-door Nushell floor is now `409` LOC across two files:

- `nushell/scripts/setup/welcome.nu`
- `nushell/scripts/yzx/menu.nu`

The old data-heavy front-door owners are already gone. What remains is the live
TTY/UI control that still belongs beside the welcome flow and command palette.

## Scope

In scope:

- `setup/welcome.nu`
- `yzx/menu.nu`

Out of scope:

- Rust-owned front-door rendering/data surfaces
- startup/bootstrap shellhook logic outside the front-door UX

## Behavior

Retained Nu ownership is limited to:

- interactive welcome display, logging, and prompt gating
- the `fzf`-driven command palette and popup interaction loop

The following things are no longer allowed back into these files:

- large static art/data tables
- upgrade-summary shaping
- deterministic command metadata ownership
- broad report formatting that Rust already owns

## Non-goals

- porting the live `fzf`/keypress/TTY flow into Rust unless that deletes the Nu
  owner cleanly
- reintroducing deleted front-door data files in shell code

## Acceptance Cases

1. The front-door Nu floor is only `welcome.nu` and `menu.nu`
2. Those files still own direct TTY/UI interaction rather than deterministic
   metadata/data surfaces
3. The family stays at `409` LOC / `2` files in the canonical budget

## Verification

- `yzx_repo_validator validate-specs`
- `yzx_repo_validator validate-nushell-budget`

## Traceability

- Bead: `yazelix-pw9j.6`
- Bead: `yazelix-pw9j.6.1`
- Defended by: `yzx_repo_validator validate-specs`
- Defended by: `yzx_repo_validator validate-nushell-budget`

## Open Questions

- If a later cut can delete one of these files without rebuilding the same live
  terminal interaction in another shell wrapper, ratchet the family again
