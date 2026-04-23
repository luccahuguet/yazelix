# Nushell Scripts Organization

This directory contains the surviving Nushell code in Yazelix

## Current Shape

### `core/`
Narrow runtime entrypoints that still own shell or Zellij handoff

- `start_yazelix_inner.nu` - Interactive startup handoff into Zellij after Rust-owned env, preflight, and materialization decisions

### `integrations/`
Live Zellij, Yazi, and managed-editor shell glue that has not been fully deleted yet

- `yazi.nu` - Sidebar Yazi actions and sync helpers
- `zellij.nu` - Pane-orchestrator actions and managed-pane shell integration
- `managed_editor.nu` - Direct Yazi/editor entrypoint plus managed-editor integration

### `setup/`
Shell bootstrap and initializer generation

- `environment.nu` - Runtime environment file generation and setup orchestration

### `utils/`
Small surviving helpers plus runtime shell adapters

- `constants.nu` - Version constants and static metadata accessors
- `logging.nu` - Logging helpers
- `yzx_core_bridge.nu` - Narrow Rust helper transport seam

### `dev/`
Maintainer and validation helpers that have not been deleted or ported yet

- Syntax validation is Rust-owned by `yzx_repo_validator validate-nushell-syntax`

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

The public launch, desktop, restart, enter, and popup families are no longer owned by direct Nushell modules

## Manual Maintainer Helpers

These are manual or exploratory helpers, not normal runtime entrypoints

## File Naming Convention

All files use underscores, for example `start_yazelix_inner.nu` and `managed_editor.nu`
