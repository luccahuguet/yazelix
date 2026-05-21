# Yazelix: Zellij Configuration

Yazelix uses `settings.jsonc` for managed workspace behavior and managed Zellij appearance such as theme and rounded corners. Use `~/.config/yazelix/zellij.kdl` only for advanced native Zellij settings that Yazelix does not render.

## Quick Start

```bash
# Edit advanced native Zellij settings that Yazelix does not render
~/.config/yazelix/zellij.kdl
```

## Documentation

For complete configuration guide, see: [Zellij Configuration Documentation](../../docs/zellij-configuration.md)

## Current Defaults

- Layered merging: native Zellij settings + Yazelix generated keybindings, plugins, dynamic settings, and enforced settings
- Default layout optimized for sidebar/no-sidebar modes
- Managed theme, pane frame, and rounded-corner settings
- Session persistence and Helix-friendly keybindings
