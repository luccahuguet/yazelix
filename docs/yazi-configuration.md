# Yazi Configuration

Yazelix provides a layered Yazi configuration system built from Yazelix defaults, dynamic settings from `settings.jsonc`, and optional managed Yazi-home overrides under `~/.config/yazelix/yazi/`.

## Quick Start

Edit `settings.jsonc` to customize the built-in Yazi knobs:

```jsonc
{
  "yazi": {
    "plugins": ["git"],
    "theme": "dracula",
    "sort_by": "modified"
  }
}
```

Restart yazelix and your changes take effect.

## Configuration

Yazi has two customization layers:

1. Built-in Yazelix-facing Yazi settings in `settings.jsonc`
2. Optional managed Yazi override files and packages in `~/.config/yazelix/yazi/`

The `settings.jsonc` layer controls the common knobs Yazelix understands directly.

### Binary Overrides

```jsonc
{
  "yazi": {
    "command": "/path/to/custom/yazi",
    "ya_command": "/path/to/custom/ya"
  }
}
```

Leave both empty to use `yazi` and `ya` from `PATH`.

Use this only when Yazelix-managed Yazi launches and file-tree sidebar actions need a specific binary. Custom plugin initialization should still go in `~/.config/yazelix/yazi/init.lua`.

The Zellij sidebar launcher itself is controlled by `editor.sidebar_command` and `editor.sidebar_args` in `settings.jsonc`. Leave those at their defaults unless you intentionally want the managed sidebar slot to run something other than Yazelix's Yazi file-tree adapter. Custom commands do not inherit the default Yazi adapter arg when `sidebar_args` is left unchanged.

### Plugins

```jsonc
{
  "yazi": {
    // Core plugins (sidebar-status, auto-layout, sidebar-state) are always loaded before this list
    "plugins": ["git"]
  }
}
```

**Bundled plugins and helpers:**
- `git` - Git status integration (shows file changes)
- `sidebar-state` - Reports the managed Yazi file-tree sidebar id and cwd to the pane orchestrator for tab-local reveal and sync
- `zoxide-editor` - Bundled helper plugin behind `Alt+z`; it drives the direct-open Zoxide jump without needing a `plugins = [...]` entry

**Adding external plugins:**
```bash
# 1. Install the plugin
ya pkg add XYenon/clipboard.yazi

# 2. Import native Yazi plugins into Yazelix-managed overrides
yzx import yazi
```

Then add the plugin name to `settings.jsonc`:

```jsonc
{
  "yazi": {
    "plugins": ["git", "clipboard"]
  }
}
```

**Note:** Plugins in this list get auto-generated `require("plugin"):setup()` calls. If you need custom configuration options, don't add the plugin here; configure them manually in `~/.config/yazelix/yazi/init.lua` instead.

### Theme

```jsonc
{
  "yazi": {
    "theme": "dracula"
  }
}
```

Yazelix's bundled flavor catalog is packaged by `yazelix-yazi-assets`; the upstream Yazi flavor docs are at https://yazi-rs.github.io/docs/flavors/overview

The curated `random-dark` / `random-light` flavor names used by Yazelix live in `config_metadata/yazi_render_plan.toml` (defaults and `sort_by` validation stay in `config_metadata/main_config_contract.toml`).

Leave the field unset, or set `theme = "default"`, to keep Yazi's upstream built-in theme.

### Sorting

```jsonc
{
  "yazi": {
    "sort_by": "alphabetical"
  }
}
```

**Sort options:**
- `alphabetical` - A-Z sorting
- `natural` - Natural number ordering (file1, file2, file10)
- `modified` - Most recently modified first
- `created` - Most recently created first
- `size` - Largest files first

## How It Works

When yazelix starts:

1. Reads the built-in `yazi` settings from `settings.jsonc`
2. Generates the managed base `yazi.toml` from Yazelix defaults plus those settings
3. Merges your optional `~/.config/yazelix/yazi/yazi.toml` overrides when that file exists, while preserving Yazelix-owned `[opener].edit`
4. Generates `init.lua` with the built-in plugin list, then appends your optional `~/.config/yazelix/yazi/init.lua`
5. Merges your optional `~/.config/yazelix/yazi/keymap.toml` with the Yazelix keymap layer
6. Copies Yazelix-owned configs, packaged `yazelix-yazi-assets` plugins and flavors, and explicitly imported user plugins/flavors into the generated runtime Yazi directory

The generated runtime config lives under `~/.local/share/yazelix/configs/yazi/`. You customize the managed Yazi home under `~/.config/yazelix/yazi/`, not the generated output.

### Yazi opener ownership

Yazelix owns the generated `[opener].edit` entry. That opener sends file opens through `yzx_control zellij open-editor` so files target the managed editor pane instead of spawning an unmanaged editor.

`~/.config/yazelix/yazi/yazi.toml` can add or override other Yazi settings, but it does not replace `[opener].edit`. Use `keymap.toml` to remap file-open keys, and use `init.lua` for custom Lua setup.

When an override repeats a native Yazi array setting such as `[mgr].ratio`, the managed value replaces the Yazelix default instead of appending to it. Use this to disable the preview pane with a ratio such as `[1, 4, 0]`.

