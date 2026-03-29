# Yazelix: Zellij Configuration

Yazelix uses your native Zellij config when present, then layers Yazelix requirements on top.

## Quick Start

```bash
# Edit your native Zellij config
~/.config/zellij/config.kdl
```

## Documentation

For complete configuration guide, see: [Zellij Configuration Documentation](../../docs/zellij-configuration.md)

## Current Defaults

- Three-layer merging: Zellij defaults + Yazelix overrides + your personal settings
- Default layout optimized for sidebar/no-sidebar modes
- Wayland clipboard integration (`wl-copy`)
- Helix editor integration for scrollback
- Session persistence and Helix-friendly keybindings
