# Yazelix Architecture Map

Yazelix is easiest to understand as one product with two main user-facing layers and two supporting subsystems.

The two main product layers are:

1. Workspace
2. Runtime

The two supporting subsystems are:

1. Integrations
2. Maintainer Workflow

The important architectural rule is that each subsystem should have a clear owner and a clear source of truth. Modularity in Yazelix does not mean adding abstraction for its own sake. It means removing hidden ownership, stale path assumptions, and accidental cross-layer state.

## Subsystems

| Subsystem | Purpose | Examples | Main source of truth |
| --- | --- | --- | --- |
| Workspace | The terminal IDE experience users interact with directly | Zellij layouts, Yazi/editor flow, keybindings, managed panes, tab naming, workspace roots | Session and workspace state, pane orchestrator, workspace-focused commands |
| Runtime | The environment and control plane that makes the workspace reproducible | `devenv`, packs, `yazelix.toml`, `yzx refresh`, `yzx restart`, `yzx run` | `yazelix.toml`, `yazelix_default.toml`, generated runtime config, `devenv.nix` |
| Integrations | Adapters between Yazelix and external systems | Home Manager, desktop entry, shell hooks, terminal-specific launchers, CI entrypoints | The supported contract of the subsystem being integrated, not ad hoc host assumptions |
| Maintainer Workflow | The machinery that keeps the product evolving safely | Beads, CI, validators, release/version/update workflow, future specs | Beads graph, CI workflows, documented contracts, maintainer commands |

## The Two Main Layers

### Workspace

The workspace layer is what users usually mean when they talk about "using Yazelix":

- Zellij as the workspace shell
- Yazi as the file-manager/sidebar surface
- Helix or another editor as the editing surface
- keybindings, tab naming, pane creation, and workspace-root behavior

This layer should answer questions like:

- What does a tab represent?
- Where should a new pane open?
- Which pane is the managed editor?
- When should the sidebar take focus?
- How should Yazi, tab naming, and workspace roots stay in sync?

### Runtime

The runtime layer is what makes Yazelix reproducible and configurable:

- package selection through packs
- shell and editor configuration in `yazelix.toml`
- `devenv` evaluation and environment build logic
- generated configs and refresh/restart flows

This layer should answer questions like:

- Which tools are installed?
- Which shell/editor/terminal is configured?
- Where do generated configs live?
- How does `yzx refresh` or `yzx restart` rebuild and switch the runtime safely?

## Supporting Subsystems

### Integrations

Integrations translate Yazelix into other environments and entrypoints:

- Home Manager
- desktop launchers
- shell hooks
- terminal-specific launch wrappers
- CI-specific setup and smoke checks

They are important, but they should stay thin. An integration should adapt a well-defined Yazelix contract to an external system. It should not invent new sources of truth.

Examples:

- Home Manager should own declarative config and explicit desktop integration, not assume a repo checkout.
- Desktop launchers should use the supported launch contract, not depend on shell-specific quirks.
- CI should test supported entrypoints, not rely on maintainer-only machine state.

### Maintainer Workflow

This subsystem exists so the other three can evolve safely:

- Beads issue graph
- GitHub issue contract
- CI
- regression tests
- version/update/release workflows
- future spec-driven development

This is not user-facing product value by itself, but without it the product drifts quickly.

## Ownership Rules

Each area should have one clear owner.

### Runtime and Source Ownership

- The runtime/source location is one concern.
- User config is another concern.
- Generated state is another concern.

These should not be conflated.

Current direction:

- code should resolve the Yazelix root through canonical helpers such as `get_yazelix_dir`
- tests and helpers should not assume `~/.config/yazelix` is a repo checkout
- package-ready work should keep clarifying what is shipped, user-owned, and generated

See [Config Surface And Launch Profile Contract](./specs/config_surface_and_launch_profile_contract.md) for the concrete runtime ownership model.
See [Runtime Root Contract](./specs/runtime_root_contract.md) for the concrete split between config-owned paths, shipped runtime assets, and generated state.
See [Backend Capability Contract](./specs/backend_capability_contract.md) for the concrete capability buckets Yazelix expects from its runtime/environment layer before any alternative backend evaluation.
See [Runtime Dependency And Launch Preflight Contract](./specs/runtime_dependency_preflight_contract.md) for the narrower user-facing dependency story that separates fast launch blockers from heavier doctor and install-smoke diagnostics.
See [yzx Command Surface Backend Coupling](./specs/yzx_command_surface_backend_coupling.md) for the command-family audit that separates backend-required control-plane commands from workspace/config UX, runtime/distribution surfaces, and mixed seams.
See [One-Command Install UX](./specs/one_command_install_ux.md) for the planned first-class install front door and why it should be a thin `nix run` installer surface instead of a hosted shell script.
See [Flake Interface Contract](./specs/flake_interface_contract.md) for the exact phase-1 flake outputs and the stable installed-runtime layout they should target.
See [Nixpkgs Package Contract](./specs/nixpkgs_package_contract.md) for the simpler store-backed package shape Yazelix should target before preparing the upstream nixpkgs submission.
See [Helix Managed Config Contract](./specs/helix_managed_config_contract.md) for the phase-1 boundary that makes Helix reveal integration self-contained without taking ownership of the user's full Helix config tree.

### Workspace and Session Ownership

The workspace model should have a narrow contract between Nushell and the pane orchestrator.

See [Workspace Session Contract](./workspace_session_contract.md) for the current boundary in more concrete terms.

That contract should cover:

- tab workspace root
- managed editor identity
- sidebar identity and state
- tab naming
- pane creation cwd
- cross-tab isolation

If these rules are implicit, workspace bugs reappear under different symptoms.

### Integration Ownership

Integrations should be adapters around the workspace or runtime contracts.

They should not own product semantics that belong elsewhere.

Examples:

- `yazelix.toml` should express runtime/user config, not desktop-entry metadata
- Home Manager can manage both config and desktop integration, but those responsibilities should be explicit
- launcher compatibility fixes should live in launcher integration logic, not leak back into unrelated workspace code

### Maintainer Ownership

Maintainer tooling should guard real contracts:

- path model
- issue/bead lifecycle sync
- Home Manager evaluation
- workspace invariants
- version drift

Tests and automation should defend meaningful boundaries, not superficial command discovery.

## What "More Modular" Means For Yazelix

Useful modularity for Yazelix means:

- fewer hidden sources of truth
- fewer environment-variable side channels
- fewer path assumptions
- clearer ownership of workspace state
- thinner integration adapters
- better contract tests at subsystem boundaries

It does not mean:

- splitting the repo just to look cleaner
- rewriting everything in Rust immediately
- introducing wrappers with no clear ownership benefit
- creating configuration layers that duplicate each other

## Current Working Model

For now, the right mental model is:

1. Yazelix Workspace is the terminal IDE experience.
2. Yazelix Runtime is the environment and control plane.
3. Integrations connect Yazelix to external systems.
4. Maintainer Workflow keeps the product coherent over time.

Future architecture and spec work should build on this map rather than treating Yazelix as one undifferentiated shell script system.
