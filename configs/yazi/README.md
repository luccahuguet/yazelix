# Yazelix: Yazi Configuration

Yazelix provides a two-layer Yazi configuration system that prevents git conflicts when customizing Yazi settings.

## Quick Start

```bash
mkdir -p ~/.config/yazelix/user_configs/yazi

# Then create only the files you need:
#   ~/.config/yazelix/user_configs/yazi/yazi.toml
#   ~/.config/yazelix/user_configs/yazi/keymap.toml
#   ~/.config/yazelix/user_configs/yazi/init.lua
```

## Documentation

For complete configuration guide, see: [Yazi Configuration Documentation](../../docs/yazi-configuration.md)

## Current Defaults

- Layout ratio optimized for sidebar mode (20% terminal width)  
- Git integration showing file status
- Editor integration with Zellij
- Custom status bar (courtesy of Yazi's creator!)
