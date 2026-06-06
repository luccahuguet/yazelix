# First-Party Zellij Plugin Wasm Ownership

## Summary

Yazelix consumes two first-party Zellij plugin wasm artifacts from child package outputs:

- `yazelix_pane_orchestrator.wasm`, sourced from `yazelix-zellij-pane-orchestrator`
- `yzpp.wasm`, sourced from `yazelix-zellij-popup`

Each child repository builds and publishes its own wasm package, and Yazelix consumes those packages through flake inputs and explicit local overrides during active development.

## Why

Locked child packages make the package lock the provenance source. The child repository owns source, build target, artifact name, standalone examples, and package checks. Yazelix owns integration, aliases, generated Zellij config, runtime copy into user state, and live session validation.

## Scope

- First-party Zellij plugin artifacts used by normal Yazelix sessions
- `yazelix-zellij-pane-orchestrator`
- `yazelix-zellij-popup`
- Main repo runtime packaging and generated Zellij config
- Local maintainer override paths for active plugin development

Out of scope:

- `zjstatus.wasm`, which is consumed as a third-party package artifact from the main repo's `zjstatus` lock
- `yazelix-zellij-bar`, which is a child package consumed as a runtime package rather than as an integrated plugin artifact

## Contract Items

#### FPW-001
- Type: ownership
- Status: live
- Owner: child repository package boundary
- Statement: First-party Zellij plugin source repositories own their wasm
  package outputs. The Yazelix main repo consumes those outputs through package
  inputs instead of treating copied binaries as source artifacts
- Verification: validator `nix flake metadata`; manual package inspection

#### FPW-002
- Type: boundary
- Status: live
- Owner: Yazelix runtime package
- Statement: Yazelix materializes first-party plugin wasm files from child
  packages into the packaged runtime tree, and the source revision and package
  build are represented by the lock file or equivalent package lock
- Verification: automated `nix build .#runtime`

#### FPW-003
- Type: invariant
- Status: live
- Owner: local maintainer workflow
- Statement: Active plugin development must keep a local override path so a
  maintainer can test an adjacent checkout without committing local path inputs
  or copied wasm drift as the durable provenance model
- Verification: manual local flake override or path input smoke test

#### FPW-004
- Type: non_goal
- Status: live
- Owner: copied-wasm transition
- Statement: The Yazelix main repo does not track first-party plugin wasm
  sync stamps as runtime provenance. Package inputs own first-party plugin wasm
  provenance
- Verification: manual review of `configs/zellij/plugins/`

#### FPW-005
- Type: invariant
- Status: live, transitional validation
- Owner: child release transaction
- Statement: First-party Zellij plugin child packages consumed by Yazelix must
  instantiate on `aarch64-darwin` with `cargoBuildHook` disabled for the
  manual wasm build and an explicit wasm-capable Rust toolchain exported before
  `runHook preBuild`: exported `CARGO`, `RUSTC`, and `PATH`, a
  `rustc --print target-libdir --target wasm32-wasip1` preflight before
  preBuild hooks can run, and a later `--target wasm32-wasip1` Cargo build
- Verification: validator `yzx_repo_validator validate-child-release-transaction`.
  The current main-repo validator inspects child derivation markers as a
  temporary regression guard. The target validation surface is child-declared
  package metadata or child-owned checks that let main validate the wasm
  contract without hardcoding child build recipe strings

## Target Architecture

Each first-party plugin child repository provides a package with a stable wasm path:

- `yazelix-zellij-pane-orchestrator`: `share/yazelix_zellij_pane_orchestrator/yazelix_pane_orchestrator.wasm` or an equivalent documented package path
- `yazelix-zellij-popup`: `share/yazelix_zellij_popup/yzpp.wasm`

The main flake consumes those repositories as inputs and passes their package outputs into the runtime package builder. The runtime package builder materializes `configs/zellij/plugins/` from package outputs instead of relying on tracked copied binaries for first-party plugins.

The regular package path should be lock-driven:

```nix
inputs.yazelixZellijPopup.url = "github:luccahuguet/yazelix-zellij-popup";
inputs.yazelixZellijPaneOrchestrator.url = "github:luccahuguet/yazelix-zellij-pane-orchestrator";
```

Local development should stay override-driven:

```bash
nix build .#runtime --override-input yazelixZellijPopup ../yazelix-zellij-popup
nix build .#runtime --override-input yazelixZellijPaneOrchestrator ../yazelix-zellij-pane-orchestrator
```

## Acceptance Cases

1. A maintainer can tell which source revision produced packaged first-party plugin wasm artifacts from `flake.lock`
2. Regular Yazelix package builds do not depend on adjacent mutable plugin checkouts
3. Local plugin development uses explicit flake overrides or path inputs
4. `zjstatus.wasm` stays on its separate third-party package-artifact path
5. `configs/zellij/plugins/` contains no copied plugin wasm artifacts or sync stamps

## Verification

- `nix flake metadata`
- `nix build .#runtime`
- `yzx_repo_validator validate-workspace-session-contract`
- `yzx_repo_validator validate-child-release-transaction`

## Traceability

- Defended by: `docs/contracts/pane_orchestrator_component.md`
- Defended by: `docs/contracts/floating_tui_panes.md`
- Defended by: `docs/contracts/standalone_yazelix_zellij_bar_distribution.md`
- Defended by: `packaging/mk_runtime_tree.nix`
