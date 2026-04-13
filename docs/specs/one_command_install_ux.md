# One-Command Install UX

> Status: Historical installer-first planning note.
> This document describes the earlier phase where `nix run ...#install` was intended to be the canonical front door and where the installed runtime owned `devenv`.
> Do not treat it as the current branch contract. See [v15_trimmed_runtime_contract.md](./v15_trimmed_runtime_contract.md).

## Summary

Yazelix should adopt a first-class one-command install story built around a thin first-party Nix flake installer app:

```bash
nix run github:luccahuguet/yazelix#install
```

This command should become the canonical front door for new users once implemented.

This spec documents the installer-first phase-1 front door. For the later package-runtime-first target after installer-owned runtime management is demoted, see [Package-Runtime-First User And Maintainer UX](./package_runtime_first_user_and_maintainer_ux.md).

## Why

The old install story was honest but too maintainer-shaped:

- clone the repo somewhere
- install host Nushell separately
- install the Yazelix runtime and its owned bootstrap tools separately
- run `start_yazelix.nu --setup-only`

That works, but it does not feel like a modern front door. The right improvement is not to hide the Nix basis of the product. The right improvement is to package that reality behind one obvious command.

## Decision

Choose a thin first-party `nix run` installer surface as the canonical install UX.

Do not make `curl | sh` the first shipped front door.

## Exact User-Facing Shape

Primary command:

```bash
nix run github:luccahuguet/yazelix#install
```

The installer app should:

1. validate that Nix itself is available
2. materialize or refresh a persistent Yazelix runtime tree that includes Yazelix-owned `devenv` and `nu`
4. initialize `~/.config/yazelix/user_configs/` if missing
5. install the stable `yzx` executable entrypoint into `~/.local/bin/`
6. leave desktop entry installation as an explicit follow-up command (`yzx desktop install`), not an automatic side effect of the installer
7. print the next-step command, normally `yzx launch`

The installer should not mutate the user's global shell toolchain beyond the bootstrap tools Yazelix directly owns. In practice:
- Nix remains the host prerequisite
- the installer owns the runtime-local `devenv` and `nu`
- the installer does not promise to install a separate host/global Nushell for the user's normal shell sessions

## Ownership Model

- Config root:
  - `~/.config/yazelix`
  - user-owned
- Runtime root:
  - persistent installed Yazelix runtime tree
  - not the GitHub flake source path used during `nix run`
- State root:
  - `~/.local/share/yazelix`
  - generated configs, caches, launch-profile state

This is consistent with [runtime_root_contract.md](./runtime_root_contract.md).

## Important Constraint

The `nix run` source tree is not itself the installed runtime.

That flake evaluation path is good for bootstrap, but it is the wrong long-term home for runtime assets because it is ephemeral from the user's point of view and should not become the place that `yzx`, desktop entries, or editor integrations permanently target.

So the installer must materialize or install a persistent runtime and then point the stable entrypoints at that installed runtime.

## Why `nix run` Wins

- It matches Yazelix's real architecture instead of pretending Nix is optional.
- It gives users a copy-paste command without adding a second bootstrap ecosystem.
- It keeps trust and auditability better than a hosted shell installer.
- It aligns naturally with future flake and nixpkgs packaging work.
- It can stay thin: bootstrap only, then hand off to the installed runtime.

## Why `curl | sh` Loses For Now

- It creates a second trust surface immediately.
- It adds release-hosting and installer-maintenance burden before the packaged/runtime path is mature.
- It would still need to hand off to Nix and the owned-runtime dependency story anyway.
- It risks hiding important constraints behind a deceptively simple command.

`curl | sh` can be reconsidered later as a convenience wrapper around an already-solid packaged install path.

## Non-Goals

- making Git clones the canonical user-facing install path
- making the GitHub flake source path the runtime root
- designing a second environment definition separate from `devenv.nix`
- silently taking over the user's global host shell toolchain
- replacing maintainers' clone-based workflows

## Minimum Follow-On Implementation Work

1. add a thin top-level `flake.nix` for discovery and install entrypoints
2. expose an `install` app that performs bootstrap-only logic
3. define the persistent installed runtime layout
4. install the stable `yzx` executable and desktop assets from that runtime
5. make update behavior work without requiring a live Git clone
6. rewrite the installation guide around the one-command front door

## Acceptance Cases

1. A new user can install Yazelix with one copy-paste command and no repo clone.
2. The resulting `yzx` command points at a persistent installed runtime, not a transient bootstrap path.
3. User config still lives under `~/.config/yazelix/user_configs/`.
4. Maintainer workflows can still use clone-based entrypoints without becoming the primary install story.
5. The canonical install guide becomes shorter because the front door is real, not because it hides steps with vague prose.
6. The onboarding contract clearly separates installer-managed bootstrap tools from host prerequisites.

## Verification

- manual review against `runtime_root_contract.md`
- future smoke test of the `nix run ...#install` path on Linux and macOS
- future validation that installed `yzx` and desktop entry resolve the persistent runtime root

## Traceability

- Bead: `yazelix-aa24`
- Bead: `yazelix-aa24.1`
- Bead: `yazelix-aa24.3`
- Bead: `yazelix-aa24.4`
- Defended by: future install smoke tests once the flake interface exists
