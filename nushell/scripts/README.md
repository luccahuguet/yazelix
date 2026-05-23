# Nushell Scripts Organization

This directory contains the remaining irreducible Nushell code in Yazelix

## Current Shape

### `core/`
Narrow runtime entrypoints that still own shell or Zellij handoff

- `start_yazelix_inner.nu` - Interactive startup handoff into Zellij after Rust-owned env, preflight, and materialization decisions

### `integrations/`
No tracked integration owner files remain here after the Rust `yzx_control zellij`
entrypoint cut. The surviving shell boundary is the sidebar Yazi launcher wrapper
under `zellij_wrappers/`

### `setup/`
Welcome sequencing and human-facing startup presentation

- `welcome.nu` - Interactive welcome display and prompt gating

### `utils/`
Small surviving helpers plus runtime shell adapters

- `constants.nu` - Version constants and static metadata accessors
- `yzx_core_bridge.nu` - Narrow Rust helper transport seam
- `runtime_paths.nu` - Minimal runtime/state path helpers that still feed shell owners

### `yzx/`
No public shell-owned `yzx` modules remain. Syntax validation is Rust-owned by
`yzx_repo_validator validate-nushell-syntax`

### `zellij_wrappers/`
No runtime wrapper remains. The managed Yazi sidebar is launched by the
Rust-owned `yzx sidebar yazi` command

## Canonical Entry Points

For normal usage, prefer the shipped CLI and Rust-owned public commands

```bash
yzx launch
yzx run <command>
~/.config/yazelix/shells/posix/yzx_cli.sh help
```

For maintainer workflows, prefer the Rust-owned runner surfaces

```bash
yzx dev rust fmt --check
yzx dev rust check
yzx dev rust test <filter>
yzx dev inspect_session
yzx dev build_pane_orchestrator --sync
yzx dev sync_yzpp_wasm
yzx dev test --lint-only
yzx dev test
```

Use the direct `yzx dev rust ...` commands for the fast edit-check loop. Use
`yzx dev test`, package validators, Nix builds, and Home Manager switches as
explicit final gates when the change needs package/runtime coverage

The public launch, desktop, restart, enter, popup, update, sweep, plugin-build, and issue-sync families are no longer owned by direct Nushell modules

## File Naming Convention

All files use underscores, for example `start_yazelix_inner.nu` and `runtime_commands.nu`
