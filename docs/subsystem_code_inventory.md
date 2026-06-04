# Yazelix Delete-First Code Inventory

## Summary

This inventory tracks the remaining product/runtime Nushell floor after startup,
welcome sequencing, helper transport, generated-state repair, and Zellij handoff
moved to Rust.

## Current Shape

The tracked product/runtime Nushell surface is `132` lines across `2` `.nu`
files:

```text
nushell/config/config.nu
nushell/config/stack_prompt_guard.nu
```

The old product-side Nushell owner set is gone. Root help, command metadata,
generated externs, runtime materialization, Yazi generation, Zellij generation,
terminal generation, Helix generation, public doctor, config edit, import,
workspace commands, update commands, startup, welcome sequencing, and helper
transport are Rust-owned or outside the remaining Nushell floor.

## Remaining Owner Decisions

| File | Current decision | Why |
| --- | --- | --- |
| `nushell/config/config.nu` | Retain | User shell config source; it wires generated initializers and externs into Nushell |
| `nushell/config/stack_prompt_guard.nu` | Retain | Interactive prompt guard logic is shell-local and not a product control-plane owner |

Release metadata is no longer part of the Nushell owner set. The repo root
`release_metadata.toml` feeds the packaged `runtime_identity.json.version`.

No public `yzx/` Nushell module remains. `yzx menu` is Rust-owned and still uses
`fzf` as the interactive selection process.

## Resolved Follow-Up Decisions

- `yzx enter` owns interactive startup, welcome sequencing, session snapshots,
  runtime env recomputation, materialization, and final Zellij argv in Rust
- The deleted Nu bridge files are not launch fallbacks and should not be
  recreated around Rust helpers

Future follow-ups should use the same delete-first bar: a migration only counts
when the Nushell owner shrinks end-to-end or a deterministic invariant moves to
a clearer existing owner.

## Non-Goals

- Rewriting the Nushell user shell config into Rust
- Reintroducing deleted Nushell command registries or materialization wrappers
- Adding Rust helpers that leave the same Nushell owner in place

## Verification

- `yzx_repo_validator validate-nushell-syntax`
- `yzx_repo_validator validate-contracts`
- `yzx dev test` when a follow-up changes command routing, generated-state
  behavior, or shell/runtime integration
