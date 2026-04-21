# Yazi Configuration

Yazelix provides a layered Yazi configuration system built from Yazelix defaults, dynamic settings from `yazelix.toml`, and optional managed overrides under `~/.config/yazelix/user_configs/yazi/`.

## Quick Start

Edit `yazelix.toml` to customize the built-in Yazi knobs:

```toml
[yazi]
plugins = ["git"]           # Plugins to load
theme = "dracula"           # Color theme
sort_by = "modified"        # Sort files by modification time
```

Restart yazelix and your changes take effect.

## Configuration

Yazi has two customization layers:

1. Built-in Yazelix-facing Yazi settings in `yazelix.toml`
2. Optional managed Yazi override files in `~/.config/yazelix/user_configs/yazi/`

The `yazelix.toml` layer controls the common knobs Yazelix understands directly.

### Binary Overrides

```toml
[yazi]
command = "/path/to/custom/yazi"  # Optional: managed Yazi binary override
ya_command = "/path/to/custom/ya" # Optional: managed `ya` CLI override
```

Leave both empty to use `yazi` and `ya` from `PATH`.

Use this only when Yazelix-managed Yazi launches and sidebar actions need a specific binary. Custom plugin initialization should still go in `~/.config/yazelix/user_configs/yazi/init.lua`.

### Plugins

```toml
[yazi]
# Core plugins (sidebar-status, auto-layout, sidebar-state) are always loaded before this list
# sidebar-status removes a space-hungry status item so Yazi fits cleanly as a sidebar
# Add additional plugins here
plugins = ["git"]
```

**Bundled plugins and helpers:**
- `git` - Git status integration (shows file changes)
- `zoxide-editor` - Bundled helper plugin behind `Alt+z`; it drives the direct-open Zoxide jump without needing a `plugins = [...]` entry

**Adding external plugins:**
```bash
# 1. Install the plugin
ya pkg add XYenon/clipboard.yazi

# 2. Add to yazelix.toml
[yazi]
plugins = ["git", "clipboard"]
```

**Note:** Plugins in this list get auto-generated `require("plugin"):setup()` calls. If you need custom configuration options, don't add the plugin here; configure them manually in `~/.config/yazelix/user_configs/yazi/init.lua` instead.

### Theme

```toml
[yazi]
theme = "dracula"
```

For available themes, see: https://yazi-rs.github.io/docs/flavors/overview

The curated `random-dark` / `random-light` flavor names used by Yazelix live in `config_metadata/yazi_render_plan.toml` (defaults and `sort_by` validation stay in `config_metadata/main_config_contract.toml`).

Leave the field unset, or set `theme = "default"`, to keep Yazi's upstream built-in theme.

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

1. Reads the built-in `[yazi]` settings from `yazelix.toml`
2. Generates the managed base `yazi.toml` from Yazelix defaults plus those settings
3. Merges your optional `~/.config/yazelix/user_configs/yazi/yazi.toml` overrides when that file exists
4. Generates `init.lua` with the built-in plugin list, then appends your optional `~/.config/yazelix/user_configs/yazi/init.lua`
5. Merges your optional `~/.config/yazelix/user_configs/yazi/keymap.toml` with the Yazelix keymap layer
6. Copies bundled configs, plugins, and flavors into the generated runtime Yazi directory

The generated runtime config lives under `~/.local/share/yazelix/configs/yazi/`. You customize the managed inputs under `user_configs/yazi/`, not the generated output.

## Default Features

- **Layout ratio**: `[1, 4, 3]` optimized for sidebar mode
- **Git integration**: Shows git status in listings (via git plugin)
- **Editor integration**: Opens files with yazelix's configured editor
- **Direct zoxide jump**: `Alt+z` opens a Zoxide picker and retargets the managed editor/workspace directly to the selected directory
- **Auto layout**: Adjusts pane layout based on terminal width
- **Custom status bar**: Enhanced readability for sidebar mode

## Advanced Customization

For deeper customization beyond the built-in `yazelix.toml` options, use the managed override files under `~/.config/yazelix/user_configs/yazi/`.

### Custom init.lua Code

Some plugins (like `yamb` for bookmarks) require custom Lua code beyond a simple `require().setup()`. Create your own init.lua:

```bash
~/.config/yazelix/user_configs/yazi/init.lua
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
~/.config/yazelix/user_configs/yazi/keymap.toml
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

Yazelix intentionally keeps Yazi's upstream open flow intact:

- `Enter`: opens the hovered item through Yazelix's configured editor opener
- `o`: uses Yazi's built-in open action
- `O`: opens Yazi's built-in "Open with" menu for alternate handlers, including the system file manager path when available
- `Z`: keeps Yazi's native Zoxide jump behavior inside Yazi
- `Alt+z`: runs the bundled `zoxide-editor` plugin so the selected directory becomes the managed editor/workspace target immediately

Because `O` already exposes the practical "open outside the editor" flow, Yazelix does not add a separate default keybinding for opening in the host file manager.

This file is gitignored, so your keybindings persist across updates.

### Import an Existing Native Yazi Config

If you already have a native Yazi setup, import the supported override files into Yazelix-managed overrides:

```bash
yzx import yazi
```

This imports `yazi.toml`, `keymap.toml`, and `init.lua` from `~/.config/yazi/` into `~/.config/yazelix/user_configs/yazi/`.

If you need to replace existing managed override files, use:

```bash
yzx import yazi --force
```

Yazelix writes backups before overwriting existing managed files. Plugin directories and other broader Yazi state are intentionally not imported by this command.

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
# Restart Yazelix or open a fresh Yazelix window so the managed Yazi config is regenerated
yzx restart
```

- If the managed Yazi files still look stale, run `yzx doctor --fix` and restart Yazelix once more

**Plugin not loading?**
- Check plugin name in `yazelix.toml` matches installed plugin (without `.yazi` extension)
- Verify plugin exists: `ls ~/.local/share/yazelix/configs/yazi/plugins/`
- Check for warnings during yazelix startup

**Migrating from a native Yazi config?**
- Run `yzx import yazi` to copy `yazi.toml`, `keymap.toml`, and `init.lua` into `~/.config/yazelix/user_configs/yazi/`
- Use `yzx import yazi --force` to back up and replace existing managed override files

**Want default settings?**
```bash
# Reset yazelix.toml to defaults
cd ~/.config/yazelix
cp yazelix_default.toml yazelix.toml
```

## Configuration Reference

Full yazi configuration docs: https://yazi-rs.github.io/docs/configuration/yazi

Yazelix exposes the most commonly changed settings via `yazelix.toml`. For advanced configuration, edit the source configs in `~/.config/yazelix/configs/yazi/`.
