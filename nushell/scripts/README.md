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
Shell bootstrap, shellhook env mutation, and welcome sequencing

- `environment.nu` - Shellhook env setup and initializer generation
- `welcome.nu` - Interactive welcome display and prompt gating

### `utils/`
Small surviving helpers plus runtime shell adapters

- `constants.nu` - Version constants and static metadata accessors
- `yzx_core_bridge.nu` - Narrow Rust helper transport seam
- `runtime_paths.nu` - Minimal runtime/state path helpers that still feed shell owners
- `transient_pane_contract.nu` - Tiny popup/menu/sidebar shell facts

### `yzx/`
The remaining public shell-owned surfaces

- `dev.nu` - Thin maintainer router plus the startup-profile shell harness
- `menu.nu` - Interactive command palette
- Syntax validation is Rust-owned by `yzx_repo_validator validate-nushell-syntax`

### `zellij_wrappers/`
One surviving runtime wrapper

- `launch_sidebar_yazi.nu` - Sidebar Yazi launcher that still needs the shell-facing Yazi handoff

## Canonical Entry Points

For normal usage, prefer the shipped CLI and Rust-owned public commands

```bash
yzx launch
yzx run <command>
~/.config/yazelix/shells/posix/yzx_cli.sh help
```

For maintainer workflows, prefer the Rust-owned runner surfaces

```bash
yzx dev build_pane_orchestrator --sync
yzx dev test --lint-only
yzx dev test
```

The public launch, desktop, restart, enter, popup, update, sweep, plugin-build, and issue-sync families are no longer owned by direct Nushell modules

## File Naming Convention

All files use underscores, for example `start_yazelix_inner.nu` and `launch_sidebar_yazi.nu`
