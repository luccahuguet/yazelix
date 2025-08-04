# Yazi Configuration

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

## How It Works

When Yazelix starts, it automatically:

1. Checks if your personal configs have changed
2. Merges Yazelix defaults with your personal settings
3. Generates clean, validated TOML files
4. Sets environment variables to use the merged configs
5. Caches the result until files change again

This system ensures your customizations persist across Yazelix updates without conflicts.

## Troubleshooting

**Config not updating?** 
- Check file permissions in `~/.local/share/yazelix/configs/yazi/`
- Manually regenerate: `nu nushell/scripts/setup/yazi_config_merger.nu .`

**TOML parsing errors?**
- Validate your personal TOML files: `nu -c "open configs/yazi/personal/yazi.toml | from toml"`
- Check for syntax errors in your customizations

**Want to reset?**
- Delete your `configs/yazi/personal/` directory
- Copy fresh templates: `cp -r configs/yazi/user configs/yazi/personal`