# Runtime Shell Floor Contract

## Summary

The product/runtime Nushell floor is the measured allowlist below:

| Family | Files | LOC |
| --- | ---: | ---: |
| User Nushell config | `2` | `132` |

Total tracked product/runtime Nu: `132` LOC across `2` files.

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
- `nushell/config/stack_prompt_guard.nu`

`config.nu` and `stack_prompt_guard.nu` are current-shell Nushell UX files.
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

## Verification

- `yzx_repo_validator validate-contracts`
- `yzx_repo_validator validate-nushell-syntax`
