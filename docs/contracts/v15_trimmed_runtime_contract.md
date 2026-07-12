# Current Trimmed Runtime Contract

## Summary

Maintainers wrote this contract for the v15 trim and keep it as the runtime boundary for the supported Yazelix line.

Yazelix keeps the v15 slimmed-down reboot product boundary. Rust owns the control plane and integration logic, the Rust pane orchestrator owns live workspace and session state, and the repository keeps the remaining Nushell code in shipped shell configuration.

The product surface centers on:

- a fixed packaged runtime
- explicit install/update owners
- managed `./` configuration
- workspace/session commands
- configured fast popup/menu/config UI panes through `yzpp`
- generated-state repair instead of backend/profile orchestration

The trimmed runtime excludes:

- `yazelix_packs.toml`
- dynamic pack graphs
- the old runtime-local `devenv` layer
- cached launch-profile reuse
- a broad or permanent config schema migration engine beyond the bounded final-Classic migration
- a generic Yazelix-owned runtime updater

## Why

Several older contracts still describe the pre-trim planning space where Yazelix owned more runtime/package-manager behavior. That history is useful, but it is not the current branch contract anymore.

This file exists so current docs and current contracts can point at one authoritative post-trim description instead of reusing planning notes that were written before the deletions actually landed.

## Scope

- define the current user-facing runtime/config/update boundary
- define what still belongs to the normal product surface
- define what moved to maintainer-only or historical territory
- give other docs a safe current contract to point at

## Current Contract

### Runtime Surface

- The normal product runtime is the packaged `yazelix` runtime.
- That runtime ships a fixed toolset rather than a user-managed package graph, but interactive shells only export the curated user-facing tool surface instead of the full helper closure.
- Runtime tool versions come from the locked `nixpkgs` input, and the first-party flake intentionally tracks `github:NixOS/nixpkgs/nixpkgs-unstable`. Yazelix is an application/runtime distribution for fast-moving terminal and TUI integrations, so fresher package availability is more important than the extra NixOS system-gating from `nixos-unstable`.
- Maintainer update pins record the Nix helper version and the Nixpkgs-provided Nushell version, so upstream Nushell releases only become runtime bumps after they land in the locked Nixpkgs input or Yazelix deliberately changes that ownership model.
- The packaged runtime has one terminal: Mars, exposed only through the complete `yazelix` package and app. Home Manager installs that complete package by default.
- Non-Mars terminals are supported through `yzx enter`. Users who prefer Ghostty, Rio, WezTerm, Kitty, Foot, Ratty, Alacritty, or another capable emulator should configure that terminal to run `yzx enter`; that terminal's native config remains user-owned.
- The Mars path uses Yazelix-pinned Zellij and Yazi forks so Yazi image previews can use Kitty graphics through Zellij.
- The Zellij and Yazi forks are temporary product integration forks. Once upstream Zellij supports the required Kitty graphics path directly enough for Yazelix to return to upstream packages, those forks should be dropped from the default runtime and archived.
- The runtime does not ship a runtime-local `devenv`.

### Config Surface

- The canonical user config surface is `~/.config/yazelix/`.
- The canonical semantic settings file is sparse `config.toml`; absent fields inherit the packaged `config_default.toml` values.
- The canonical cursor registry is `cursors.toml`, owned separately from the semantic root.
- Managed override directories such as Zellij, Yazi, Helix, and shell user hooks remain part of that user-owned config surface.
- The current trimmed branch does not have a `yazelix_packs.toml` sidecar and does not expose a first-class pack graph.
- Legacy or removed config fields fail fast instead of degrading silently.
- Writable released `settings.jsonc` roots migrate once, backup-first, to the Nova-shaped `config.toml` contract during the final Classic observation release. Coexistence, read-only ownership, collisions, and unsupported values fail before Yazelix chooses or rewrites an owner.
- Old mutable `yazelix.toml`, `user_configs/` sidecars, and embedded cursor settings are unsupported legacy inputs; they hard-error with actionable diagnostics instead of being rewritten automatically. Removed terminal sidecars are ignored by the current runtime and reported as legacy adjacency when reset/config-maintenance commands inspect the config root.

### Generated State

- Generated Zellij/Yazi configs, shell initializers, logs, and repair hashes live under `~/.local/share/yazelix`.
- Those files are derived artifacts, not canonical handwritten config.
- Generated-state repair is an internal runtime responsibility surfaced through startup preflight, `yzx doctor`, and maintainer canaries rather than through a public refresh command.
- Generated-state repair does not rebuild or reuse a cached backend launch profile.

### Update And Distribution Ownership

- Users choose one explicit update owner per install.
  - `yzx update upstream` for default-profile installs of `#yazelix`
  - `yzx update home_manager` for Home Manager installs
- The flake no longer exposes `#install`.
- The product no longer promises a generic in-app runtime updater that owns every install mode.

### Maintainer Boundary

- `maintainer_shell.nix` defines the repo development shell for maintainer workflows.
- That maintainer shell is not the normal user runtime contract.
- Maintainer-only commands may still touch flake inputs, repo profiling flows, or release automation.
- Those maintainer semantics should not leak back into user-facing runtime docs.

## Non-goals

- reintroducing dynamic pack management on the current trimmed branch
- treating cached launch-profile reuse as a current product guarantee
- restoring the old runtime-local `devenv` layer as part of the normal shipped runtime
- restoring a broad config schema migration engine as a normal trimmed-line product surface
- pretending the compatibility installer is the canonical everyday product flow
- treating v15.0 as the Rust-forward release

## Acceptance Cases

1. A current user can understand the product without learning about `yazelix_packs.toml`, launch-profile reuse, runtime-local `devenv`, or a permanent config schema migration engine.
2. Current docs explain generated-state repair through startup and `yzx doctor` rather than through a public refresh command.
3. Current docs explain update ownership through explicit owner commands rather than a generic runtime updater.
4. Current docs distinguish the normal packaged runtime from maintainer-only `nix develop` workflows.
5. Maintainers describe v15.0 as the trimmed reboot that set the narrower boundary and v16 as the first Rust-forward line that carried the contract forward in release history.

## Verification

- `cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_maintainer --bin yzx_repo_validator -- validate-installed-runtime-contract`
- `cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_maintainer --bin yzx_repo_validator -- validate-flake-profile-install all`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_control_workspace_surface`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core runtime_materialization`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_core_classic_nova_root_translation`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_maintainer`

## Traceability
- Defended by: `cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_maintainer --bin yzx_repo_validator -- validate-installed-runtime-contract`
- Defended by: `cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_maintainer --bin yzx_repo_validator -- validate-flake-profile-install all`
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_control_workspace_surface`
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core runtime_materialization`
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_core_classic_nova_root_translation`
- Defended by: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_maintainer`
