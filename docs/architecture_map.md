# Yazelix Architecture Map

Yazelix is easiest to understand as five maintained subsystem families.

This is a maintainer-facing map, not a marketing split. The point is to answer two practical questions:

1. Which part of the repo owns a behavior?
2. Where should deletion-first simplification pay off first?

See [Subsystem Code Inventory](./subsystem_code_inventory.md) for the current
delete-first inventory, the remaining Nushell deletion lanes, and the surfaces
that are still honest Nushell fits.

See [Documentation Architecture](./documentation_architecture.md) for the
delete-first boundary between canonical contracts, user docs, maintainer docs,
history, and Beads-owned planning state.

## Subsystem Families

| Subsystem family | What it owns | Main paths | Main source of truth |
| --- | --- | --- | --- |
| Runtime control plane and command surface | Config parsing, runtime/bootstrap behavior, generated-state repair, and the `yzx` command surface | `nushell/scripts/core`, `nushell/scripts/setup`, `nushell/scripts/utils`, `nushell/scripts/yzx` | Runtime/config/state contracts plus the surviving `yzx` command semantics |
| Workspace session orchestration | Live Zellij/Yazi/editor session behavior: panes, tabs, sidebar identity, reveal/open flows, popup flows, and layout-family transitions | `nushell/scripts/integrations`, `nushell/scripts/zellij_wrappers`, `rust_plugins/` | Live Zellij session truth, pane-orchestrator contracts, and workspace/session contracts |
| Distribution and host integration | How Yazelix is packaged, launched, and adapted into external owners such as Home Manager, shells, terminals, desktop integration, and profile-owned installs | `home_manager`, `packaging`, `shells`, `flake.nix`, `yazelix_package.nix`, `yazelix_runtime_package.nix` | The packaged runtime shape and explicit integration contracts |
| Shipped runtime data and assets | The tracked data the runtime consumes directly: layouts, themes, plugins, templates, release metadata, TOML tooling support, cursor presets, and visual assets | `configs`, `config_metadata`, `user_configs`, `assets`, `nushell/config`, `tombi.toml`, `yazelix_default.toml`, `yazelix_cursors_default.toml`, `docs/upgrade_notes.toml` | Version-controlled shipped files |
| Maintainer workflow and validation | The non-user-facing machinery that keeps the other four coherent: tests, validators, release/update workflow, CI, and maintainer tooling | `nushell/scripts/dev`, `.github`, `maintainer_shell.nix`, `.nu-lint.toml` | Beads, contracts, CI policy, and maintainer command surfaces |

## How They Fit Together

The current repo shape is best read in this order:

1. Shipped runtime data and assets define the tracked files the runtime and workspace consume.
2. The runtime control plane turns config plus shipped files into actual behavior, generated state, and user-facing commands.
3. The workspace session subsystem owns the live terminal-IDE behavior once a Yazelix session is running.
4. Distribution and host integration expose that runtime/workspace behavior through package, install, shell, terminal, and Home Manager entrypoints.
5. The maintainer workflow guards the contracts of the other four so they do not drift.

That means Yazelix is no longer best described as just "workspace plus runtime." The current v16 repo has a real data/config payload, a real maintainer/validation payload, and a small but important distribution layer.

## Subsystem Notes

### Runtime Control Plane And Command Surface

This subsystem answers questions like:

- How does `yazelix.toml` become actual runtime behavior?
- Where do generated configs live?
- What does `yzx launch`, `yzx env`, `yzx doctor`, or `yzx update` mean now?
- Which paths are config-owned, runtime-owned, or generated-state-owned?

This is still the single largest shipped logic surface in the repo. If Yazelix is too heavy, this is still the first place to look before blaming Nix glue or Rust plugins.

Related contracts:

- [Current Trimmed Runtime Contract](./contracts/v15_trimmed_runtime_contract.md)
- [Runtime Root Contract](./contracts/runtime_root_contract.md)
- [Rust/Nushell Bridge Contract](./contracts/rust_nushell_bridge_contract.md)
- [Runtime Dependency And Launch Preflight Contract](./contracts/runtime_dependency_preflight_contract.md)

### Workspace Session Orchestration

This subsystem answers questions like:

- Which pane is the managed editor?
- Which pane is the sidebar in the current tab?
- When should reveal/open target the existing editor pane versus create one?
- How should layout-family changes, popup flows, and workspace roots behave?

This is where the Rust plugins matter. They are not "extra integration code"; they are part of the live workspace owner.

Related contracts:

- [Workspace Session Contract](./contracts/workspace_session_contract.md)
- [Backend-Free Workspace Slice](./contracts/backend_free_workspace_slice.md)
- [Cross-Language Runtime Ownership](./contracts/cross_language_runtime_ownership.md)

### Distribution And Host Integration

This subsystem answers questions like:

- What does the flake actually expose?
- What belongs to the packaged runtime versus the host?
- How should Home Manager, install-owner-provided `yzx`, desktop entry installation, and terminal launchers adapt Yazelix without inventing new product semantics?

This layer should stay thin. If it starts owning behavior that belongs to runtime or workspace, that is architecture drift.

Related contracts:

- [Nixpkgs Package Contract](./contracts/nixpkgs_package_contract.md)
- [Runtime Distribution Capability Tiers](./contracts/runtime_distribution_capability_tiers.md)
- [Helix Managed Config Contract](./contracts/helix_managed_config_contract.md)

### Shipped Runtime Data And Assets

This subsystem answers questions like:

- Which tracked files are part of the runtime payload?
- Which layouts, plugins, themes, templates, and metadata are product code rather than generated state?
- Which user-facing defaults are actually shipped with the runtime?

This bucket matters because Yazelix is not just Nushell and Rust. A meaningful part of the product lives in TOML, Lua, GLSL, KDL, shell config, and release metadata.

### Maintainer Workflow And Validation

This subsystem answers questions like:

- Which tests and validators defend the real current contract?
- How are release notes, version bumps, package smoke checks, and issue/bead sync handled?
- What should `nix develop` guarantee for maintainers?

This subsystem exists so the other four can change without silently regressing. It is not user-facing product value, but it is a large real code cost.

## Ownership Rules

Useful modularity in Yazelix means:

- one clear owner per behavior
- one clear source of truth per maintained contract
- fewer hidden path assumptions
- fewer environment-variable side channels
- thinner adapters around external systems
- tests and validators that defend real boundaries instead of trivia

It does not mean:

- splitting the repo just to look cleaner
- introducing wrappers with no ownership benefit
- moving behavior into an integration layer because it is inconvenient elsewhere
- pretending shipped data files are not part of the architecture

## Current Working Model

For current v16 work, the right mental model is:

1. Runtime control plane owns command/runtime semantics.
2. Workspace session orchestration owns live pane/tab/sidebar behavior.
3. Distribution and host integration adapts Yazelix to package/install/launcher owners.
4. Shipped runtime data and assets are a first-class subsystem, not miscellaneous residue.
5. Maintainer workflow and validation keeps the other four honest.

Historical pre-trim planning notes do not belong in canonical contracts. This map should describe the living repo shape, not the old broader Classic-era model.
