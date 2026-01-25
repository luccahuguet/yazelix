# Helix Keybindings Configuration for Yazelix

This guide covers recommended Helix keybindings that enhance your editing experience when using Yazelix.

## Basic Yazelix Integration

The essential keybinding for Yazelix integration should be added to your Helix config (usually `~/.config/helix/config.toml`):

```toml
[keys.normal]
# Yazelix sidebar integration - reveal current file in Yazi sidebar
A-y = ":sh nu ~/.config/yazelix/nushell/scripts/integrations/reveal_in_yazi.nu \"%{buffer_name}\""
```

**Note:** Only works for Helix instances opened from Yazi.

## Additional Recommended Keybindings

Add these keybindings to your Helix config for an enhanced editing experience:

```toml
[keys.normal]
# Navigation and movement
"{" = "goto_prev_paragraph"
"}" = "goto_next_paragraph"
g.e = "goto_file_end"
A-ret = ["move_line_down", "goto_first_nonwhitespace"]

# Selection and editing
X = "extend_line_up"
C-k = [
  "extend_to_line_bounds",
  "delete_selection",
  "move_line_up",
  "paste_before",
]
C-j = ["extend_to_line_bounds", "delete_selection", "paste_after"]

# System integration
A-r = [":config-reload", ":reload"]

# Git integration
A-g.b = ":sh git blame -L %{cursor_line},+1 %{buffer_name}"
A-g.s = ":sh git status --porcelain"
A-g.l = ":sh git log --oneline -10 %{buffer_name}"

# Utility shortcuts (ret prefix)
ret.d = ":yank-diagnostic"
ret.h = ":toggle-option file-picker.hidden"
ret.i = ":toggle-option file-picker.git-ignore"
ret.l = ":o ~/.config/helix/languages.toml"
ret.c = ":config-open"
```

## Keybinding Categories

### Navigation and Movement
- `{` / `}`: Navigate between paragraphs
- `g.e`: Go to end of file
- `A-ret`: Move line down and go to first non-whitespace

### Selection and Editing
- `X`: Extend selection line up
- `C-k`: Cut current line and paste above
- `C-j`: Cut current line and paste below

### System Integration
- `A-r`: Reload configuration and current file
- `A-y`: **Yazelix integration** - Reveal current file in Yazi sidebar

### Git Integration
- `A-g.b`: Show git blame for current line
- `A-g.s`: Show git status (porcelain format)
- `A-g.l`: Show recent git log for current file

### Utility Shortcuts (ret prefix)
- `ret.d`: Yank diagnostic messages
- `ret.h`: Toggle hidden files in file picker
- `ret.i`: Toggle git-ignore filtering in file picker
- `ret.l`: Open Helix languages.toml
- `ret.c`: Open main Helix configuration

## Usage Tips

1. **Git Integration**: The git keybindings help you stay in context while coding. `A-g.b` is particularly useful for understanding code history.

2. **File Picker Toggles**: Use `ret.h` and `ret.i` to quickly adjust what files are visible when using Helix's file picker.

3. **Yazelix Integration**: The `A-y` keybinding creates a seamless workflow between Helix and Yazi - you can quickly reveal any file you're editing in the sidebar.

## Customization

Feel free to modify these keybindings to match your workflow. The key principles are:
- Use `Alt` (A-) for Yazelix-specific integrations
- Use `ret.` for utility functions that don't interfere with normal editing
- Keep git operations grouped under `A-g.`

For more Helix configuration options, see the [Helix documentation](https://docs.helix-editor.com/configuration.html).

## Complete Keybindings Reference

For a complete list of all Yazelix keybindings across all tools, see [keybindings.md](./keybindings.md).
