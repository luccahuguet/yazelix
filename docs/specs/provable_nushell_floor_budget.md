# Provable Nushell Floor Budget

## Summary

`yazelix-pw9j` finishes the old under-`5k` push and replaces it with a literal
measured allowlist.

The tracked Nushell floor is now `1,880` LOC across `14` files under
`nushell/scripts/`.

This is no longer a transitional ceiling. It is the current irreducible core:
every surviving file is named explicitly, every family has an exact budget, and
anything outside that allowlist is out of contract by default.

## Why

The old `4.1k` budget still mixed real shell boundaries with transitional debt.
That made it too easy to talk about an "irreducible floor" while still carrying
files that were only surviving because nobody had deleted them yet.

The repo now needs a harder stop condition:

- deterministic maintainer/update/sweep/plugin-build ownership is Rust-owned
- dead helper files are deleted instead of grandfathered
- popup and menu wrappers remain Nu-owned for consistency with the shipped shell
  boundary
- the remaining Nu files must justify themselves as the real shell/TTY/process
  boundary

## Scope

In scope:

- the canonical Nushell budget manifest in `config_metadata/nushell_budget.toml`
- the surviving Nu owner families under `nushell/scripts/`
- the literal survivor classes that may still live in Nushell

Out of scope:

- broader public-CLI Rust migration beyond the current v15 shell boundary
- deleting welcome/menu/startup shells by hiding the same process control behind
  a fake Rust wrapper
- non-Nushell runtime code

## Behavior

The surviving Nu floor is exactly these families:

| Family | Files | LOC | Why it still qualifies |
| --- | ---: | ---: | --- |
| Maintainer and `yzx dev` shell surface | `1` | `425` | public maintainer routing plus the startup-profile shell harness |
| Integration wrapper floor | `3` | `90` | sidebar, popup, and menu wrappers stay as tiny Nu trampolines at the live shell boundary |
| Setup and bootstrap | `2` | `345` | shellhook env mutation, initializer generation, welcome/startup sequencing, and the final `zellij` exec |
| Front-door presentation | `2` | `409` | direct TTY/UI control for welcome and the interactive command palette |
| Runtime helper seam | `6` | `611` | the narrow remaining path/env/bridge helpers consumed by those shell surfaces |

Exact allowlisted files:

- `nushell/scripts/core/start_yazelix_inner.nu`
- `nushell/scripts/setup/environment.nu`
- `nushell/scripts/setup/welcome.nu`
- `nushell/scripts/utils/constants.nu`
- `nushell/scripts/utils/runtime_commands.nu`
- `nushell/scripts/utils/runtime_defaults.nu`
- `nushell/scripts/utils/runtime_paths.nu`
- `nushell/scripts/utils/transient_pane_contract.nu`
- `nushell/scripts/utils/yzx_core_bridge.nu`
- `nushell/scripts/yzx/dev.nu`
- `nushell/scripts/yzx/menu.nu`
- `nushell/scripts/zellij_wrappers/launch_sidebar_yazi.nu`
- `nushell/scripts/zellij_wrappers/yzx_popup_program.nu`
- `nushell/scripts/zellij_wrappers/yzx_menu_popup.nu`

Allowed survivor classes are now literal:

- shellhook env mutation and shell initializer generation
- final shell/process/TTY handoff into `zellij`, `fzf`, or the configured tools
- direct interactive presentation logic whose value is the live terminal
  interaction itself
- tiny shell-bound helper seams that still pass env/path facts to those owners

These classes are explicitly disallowed from surviving in Nushell now:

- path or helper discovery that can move below the shell boundary
- JSON envelope parsing and broad error/report shaping
- deterministic config/state/report planning
- maintainer update/build/sweep policy
- dead helpers with no callers
- duplicated popup/menu wrapper logic outside the shipped Nu shell boundary

## Non-goals

- claiming the floor is philosophically minimal for all future Yazelix versions
- porting the remaining startup/welcome/menu/profile shells just to move the LOC
  number without deleting the real shell owner
- keeping any unnamed "miscellaneous" Nushell budget

## Acceptance Cases

1. `config_metadata/nushell_budget.toml` matches the measured tracked Nu floor
   exactly
2. Every surviving Nu file fits one of the explicit survivor classes above
3. Any new Nu file or family growth is out of contract unless the allowlist and
   this decision are updated deliberately

## Verification

- `yzx_repo_validator validate-specs`
- `yzx_repo_validator validate-nushell-budget`
- `yzx_repo_validator validate-nushell-syntax`

## Traceability

- Bead: `yazelix-pw9j.1`
- Bead: `yazelix-pw9j.7`
- Bead: `yazelix-uz6m`
- Defended by: `yzx_repo_validator validate-specs`
- Defended by: `yzx_repo_validator validate-nushell-budget`
- Defended by: `yzx_repo_validator validate-nushell-syntax`

## Open Questions

- If a later change can delete one of the remaining startup/profile/menu shells
  end-to-end instead of wrapping it, the floor should ratchet again from the
  new measured tree rather than by reopening the old `4.1k` planning budget
