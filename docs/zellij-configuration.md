# Zellij Configuration

Yazelix uses a three-layer Zellij configuration system centered on its managed user config surface.

## Quick Start

Edit your Yazelix-managed Zellij config:

```bash
~/.config/yazelix/user_configs/zellij/config.kdl
```

If you already have a native Zellij config, import it into the managed path first:

```bash
yzx import zellij
```

## How It Works

The merger now prefers your **Yazelix-managed Zellij config** when present, then falls back to your native Zellij config, then forcibly layers Yazelix requirements on top:

1. **User config**: `~/.config/yazelix/user_configs/zellij/config.kdl` (if it exists). If missing, Yazelix reads `~/.config/zellij/config.kdl` as a read-only fallback. If neither exists, Yazelix falls back to `zellij setup --dump-config`.
2. **Dynamic Yazelix settings**: Generated from `yazelix.toml` (e.g., rounded corners) and appended after the user config so they win.
3. **Enforced Yazelix settings**: Always appended last to guarantee required behavior:
   - `pane_frames false` (needed for `zjstatus`)
   - `support_kitty_keyboard_protocol` set from `yazelix.toml` (default: false)
   - `on_force_close` set from Yazelix session mode (`quit` for default non-persistent sessions, `detach` for persistent sessions)
   - `default_layout` set to Yazelix’s layout file (absolute path)
   - `layout_dir` set to Yazelix’s generated layouts directory

Layouts are copied into `~/.local/share/yazelix/configs/zellij/layouts`, and the merged config is written to `~/.local/share/yazelix/configs/zellij/config.kdl` on every launch. Yazelix also passes `--pane-frames false` and an absolute `--default-layout` at launch for extra safety.

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

**Zjstatus widget tray (yazelix.toml):**
```toml
[zellij]
widget_tray = [
  "editor",  # Active editor
  "shell",   # Active shell
  "term",    # Terminal emulator
  "cpu",     # CPU usage
  "ram",     # RAM usage
]
```
Comment out any line to hide that widget. Order matters. Restart Yazelix to regenerate layouts.

**Session behavior:**
```kdl
// Show startup tips (Yazelix disables by default)
show_startup_tips true

// Copy/paste settings
copy_on_select false
copy_clipboard "primary"
scroll_buffer_size 50000
```

## Best Practices

**For UI settings**, add them to your managed Zellij config:
```kdl
ui {
    pane_frames {
        rounded_corners true  # Yazelix may override via yazelix.toml
    }
}
```

**For keybindings**, edit the layout files directly:
- `configs/zellij/layouts/yzx_side.kdl` (sidebar mode)
- `configs/zellij/layouts/yzx_no_side.kdl` (no-sidebar mode)
- Only define keybinds in personal config if you want to replace ALL bindings

**Simple settings** (like `theme`, `copy_command`) work perfectly - your value always wins.

## Current Yazelix Defaults

- Default layout: `yzx_side` (sidebar) or `yzx_no_side`
- Copy command: `wl-copy` (Wayland clipboard)
- Scrollback editor: `hx` (Helix)
- Session serialization: enabled for persistence
- `on_force_close`: `quit` for default non-persistent sessions, `detach` for persistent sessions
- Startup tips: disabled

## Troubleshooting

**Config not updating?**
- Run: `nu nushell/scripts/setup/zellij_config_merger.nu .`

**Settings not working as expected?**
- Check `~/.local/share/yazelix/configs/zellij/config.kdl` for duplicate sections
- Look for your setting - it should appear last to take effect
- For nested blocks (ui, keybinds), you may need to override the entire section

**KDL syntax errors?**
- Check your managed Zellij config syntax against examples in the template
- Zellij will show parsing errors on startup if KDL is invalid

**Migrating from a native Zellij config?**
- Preferred explicit path: run `yzx import zellij` to copy `~/.config/zellij/config.kdl` into `~/.config/yazelix/user_configs/zellij/config.kdl`
- If you already have a managed override and want to replace it, use `yzx import zellij --force` so Yazelix writes a backup first
- If the managed file is missing, Yazelix can still read `~/.config/zellij/config.kdl` as the base config for that launch, but it will not move or delete it
- If both files exist, Yazelix keeps using the managed `user_configs` copy and leaves the native Zellij config alone for plain `zellij` launches
- If you want changes from `~/.config/zellij/config.kdl` to become the managed Yazelix config, run `yzx import zellij` explicitly

For complete Zellij configuration options: https://zellij.dev/documentation/
