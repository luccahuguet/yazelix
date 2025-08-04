# Zellij Configuration

Yazelix uses a three-layer Zellij configuration system that prevents git conflicts when customizing settings.

## Quick Start

```bash
# Copy templates to create your personal configs (one time setup)
cp -r configs/zellij/user configs/zellij/personal

# Edit files in configs/zellij/personal/ to customize Zellij
# Your settings automatically override defaults
```

## How It Works

The configuration system merges three layers in order of priority:

1. **Zellij defaults** - Base configuration from `zellij setup --dump-config`
2. **Yazelix overrides** (`yazelix_overrides.kdl`) - Yazelix-specific settings (git tracked)
3. **Your personal settings** (`personal/user_config.kdl`) - Your customizations (git ignored, highest priority)

When you start Yazelix, these layers merge automatically into `~/.local/share/yazelix/configs/zellij/config.kdl`. The system uses smart caching - configs only regenerate when source files change. Your personal configs persist across Yazelix updates without conflicts.

## Common Customizations

For complete examples and documentation, see the [user config template](../configs/zellij/user/user_config.kdl).

**Themes and UI:**
```kdl
theme "dracula"
// theme "nord"
// theme "tokyo-night"

// Disable mouse mode if it interferes with terminal selection
mouse_mode false

// Simplified UI for better compatibility
simplified_ui true
```

**Session behavior:**
```kdl
// Show startup tips (Yazelix disables by default)
show_startup_tips true

// What to do when terminal closes
on_force_close "quit"

// Copy/paste settings
copy_on_select false
copy_clipboard "primary"
scroll_buffer_size 50000
```


## Current Yazelix Defaults

- Default layout: `yazelix` (sidebar) or `yazelix_no_sidebar`
- Copy command: `wl-copy` (Wayland clipboard)
- Scrollback editor: `hx` (Helix)
- Session serialization: enabled for persistence
- Startup tips: disabled
- UI: rounded pane corners enabled

## Troubleshooting

- **Config not updating?** Run: `nu nushell/scripts/setup/zellij_config_merger.nu .`
- **KDL syntax errors?** Check your personal config file syntax against examples in the template
- **Want to reset?** Delete `configs/zellij/personal/` and copy templates again: `cp -r configs/zellij/user configs/zellij/personal`

For complete Zellij configuration options: https://zellij.dev/documentation/