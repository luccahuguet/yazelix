# Yazelix: Zellij Configuration

Yazelix provides a three-layer Zellij configuration system that prevents git conflicts when customizing Zellij settings.

## Quick Start

```bash
# Copy templates to create your personal configs (one time setup)
cp -r configs/zellij/user configs/zellij/personal

# Edit files in configs/zellij/personal/ to customize Zellij
# Your settings automatically override Yazelix defaults
```

## Documentation

For complete configuration guide, see: [Zellij Configuration Documentation](../../docs/zellij-configuration.md)

## Current Defaults

- Three-layer merging: Zellij defaults + Yazelix overrides + your personal settings
- Default layout optimized for sidebar/no-sidebar modes
- Wayland clipboard integration (`wl-copy`)
- Helix editor integration for scrollback
- Session persistence and Helix-friendly keybindings