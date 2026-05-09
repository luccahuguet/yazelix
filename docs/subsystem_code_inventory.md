# Yazelix Delete-First Code Inventory

## Summary

This inventory tracks the remaining Nushell-owned surface after the v16
Rust-forward cuts. It is not a repo-wide LOC scoreboard and it is not an
implementation queue.

The current question is narrower:

1. Which surviving Nushell files are honest shell, UI, or process-bound owners?
2. Which files still contain deterministic seams worth shrinking later?

The ranked execution order belongs in Beads. This document records the current
owner map so future refactors do not reopen the same broad Rust-port question
without a concrete deletion target.

## Current Shape

The tracked Nushell surface is about `1,496` lines across `12` `.nu` files:

```text
nushell/config/config.nu
nushell/config/stack_prompt_guard.nu
nushell/scripts/core/start_yazelix_inner.nu
nushell/scripts/setup/environment.nu
nushell/scripts/setup/welcome.nu
nushell/scripts/utils/constants.nu
nushell/scripts/utils/runtime_commands.nu
nushell/scripts/utils/runtime_defaults.nu
nushell/scripts/utils/runtime_paths.nu
nushell/scripts/utils/yzx_core_bridge.nu
nushell/scripts/yzx/menu.nu
nushell/scripts/zellij_wrappers/launch_sidebar_yazi.nu
```

The old broad product-side Nushell owner set is gone. Root help, command
metadata, generated externs, runtime materialization, Yazi generation, Zellij
generation, terminal generation, Helix generation, public doctor, config edit,
import, workspace commands, update commands, and maintainer command dispatch are
Rust-owned or outside the remaining Nushell floor.

## Remaining Owner Decisions

| File | Current decision | Why |
| --- | --- | --- |
| `nushell/config/config.nu` | Retain | User shell config source; it wires generated initializers and externs into Nushell |
| `nushell/config/stack_prompt_guard.nu` | Retain | Interactive prompt guard logic is shell-local and not a product control-plane owner |
| `nushell/scripts/core/start_yazelix_inner.nu` | Retain | Owns final interactive startup handoff, welcome display sequencing, startup profiling boundaries, session snapshot env mutation, and Zellij process launch |
| `nushell/scripts/setup/environment.nu` | Retain with follow-ups | Shellhook setup, initializer generation, extern sync, and source-checkout executable repair are shell-bound, but initializer generation and chmod repair are deterministic enough to re-evaluate |
| `nushell/scripts/setup/welcome.nu` | Retain | Human-facing welcome rendering and prompt gating remain a good Nushell fit |
| `nushell/scripts/utils/constants.nu` | Retain | Tiny compatibility export for runtime version and static metadata access |
| `nushell/scripts/utils/runtime_commands.nu` | Retain | Shell-facing default-shell resolution and command assembly support startup handoff |
| `nushell/scripts/utils/runtime_defaults.nu` | Retain | Tiny shared constant module for shell defaults |
| `nushell/scripts/utils/runtime_paths.nu` | Retain | Shell/env path resolution for remaining Nushell entrypoints |
| `nushell/scripts/utils/yzx_core_bridge.nu` | Shrink later | Still contains the shared Rust-helper transport and error surface; it should collapse toward a minimal transport helper rather than grow domain policy |
| `nushell/scripts/yzx/menu.nu` | Retain | The command palette is the honest `fzf`/interactive menu boundary over Rust-owned command metadata |
| `nushell/scripts/zellij_wrappers/launch_sidebar_yazi.nu` | Retain | Thin process wrapper for launching the managed Yazi sidebar at the Zellij boundary |

## Follow-Up Beads

The deterministic migration candidates are split into follow-up beads instead
of being mixed into this inventory audit:

- `yazelix-6h1n.4.1` — Collapse `yzx_core_bridge.nu` to a minimal transport helper
- `yazelix-6h1n.4.2` — Move source-checkout runtime script chmod repair out of shellhook
- `yazelix-6h1n.4.3` — Re-evaluate `setup/environment.nu` initializer generation ownership

Those follow-ups should use the same delete-first bar: a migration only counts
when the Nushell owner shrinks end-to-end or a deterministic invariant moves to
a clearer existing owner.

## Non-Goals

- Rewriting all shell glue into Rust
- Reintroducing deleted Nushell command registries or materialization wrappers
- Adding Rust helpers that leave the same Nushell owner in place
- Treating interactive presentation, shellhook behavior, or Zellij/Yazi process
  handoff as Rust targets without a concrete deletion payoff

## Verification

- `yzx_repo_validator validate-nushell-syntax`
- `yzx_repo_validator validate-contracts`
- `yzx dev test` when a follow-up changes command routing, generated-state
  behavior, or shell/runtime integration
