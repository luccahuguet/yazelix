# Runtime Shell Floor Contract

## Summary

The product/runtime Nushell floor is the measured allowlist below:

| Family | Files | LOC |
| --- | ---: | ---: |
| User Nushell config | `3` | `180` |

Total tracked product/runtime Nu: `180` LOC across `3` files.

## Scope

In scope:

- the runtime-side Nushell files that remain part of the packaged runtime

Out of scope:

- Rust-owned startup, launch, desktop, restart, popup, generated-state repair,
  welcome sequencing, helper transport, and maintainer command orchestration

## Behavior

### Runtime Nushell Floor

The remaining runtime-side Nushell files are:

- `nushell/config/config.nu`
- `nushell/config/rtk_wrappers.nu`
- `nushell/config/stack_prompt_guard.nu`

These files are current-shell Nushell UX files. `rtk_wrappers.nu` defines native
Nushell commands that invoke the profile-owned external RTK binary; it is not an
RTK executable, plugin, bridge, or alternate install owner. Raw verification
commands remain available through `^rtk proxy -- command ...`; the caret selects
the external RTK binary, not a direct bypass around it.
Release version metadata is Rust/Nix-owned: maintainers edit root
`release_metadata.toml`, and packaged runtimes expose the same version through
`runtime_identity.json.version`.

### Rust Startup Owner

Interactive startup is Rust-owned by `yzx enter`. It owns welcome sequencing,
runtime materialization, launch session snapshots, runtime env recomputation,
startup handoff capture, default cwd/layout/default-shell resolution, and final
Zellij argv construction.

No runtime-side Nushell file is a startup fallback.

## Non-Goals

- broad Rust wrapper insertions that leave the same shell/process boundary in
  place
- reviving deleted config/logging/helper files in Nushell

## Acceptance Cases

1. No runtime-side Nushell wrapper file owns startup, helper transport, or
   generated-state repair
2. Normal launch/setup does not call a product shellhook or runtime
   `environment.nu` script
3. The runtime helper allowlist is reviewed directly against the retained shell
   floor
4. Interactive `cargo`, `git`, and agent commands route through the
   profile-owned RTK binary without creating a second RTK executable

## Verification

- `yzx_repo_validator validate-contracts`
- `yzx_repo_validator validate-nushell-syntax`
