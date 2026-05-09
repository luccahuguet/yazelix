# First-Party Zellij Plugin Wasm Ownership

## Summary

Yazelix currently consumes two first-party Zellij plugin wasm artifacts by tracking copied binaries in `configs/zellij/plugins/`:

- `yazelix_pane_orchestrator.wasm`, sourced from `yazelix-zellij-pane-orchestrator`
- `yzpp.wasm`, sourced from `yazelix-zellij-popup`

This copied-artifact model is acceptable only as an interim maintainer workflow. The target architecture is package ownership: each child repository builds and publishes its own wasm package, and Yazelix consumes those packages through locked flake inputs.

## Why

The copied-artifact model creates a three-way consistency problem between child source, tracked binary, and generated runtime state. Sync stamps and validators reduce the risk, but they are guardrails around manual artifact ownership.

Locked child packages make the package lock the provenance source. The child repository owns source, build target, artifact name, standalone examples, and package checks. Yazelix owns integration, aliases, generated Zellij config, runtime copy into user state, and live session validation.

## Scope

- First-party Zellij plugin artifacts used by normal Yazelix sessions
- `yazelix-zellij-pane-orchestrator`
- `yazelix-zellij-popup`
- Main repo runtime packaging and generated Zellij config
- Local maintainer override paths for active plugin development

Out of scope:

- `zjstatus.wasm`, which is third-party vendoring through the main repo's `zjstatus` lock and update workflow
- `yazelix-zellij-bar`, which is a child package consumed as a runtime package rather than copied into the integrated plugin directory

## Contract Items

#### FPW-001
- Type: ownership
- Status: planning
- Owner: child repository package boundary
- Statement: First-party Zellij plugin source repositories should own their
  wasm package outputs. The Yazelix main repo should consume those outputs
  through locked package inputs instead of treating copied binaries as source
  artifacts
- Verification: validator `nix flake metadata`; manual package inspection

#### FPW-002
- Type: boundary
- Status: planning
- Owner: Yazelix runtime package
- Statement: Yazelix may copy first-party plugin wasm files from locked child
  packages into the packaged runtime tree, but the source revision and package
  build must be represented by the lock file or equivalent package lock
- Verification: automated `nix build .#runtime`

#### FPW-003
- Type: invariant
- Status: planning
- Owner: local maintainer workflow
- Statement: Active plugin development must keep a local override path so a
  maintainer can test an adjacent checkout without committing copied wasm
  drift as the durable provenance model
- Verification: manual local flake override or path input smoke test

#### FPW-004
- Type: non_goal
- Status: planning
- Owner: copied-wasm transition
- Statement: Interim sync stamps for copied first-party wasm artifacts are not
  the final architecture. They exist to make current copied artifacts auditable
  until locked child package consumption replaces them
- Verification: validator `yzx_repo_validator validate-pane-orchestrator-sync`

## Target Architecture

Each first-party plugin child repository should provide a package with a stable wasm path:

- `yazelix-zellij-pane-orchestrator`: `share/yazelix_zellij_pane_orchestrator/yazelix_pane_orchestrator.wasm` or an equivalent documented package path
- `yazelix-zellij-popup`: `share/yazelix_zellij_popup/yzpp.wasm`

The main flake should consume those repositories as inputs and pass their package outputs into the runtime package builder. The runtime package builder should materialize `configs/zellij/plugins/` from package outputs instead of relying on tracked copied binaries for first-party plugins.

The regular package path should be lock-driven:

```nix
inputs.yazelixZellijPopup.url = "github:luccahuguet/yazelix-zellij-popup";
```

Local development should stay override-driven:

```bash
nix build .#runtime --override-input yazelixZellijPopup ../yazelix-zellij-popup
```

Pane-orchestrator migration has one prerequisite: the child repository must publish a Nix package or equivalent package output. `yazelix-zellij-popup` already publishes a `yzpp` package, so it can migrate first.

## Transition

1. Keep copied wasm guardrails for current releases
2. Add a package output to `yazelix-zellij-pane-orchestrator`
3. Add main flake inputs for `yazelix-zellij-pane-orchestrator` and `yazelix-zellij-popup`
4. Teach the runtime package builder to place first-party plugin wasm files from package outputs into `configs/zellij/plugins/`
5. Update workspace asset validation to validate packaged plugin files instead of tracked copied first-party binaries
6. Delete tracked first-party wasm binaries and temporary sync stamps once package consumption is the only supported path

Current copied-artifact guardrails are `yzx dev build_pane_orchestrator --sync` plus `yzx_repo_validator validate-pane-orchestrator-sync` for the pane orchestrator, and `yzx dev sync_yzpp_wasm` plus `yzx_repo_validator validate-yzpp-sync` for `yzpp`

## Acceptance Cases

1. A maintainer can tell which source revision produced packaged first-party plugin wasm artifacts from `flake.lock`
2. Regular Yazelix package builds do not depend on adjacent mutable plugin checkouts
3. Local plugin development uses explicit flake overrides or path inputs
4. `zjstatus.wasm` stays on its separate third-party vendoring path
5. Sync stamps remain only as interim copied-artifact guardrails

## Verification

- `nix flake metadata`
- `nix build .#runtime`
- `yzx_repo_validator validate-workspace-session-contract`
- `yzx_repo_validator validate-pane-orchestrator-sync` while copied pane-orchestrator wasm remains tracked

## Traceability

- Defended by: `docs/contracts/pane_orchestrator_component.md`
- Defended by: `docs/contracts/floating_tui_panes.md`
- Defended by: `docs/contracts/standalone_yazelix_zellij_bar_distribution.md`
- Defended by: `packaging/mk_runtime_tree.nix`
