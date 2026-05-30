# Yazelix Helix Steel Plugins

This directory is the runtime-owned source for curated Helix Steel plugin files

The initial bundled files come from `mattwparas/helix-config` and are loaded
through the generated Yazelix Steel config when their corresponding settings are
enabled

The supported user-facing config keys are:

- `helix.plugins.recentf`
- `helix.plugins.splash`
- `helix.plugins.spacemacs_theme`

`cogs/keymaps.scm` and `cogs/labelled-buffers.scm` are bundled helper modules,
not separate settings

Users can override or test the same file contract locally by placing matching
files under `~/.config/yazelix/helix/steel_plugins/`
