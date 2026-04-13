# Nixpkgs Package Contract

## Summary

Yazelix should target a simple nixpkgs package shape: one `yazelix` package that ships the immutable runtime tree plus the `yzx` command, and runs directly from the store path.

The nixpkgs package should not re-enact the flake installer. It should not mutate the user's home directory at install time, install a separate legacy environment-manager binary into the user's profile, or require a `runtime/current` indirection just to function.

## Why

Older Yazelix releases exposed a flake installer that materialized persistent user-local state, while a nixpkgs package is an immutable store package that should work directly when added to a profile or system configuration

If the nixpkgs package tries to behave like that older installer model, it creates unnecessary duplication, more wrapper logic, and a more fragile review surface. The package contract should delete that duplication instead of preserving it.

## Delete-First Decisions

To keep the nixpkgs submission small, honest, and maintainable:

1. The nixpkgs package should run directly from its store path.
2. It should not require `~/.local/share/yazelix/runtime/current` to exist.
3. It should not auto-install shell hooks or desktop entries.
4. It should not auto-create `~/.local/bin/yzx`.
5. It should not install a second legacy environment-manager binary into the user's profile as a side effect.
6. It should not introduce a second runtime definition outside the existing runtime package/source tree.

## Package Identity

First submission target:

- package name: `yazelix`
- package kind: runtime/CLI package
- primary entrypoint: `bin/yzx`

The package should be consumable through normal nixpkgs surfaces such as:

- `nix profile install nixpkgs#yazelix`
- `environment.systemPackages = [ pkgs.yazelix ];`
- Home Manager package installation

The package should not require any separate installer surface after installation.

## Installed Package Contents

The package should ship the immutable runtime assets Yazelix executes directly:

- `assets/`
- `config_metadata/`
- `configs/`
- `docs/` when runtime scripts depend on shipped docs/help text
- `nushell/`
- `rust_plugins/`
- `shells/`
- shipped templates such as:
  - `yazelix_default.toml`
- `bin/yzx`
- runtime-local `bin/nu`

It must not own or embed user config/state paths.

## Runtime Model

For nixpkgs packaging, the package path itself is the runtime root.

That means:

- `YAZELIX_RUNTIME_DIR` resolves to the packaged store path
- `yzx` runs directly from the packaged runtime
- runtime-owned scripts and assets are resolved relative to that packaged runtime

The package should still respect the existing split-root contract:

- config root: `~/.config/yazelix`
- user config root: `~/.config/yazelix/user_configs`
- state root: `~/.local/share/yazelix`

The package must never treat the runtime root as the user config root.

## Dependencies

The nixpkgs package should depend on the tools needed to bootstrap and run Yazelix itself, not every optional pack-managed tool.

Expected package-level runtime dependencies:

- Nushell
- other small direct bootstrap/runtime tools that Yazelix invokes outside its own runtime scripts

Non-goal for first submission:

- bundling every optional terminal, editor, or pack-managed dependency into the package closure

If `yzx launch` needs a terminal emulator that is not available, it should fail clearly with an actionable error. That is preferable to silently widening the package scope.

## Entry Point Expectations

`bin/yzx` is the canonical user-facing surface for the nixpkgs package.

For the first nixpkgs submission:

- `yzx --version-short` should work directly after installation
- `yzx doctor` should work directly after installation
- `yzx launch` should work when the necessary host/runtime conditions are present
- `yzx env` should work without requiring a cloned repo

The package should not require:

- a source checkout
- a flake-install-generated `~/.local/bin/yzx`
- a pre-existing `runtime/current` symlink

## Desktop Integration

Desktop integration should stay explicit for the first nixpkgs submission.

Phase-1 nixpkgs package rule:

- do not make desktop entry installation a package-install side effect
- continue to use `yzx desktop install` for user-local desktop integration

Reason:

- this matches the current product contract
- it avoids introducing package-manager-specific desktop state semantics during the first submission
- it keeps stale-desktop-entry repair in one existing surface instead of creating a second one

Shipping a nixpkgs desktop item can be evaluated later if it simplifies the model without duplicating the current user-local desktop contract.

## Linux and macOS Scope

First submission target:

- Linux-first

Linux should be the platform whose launch, runtime, and desktop expectations define the initial package contract.

For macOS:

- keep the implications explicit during contract review
- do not let macOS-specific packaging uncertainty expand the first nixpkgs submission unnecessarily

If the package can build or partially work on Darwin without extra complexity, that is good. But the first nixpkgs submission should not be blocked on promising a fully-polished macOS package story.

## Update Story

For nixpkgs users, updates come from the package manager:

- `nix profile upgrade`
- channel/flake input updates
- system/Home Manager rebuilds

The package contract should not depend on any separate installer as the update mechanism for nixpkgs users.

## Non-goals

- restoring a separate flake installer
- replacing Home Manager
- automatically installing desktop entries
- automatically installing shell hooks
- bundling every optional pack-managed dependency
- solving the entire future nixpkgs review thread up front
- redesigning Yazelix around packaging-only abstractions

## Acceptance Cases

1. The package has one clear identity: an immutable store-backed Yazelix runtime plus `yzx`.
2. The package does not need `runtime/current` or `~/.local/bin/yzx` to function.
3. The package does not mutate the user's home directory during install.
4. User config and generated state remain outside the package.
5. The package includes the runtime-local `nu` and the direct bootstrap/runtime dependencies needed to invoke Yazelix itself.
6. Desktop integration remains explicit via `yzx desktop install` instead of becoming implicit package behavior.
7. The remaining work to upstream the package is mostly translation and review, not product-boundary redesign.

## Verification

- manual review against [flake_interface_contract.md](./flake_interface_contract.md)
- manual review against [runtime_root_contract.md](./runtime_root_contract.md)
- future packaging smoke checks that build the package and run:
  - `yzx --version-short`
  - `yzx doctor`
  - `yzx env --no-shell`
- CI/spec check: `nu nushell/scripts/dev/validate_specs.nu`

## Traceability

- Bead: `yazelix-2ex.1.4.3.1`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
