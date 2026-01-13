# User Yazi Configuration

This directory is for your custom Yazi configurations. Files here are gitignored.

## Custom Keybindings (keymap.toml)

Create `keymap.toml` to add custom keybindings. Your keybindings are merged with yazelix defaults.

### Example: yamb bookmarks

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

### Available sections

- `mgr.append_keymap` - Manager mode keybindings (file browser)
- `mgr.prepend_keymap` - Manager keybindings with higher priority
- `input.append_keymap` - Input mode keybindings
- `cmp.append_keymap` - Completion mode keybindings

## Custom Lua Code (init.lua)

Create `init.lua` to add custom Lua code to Yazi's initialization.

Your code is appended after the plugin `require()` statements.

### Example: yamb setup

```lua
require("yamb"):setup({
    jump_notify = true,
})
```

## Notes

- Changes take effect on next Yazelix startup
- Keybindings are merged (yours appended to yazelix defaults)
- Lua code is appended after core and user plugins