## Default Features

- **Layout ratio**: `[1, 4, 3]` optimized for sidebar mode
- **Git integration**: Shows git status in listings (via git plugin)
- **Editor integration**: Opens files with yazelix's configured editor
- **Direct zoxide jump**: `Alt+z` opens a Zoxide picker and retargets the managed editor/workspace directly to the selected directory
- **Semantic integration remaps**: `yazi.keybindings` in `settings.jsonc` remaps Yazelix-owned Yazi integration actions such as `open_zoxide_in_editor` and `open_directory_as_workspace_pane`
- **Auto layout**: Adjusts pane layout based on terminal width
- **Custom status bar**: Enhanced readability for sidebar mode

## Advanced Customization

For deeper customization beyond the built-in `settings.jsonc` options, use the managed Yazi home under `~/.config/yazelix/yazi/`.

### Custom init.lua Code

Some plugins (like `yamb` for bookmarks) require custom Lua code beyond a simple `require().setup()`. Create your own init.lua:

```bash
~/.config/yazelix/yazi/init.lua
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
~/.config/yazelix/yazi/keymap.toml
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

- `Enter`: opens selected files through Yazelix's configured editor opener
- `o`: uses Yazi's built-in open action for selected files
- `O`: opens Yazi's built-in "Open with" menu for alternate handlers, including the system file manager path when available
- `Z`: keeps Yazi's native Zoxide jump behavior inside Yazi
- `Alt+z`: runs the bundled `zoxide-editor` plugin so the selected directory becomes the managed editor/workspace target immediately

To change the file-open key itself, remap Yazi's native `open` command in `yazi/keymap.toml`; do not add an `open_selected_in_editor` entry to `settings.jsonc`

```toml
[[mgr.prepend_keymap]]
on = ["e"]
run = "open"
desc = "Open selected files"
```

Use `settings.jsonc` for the Yazelix-owned generated integration bindings:

```jsonc
{
  "yazi": {
    "keybindings": {
      "open_zoxide_in_editor": ["<A-x>"],
      "open_directory_as_workspace_pane": []
    }
  }
}
```

Omitted actions keep Yazelix defaults. Empty lists disable that generated integration binding. Multiple entries generate multiple alternate bindings for the same Yazelix-owned action. Arbitrary native Yazi actions and multi-key sequences still belong in `~/.config/yazelix/yazi/keymap.toml`.

Because `O` already exposes the practical "open outside the editor" flow, Yazelix does not add a separate default keybinding for opening in the host file manager.

This file is gitignored, so your keybindings persist across updates.

### Import an Existing Native Yazi Config

If you already have a native Yazi setup, import the supported override files, package file, plugin directories, and flavor directories into the Yazelix-managed Yazi home:

```bash
yzx import yazi
```

This imports `yazi.toml`, `keymap.toml`, `init.lua`, `package.toml`, `plugins/`, and `flavors/` from `~/.config/yazi/` into `~/.config/yazelix/yazi/`. Yazelix then copies those managed plugins and flavors into the generated runtime Yazi directory when the runtime config is refreshed.

If you need to replace existing managed override files, use:

```bash
yzx import yazi --force
```

Yazelix writes backups before overwriting existing managed files, plugin directories, or flavor directories. Broader Yazi state outside those supported inputs is intentionally not imported by this command.

## Plugin Management

Plugin catalog: https://github.com/yazi-rs/plugins

For plugin management commands, see: https://yazi-rs.github.io/docs/cli

After installing plugins via `ya pkg`, add them to `settings.jsonc`:

```jsonc
{
  "yazi": {
    "plugins": ["git", "your-new-plugin"]
  }
}
```

If the plugin was installed into native Yazi config with `ya pkg`, run `yzx import yazi` before restarting Yazelix so the managed runtime can see the plugin files.

## Troubleshooting

**Config not updating?**
```bash
# Restart Yazelix or open a fresh Yazelix window so the managed Yazi config is regenerated
yzx restart
```

- If the managed Yazi files still look stale, run `yzx doctor --fix` and restart Yazelix once more

**Plugin not loading?**
- Check plugin name in `settings.jsonc` matches installed plugin (without `.yazi` extension)
- If installed with `ya pkg`, run `yzx import yazi` and restart Yazelix
- Advanced check: verify the plugin exists in the generated runtime plugin directory under `~/.local/share/yazelix/configs/yazi/plugins/`
- Check for warnings during yazelix startup

**Migrating from a native Yazi config?**
- Run `yzx import yazi` to copy `yazi.toml`, `keymap.toml`, `init.lua`, `package.toml`, `plugins/`, and `flavors/` into the managed Yazi home
- Use `yzx import yazi --force` to back up and replace existing managed files and directories

**Want default settings?**
```bash
yzx reset config --yes
```

## Configuration Reference

Full yazi configuration docs: https://yazi-rs.github.io/docs/configuration/yazi

Yazelix exposes the most commonly changed settings via `settings.jsonc`. For advanced configuration, edit the managed Yazi files in `~/.config/yazelix/yazi/`.
