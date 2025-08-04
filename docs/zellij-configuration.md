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

**Important**: The merger uses simple concatenation - Zellij reads all three sections and uses the last occurrence of any setting. For simple settings this works perfectly, but be careful with nested blocks like `ui`, `keybinds`, or `themes` to avoid unintended duplicates.

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

## Best Practices

**For UI settings**, add them to your personal config (no conflicts with Yazelix defaults):
```kdl
ui {
    pane_frames {
        rounded_corners true
    }
}
```

**For keybindings**, edit the layout files directly:
- `configs/zellij/layouts/yazelix.kdl` (sidebar mode)
- `configs/zellij/layouts/yazelix_no_sidebar.kdl` (no-sidebar mode)
- Only define keybinds in personal config if you want to replace ALL bindings

**Simple settings** (like `theme`, `copy_command`) work perfectly - your value always wins.

## Current Yazelix Defaults

- Default layout: `yazelix` (sidebar) or `yazelix_no_sidebar`
- Copy command: `wl-copy` (Wayland clipboard)
- Scrollback editor: `hx` (Helix)
- Session serialization: enabled for persistence
- Startup tips: disabled

## Troubleshooting

**Config not updating?**
- Run: `nu nushell/scripts/setup/zellij_config_merger.nu .`

**Settings not working as expected?**
- Check `~/.local/share/yazelix/configs/zellij/config.kdl` for duplicate sections
- Look for your setting - it should appear last to take effect
- For nested blocks (ui, keybinds), you may need to override the entire section

**KDL syntax errors?**
- Check your personal config file syntax against examples in the template
- Zellij will show parsing errors on startup if KDL is invalid

**Want to reset?**
- Delete `configs/zellij/personal/` and copy templates again: `cp -r configs/zellij/user configs/zellij/personal`

For complete Zellij configuration options: https://zellij.dev/documentation/