# Yazelix: Yazi Configuration

Yazelix provides a two-layer Yazi configuration system that separates Yazelix defaults from your personal customizations, preventing git conflicts when updating Yazelix.

## Quick Start

To customize Yazi settings, copy the template directory once:

```bash
cp -r configs/yazi/user configs/yazi/personal
```

Then edit files in `configs/yazi/personal/` to customize your Yazi experience. Yazelix will automatically merge your settings with the defaults.

## Configuration Layers

1. **Yazelix defaults** (`yazelix_*.toml`) - Sensible defaults, git tracked
2. **Your customizations** (`personal/*.toml`) - Your settings override defaults, git ignored

## Files Structure

```
configs/yazi/
├── user/           # Templates with documentation (git tracked)
├── personal/       # Your customizations (git ignored)
├── yazelix_*.toml  # Yazelix defaults (git tracked)
└── plugins/        # Yazelix plugin defaults
```

## Configuration Options

For comprehensive configuration options, see: https://yazi-rs.github.io/docs/configuration/yazi

## Features

- **Dynamic merging**: Your settings intelligently override Yazelix defaults
- **No git conflicts**: Personal configs are git ignored  
- **Smart caching**: Configs regenerate only when files change
- **TOML validation**: Proper section merging prevents duplicate keys
- **Plugin support**: Personal plugins override Yazelix plugins

## Current Defaults

- Layout ratio: `[1, 4, 3]` optimized for sidebar mode (20% terminal width)
- Git integration: Shows git status in file listings
- Editor integration: Opens files with configured editor in Zellij
- Custom status bar: Enhanced readability (courtesy of Yazi's creator!)
