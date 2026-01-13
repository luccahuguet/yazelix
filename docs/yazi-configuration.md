# Yazi Configuration

Yazelix provides a simple, streamlined Yazi configuration system that generates configs from yazelix defaults with dynamic settings from `yazelix.toml`.

## Quick Start

Edit `yazelix.toml` to customize Yazi:

```toml
[yazi]
plugins = ["git"]           # Plugins to load
theme = "dracula"           # Color theme
sort_by = "modified"        # Sort files by modification time
```

Restart yazelix and your changes take effect!

## Configuration

All yazi settings are in `yazelix.toml`:

### Plugins

```toml
[yazi]
# Core plugins (auto-layout, sidebar-status) are always loaded
# sidebar-status removes a space-hungry status item so Yazi fits cleanly as a sidebar
# Add additional plugins here
plugins = ["git"]
```

**Bundled plugins:**
- `git` - Git status integration (shows file changes)

**Adding external plugins:**
```bash
# 1. Install the plugin
ya pkg add XYenon/clipboard.yazi

# 2. Add to yazelix.toml
[yazi]
plugins = ["git", "clipboard"]
```

### Theme

```toml
[yazi]
theme = "dracula"
```

For available themes, see: https://yazi-rs.github.io/docs/flavors/overview

### Sorting

```toml
[yazi]
sort_by = "alphabetical"
```

**Sort options:**
- `alphabetical` - A-Z sorting
- `natural` - Natural number ordering (file1, file2, file10)
- `modified` - Most recently modified first
- `created` - Most recently created first
- `size` - Largest files first

## How It Works

When yazelix starts:

1. Reads settings from `yazelix.toml`
2. Generates `yazi.toml` with your theme and sort_by settings
3. Generates `init.lua` with your plugin list
4. Copies bundled configs (keymap, theme)
5. Copies bundled plugins to plugins directory

**No merging. No personal folders. Simple generation.**

## Default Features

- **Layout ratio**: `[1, 4, 3]` optimized for sidebar mode
- **Git integration**: Shows git status in listings (via git plugin)
- **Editor integration**: Opens files with yazelix's configured editor
- **Auto layout**: Adjusts pane layout based on terminal width
- **Custom status bar**: Enhanced readability for sidebar mode

## Advanced Customization

For deeper customization beyond `yazelix.toml` options:

### Custom init.lua Code

Some plugins (like `yamb` for bookmarks) require custom Lua code beyond a simple `require().setup()`. Create your own init.lua:

```bash
~/.config/yazelix/configs/yazi/user/init.lua
```

Your code is appended after the auto-generated plugin requires. Example:

```lua
-- Custom yamb setup
require("yamb"):setup({
    jump_notify = true,
})

-- Custom keybindings or other Lua code
```

This file is gitignored, so your customizations persist across updates.

### Custom Keybindings

Add custom keybindings without editing the base keymap:

```bash
~/.config/yazelix/configs/yazi/user/keymap.toml
```

Your keybindings are merged with yazelix defaults. Example for yamb bookmarks:

```toml
[[mgr.append_keymap]]
on = ["b", "a"]
run = "plugin yamb save"
desc = "Add bookmark"

[[mgr.append_keymap]]
on = ["b", "g"]
run = "plugin yamb jump"
desc = "Jump to bookmark"
```

This file is gitignored, so your keybindings persist across updates.

### Edit Source Configs

For structural changes to the base configuration:

```bash
# These are the source templates (your changes persist)
~/.config/yazelix/configs/yazi/yazelix_yazi.toml    # Main config
~/.config/yazelix/configs/yazi/yazelix_keymap.toml  # Keybindings
~/.config/yazelix/configs/yazi/yazelix_theme.toml   # Theme details
```

## Plugin Management

Plugin catalog: https://github.com/yazi-rs/plugins

For plugin management commands, see: https://yazi-rs.github.io/docs/cli

After installing plugins via `ya pkg`, add them to `yazelix.toml`:

```toml
[yazi]
plugins = ["git", "your-new-plugin"]
```

## Troubleshooting

**Config not updating?**
```bash
# Manually regenerate configs
cd ~/.config/yazelix
nu nushell/scripts/setup/yazi_config_merger.nu .
```

**Plugin not loading?**
- Check plugin name in `yazelix.toml` matches installed plugin (without `.yazi` extension)
- Verify plugin exists: `ls ~/.local/share/yazelix/configs/yazi/plugins/`
- Check for warnings during yazelix startup

**Want default settings?**
```bash
# Reset yazelix.toml to defaults
cd ~/.config/yazelix
cp yazelix_default.toml yazelix.toml
```

## Configuration Reference

Full yazi configuration docs: https://yazi-rs.github.io/docs/configuration/yazi

Yazelix exposes the most commonly changed settings via `yazelix.toml`. For advanced configuration, edit the source configs in `~/.config/yazelix/configs/yazi/`.
