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
3. materialize the persistent Yazelix runtime
4. initialize `~/.config/yazelix/user_configs/` if missing
5. install or refresh the stable `yzx` executable in `~/.local/bin/`
6. optionally install or refresh desktop assets
7. print the next step, usually `yzx launch`

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
- desktop entry assets should resolve the same stable runtime pointer, not a clone path and not the transient bootstrap flake path

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

- a separate self-update product surface
- preserving clone-oriented `yzx update repo` semantics for packaged installs
- inventing a second runtime definition outside `devenv.nix`

## Home Manager Relationship

Home Manager should remain a supported integration path, but it should not block the first flake install surface.

If re-exporting the existing module at top level is cheap and clean, do it. If not, land the installer and runtime package first and re-export the module as a follow-on slice.

## Acceptance Cases

1. The top-level flake has a minimal, explicit output set instead of a vague “maybe devShell, maybe package, maybe app” surface.
2. The install app targets a persistent installed runtime, not the flake source tree.
3. The stable `yzx` command and desktop entry resolve through `~/.local/share/yazelix/runtime/current`.
4. User config remains under `~/.config/yazelix/user_configs/`.
5. The flake stays thin enough that `devenv.nix` remains the real runtime source of truth.

## Verification

- manual review against [one_command_install_ux.md](./one_command_install_ux.md)
- manual review against [runtime_root_contract.md](./runtime_root_contract.md)
- future flake-eval checks for required outputs
- future install smoke check proving that `yzx` resolves through the stable runtime pointer

## Traceability

- Bead: `yazelix-2ex.1.4.2`
- Bead: `yazelix-2ex.1.4.2.1`
