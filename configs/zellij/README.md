# Yazelix: Zellij Configuration

Yazelix uses `settings.jsonc` for managed workspace behavior and `~/.config/yazelix/zellij.kdl` for advanced native Zellij settings that do not include keybindings.

## Quick Start

```bash
# Edit advanced non-keybinding Zellij settings
~/.config/yazelix/zellij.kdl
```

## Documentation

For complete configuration guide, see: [Zellij Configuration Documentation](../../docs/zellij-configuration.md)

## Current Defaults

- Layered merging: native Zellij settings + Yazelix generated keybindings, plugins, dynamic settings, and enforced settings
- Default layout optimized for sidebar/no-sidebar modes
- Wayland clipboard integration (`wl-copy`)
- Helix editor integration for scrollback
- Session persistence and Helix-friendly keybindings
