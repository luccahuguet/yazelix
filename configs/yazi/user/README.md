# User Yazi Configuration

This directory is for your custom Yazi configurations. Files here are gitignored.

## Custom init.lua

Create `init.lua` in this directory to add custom Lua code to Yazi's initialization.

Your code will be appended to the auto-generated init.lua, after the plugin `require()` statements.

### Example: yamb bookmarks plugin

```lua
-- Custom yamb setup with keybindings
require("yamb"):setup({
    jump_notify = true,
})
```

### Example: Custom keybindings

```lua
-- Add custom key mappings
local function custom_keybinds()
    -- Your keybind setup here
end
custom_keybinds()
```

## Notes

- Changes take effect on next Yazelix startup
- Your init.lua is appended after core plugins (sidebar_status, auto_layout) and user plugins from yazelix.toml
