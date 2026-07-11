# Yazelix Yazi Configuration

Yazelix provides a managed Yazi configuration built from `settings.jsonc`, managed sidecar overrides under `~/.config/yazelix/yazi/`, Yazelix-owned sidebar/editor plugins, and the packaged `yazelix-yazi-assets` config-pack/flavor/plugin pack

## Quick Start

```bash
# Create only the sidecar files you need:
#   ~/.config/yazelix/yazi/yazi.toml
#   ~/.config/yazelix/yazi/keymap.toml
#   ~/.config/yazelix/yazi/init.lua
```

## Documentation

For the complete configuration guide, see [Yazi Configuration Documentation](../../docs/yazi-configuration.md)

## Current Defaults

- Config-pack templates live in `yazelix-yazi-assets`
- Main keeps sidebar/editor integration plugins

## Where the plugins live (nothing is missing from `configs/yazi/plugins/`)

The eight bundled Yazi plugins are split across two repositories by design, then
merged into the runtime at materialization time. `configs/yazi/plugins/` in this
repo intentionally holds only the three Yazelix-authored plugins — the other five
are vendored in the `yazelix-yazi-assets` child repo (consumed as a flake input),
**not** copied into this tree:

| Plugin | Source | Enabled by default |
|---|---|---|
| `sidebar-status` | this repo (`configs/yazi/plugins/`) | always (core) |
| `sidebar-state` | this repo (`configs/yazi/plugins/`) | always (core) |
| `zoxide-editor` | this repo (`configs/yazi/plugins/`) | via keybinding (`plugin zoxide-editor`) |
| `auto-layout` | `yazelix-yazi-assets` | always (core) |
| `git` | `yazelix-yazi-assets` | yes (`yazi.plugins`) |
| `starship` | `yazelix-yazi-assets` | yes (`yazi.plugins`) |
| `lazygit` | `yazelix-yazi-assets` | yes (`yazi.plugins`) |
| `smart-tabs` | `yazelix-yazi-assets` | yes (`yazi.plugins`) |

All eight land in the materialized runtime at
`~/.local/share/yazelix/configs/yazi/plugins/`. If you only see three plugins in
`configs/yazi/plugins/`, that is expected — the vendored five come from the child
repo, not this directory.
