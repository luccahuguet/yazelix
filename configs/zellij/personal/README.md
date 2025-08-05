# Personal Zellij Configuration

This directory contains your personal Zellij customizations that override Yazelix defaults.

## Quick Start

To customize Zellij settings:

1. **Copy the template**: `cp ../user/user_config.kdl ./user_config.kdl`
2. **Edit your config**: Modify `user_config.kdl` to customize Zellij behavior
3. **Restart Yazelix**: Your changes will be automatically merged and applied

## How It Works

Yazelix uses a three-layer configuration system:

1. **Zellij defaults** - Base configuration from `zellij setup --dump-config`
2. **Yazelix overrides** - Yazelix-specific settings (in `../yazelix_overrides.kdl`)
3. **Your personal settings** (in this directory) - **Highest priority**

Your settings in this directory always take precedence and are git-ignored to prevent conflicts when updating Yazelix.

## Configuration Layers

- **Generated config**: `~/.local/share/yazelix/configs/zellij/config.kdl` (automatically created)
- **Your customizations**: Files in this `personal/` directory
- **Template reference**: Files in `../user/` directory (don't edit these directly)

## Common Customizations

See the template file `../user/user_config.kdl` for examples of:
- Themes and UI customization
- Keybinding modifications  
- Session behavior settings
- Copy/paste configuration

## Documentation

- [Yazelix Zellij docs](../../../docs/zellij-configuration.md)
- [Official Zellij documentation](https://zellij.dev/documentation/)

---

**Note**: This directory exists to ensure proper config generation. You can safely delete this README if you add your own configuration files.