# Flake Interface Contract

## Summary

Yazelix should add a thin top-level `flake.nix` that exposes a small, honest set of outputs around the existing runtime model.

The flake is a discovery and bootstrap surface. It must not become a second environment definition, and it must not treat the flake source path as the long-term runtime root.

## Goals

- provide a real one-command install front door
- expose a package-ready runtime artifact
- keep `devenv.nix` and the existing runtime assets as the source of truth
- avoid reviving the old `~/.config/yazelix` clone assumption

## Phase 1 Required Outputs

### `apps.<system>.install`

The canonical install front door:

```bash
nix run github:luccahuguet/yazelix#install
```

Responsibilities:

1. verify that Nix is available
2. install or refresh the Yazelix-pinned `devenv` CLI if needed
3. materialize the persistent Yazelix runtime, including the runtime-local `nu`
4. initialize `~/.config/yazelix/user_configs/` if missing
5. install or refresh the stable `yzx` executable in `~/.local/bin/`
6. print the next step, usually `yzx launch`

Bootstrap-tool ownership in phase 1:
- **Host prerequisite**: Nix with flakes enabled
- **Installer-managed**: the pinned `devenv` CLI, the packaged runtime, and the runtime-local `nu` used by installed POSIX entrypoints
- **Not installer-managed**: a separate host/global Nushell install for the user's everyday shell outside Yazelix

### `packages.<system>.runtime`

A package output containing the shipped Yazelix runtime tree.

It should contain the runtime-owned assets only:

- `nushell/scripts/`
- `shells/`
- `configs/`
- `rust_plugins/` runtime artifacts when needed
- shipped templates and docs needed by the installed runtime

It must not contain or own user config.

### `packages.<system>.default`

Alias to `packages.<system>.runtime`.

This keeps the flake predictable for users and tools without requiring them to guess the package name.

## Phase 1 Optional Outputs

These are useful but should not block the first shipped front door:

- `homeManagerModules.default`
  - top-level re-export of the existing Home Manager module
- `apps.<system>.launch`
  - a direct launcher app if it falls out naturally from the runtime package
- `checks.<system>.*`
  - focused flake-evaluation and installer smoke checks

## Installed Runtime Layout

Phase 1 should use a stable user-local runtime pointer plus an immutable packaged runtime target.

Stable user-facing paths:

- config root:
  - `~/.config/yazelix`
- state root:
  - `~/.local/share/yazelix`
- runtime pointer:
  - `~/.local/share/yazelix/runtime/current`
- stable command:
  - `~/.local/bin/yzx`

Model:

- `packages.runtime` produces the immutable runtime tree
- the installer materializes that runtime by pointing `~/.local/share/yazelix/runtime/current` at the packaged runtime target
- `~/.local/bin/yzx` points at `~/.local/share/yazelix/runtime/current/shells/posix/yzx_cli.sh`
- installed POSIX entrypoints prefer `~/.local/share/yazelix/runtime/current/bin/nu` over host `nu`
- desktop entry assets should resolve the same stable runtime pointer, not a clone path and not the transient bootstrap flake path
- phase 1 should keep desktop-entry installation explicit via `yzx desktop install`, not as an automatic side effect of `nix run ...#install`

This keeps the user-facing paths stable while allowing the underlying packaged runtime to change on reinstall or update.

## Important Constraint

The flake source path used during `nix run` is bootstrap-only.

It is acceptable for the installer app to run from that path. It is not acceptable for the installed `yzx` command, desktop entry, or editor integrations to keep pointing at that path afterward.

## Update Behavior

Phase 1 update story:

- rerun the canonical installer command
- installer refreshes the runtime pointer and the stable `yzx` entrypoint

That is enough for the first shipped flake surface.

Non-goals for phase 1:

- preserving clone-oriented `yzx update repo` semantics for packaged installs
- inventing a second runtime definition outside `devenv.nix`
- automatically installing a separate host/global Nushell as part of bootstrap

## Home Manager Relationship

Home Manager should remain a supported integration path, but it should not block the first flake install surface.

If re-exporting the existing module at top level is cheap and clean, do it. If not, land the installer and runtime package first and re-export the module as a follow-on slice.

## Acceptance Cases

1. The top-level flake has a minimal, explicit output set instead of a vague “maybe devShell, maybe package, maybe app” surface.
2. The install app targets a persistent installed runtime, not the flake source tree.
3. The stable `yzx` command and desktop entry resolve through `~/.local/share/yazelix/runtime/current`.
4. User config remains under `~/.config/yazelix/user_configs/`.
5. Desktop-entry installation remains an explicit opt-in step instead of an automatic install side effect.
6. The flake stays thin enough that `devenv.nix` remains the real runtime source of truth.
7. The onboarding contract clearly distinguishes installer-managed bootstrap tools from host prerequisites.

## Verification

- manual review against [one_command_install_ux.md](./one_command_install_ux.md)
- manual review against [runtime_root_contract.md](./runtime_root_contract.md)
- future flake-eval checks for required outputs
- CI check: `nu nushell/scripts/dev/validate_flake_install.nu`

## Traceability

- Bead: `yazelix-2ex.1.4.2`
- Bead: `yazelix-2ex.1.4.2.1`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu` (structure and required fields check)
