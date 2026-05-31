# Yazelix Helix Steel Plugins

This directory is the runtime-owned source for curated Helix Steel plugin files

The initial bundled files come from `mattwparas/helix-config` and are loaded
through the generated Yazelix Steel config when their corresponding settings are
enabled

The bundled plugin repository is declared in `manifest.toml`

The supported user-facing config surface is:

- `helix.steel_plugins.enabled`: bundled plugin ids to load from this repository
- `helix.steel_plugins.extra`: user-owned plugin manifests

User-owned plugin files are resolved below
`~/.config/yazelix/helix/steel_plugins/`
