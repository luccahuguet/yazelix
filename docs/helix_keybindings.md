# Helix Keybindings Configuration for Yazelix

This guide covers recommended Helix keybindings that enhance your editing experience when using Yazelix.

## Basic Yazelix Integration

Yazelix does not ship a built-in Helix-local Yazi binding. The default workspace bindings live in Zellij: `Ctrl+y` toggles focus between the managed editor and sidebar, and `Alt+y` toggles the sidebar open or closed.

If you want an editor-local reveal action, bind `reveal_in_yazi.nu` to `Alt+r`. Yazelix now reserves `Alt+r` globally and forwards it into the editor when the managed editor is focused; outside the editor, the same key falls back to the editor/sidebar focus flow.

## Additional Recommended Keybindings

Add these keybindings to your Helix config for an enhanced editing experience:

```toml
[keys.normal]
# Navigation and movement
"{" = "goto_prev_paragraph"
"}" = "goto_next_paragraph"
g.e = "goto_file_end"
ret = ["move_line_down", "goto_first_nonwhitespace"]
A-ret = ["move_line_up", "goto_first_nonwhitespace"]

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
C-r = [":config-reload", ":reload"]
A-r = ":sh nu ~/.config/yazelix/nushell/scripts/integrations/reveal_in_yazi.nu \"%{buffer_name}\""

# Git integration
A-g.b = ":sh git blame -L %{cursor_line},+1 %{buffer_name}"
A-g.s = ":sh git status --porcelain"
A-g.l = ":sh git log --oneline -10 %{buffer_name}"

# Utility shortcuts (backspace prefix)
backspace.d = ":yank-diagnostic"
backspace.h = ":toggle-option file-picker.hidden"
backspace.i = ":toggle-option file-picker.git-ignore"
backspace.l = ":o ~/.config/helix/languages.toml"
backspace.c = ":config-open"
```

## Keybinding Categories

### Navigation and Movement
- `{` / `}`: Navigate between paragraphs
- `g.e`: Go to end of file
- `ret`: Move line down and go to first non-whitespace
- `A-ret`: Move line up and go to first non-whitespace

### Selection and Editing
- `X`: Extend selection line up
- `C-k`: Cut current line and paste above
- `C-j`: Cut current line and paste below

### System Integration
- `C-r`: Reload configuration and current file
- `A-r`: Reveal the current file in the managed Yazi sidebar

### Git Integration
- `A-g.b`: Show git blame for current line
- `A-g.s`: Show git status (porcelain format)
- `A-g.l`: Show recent git log for current file

### Utility Shortcuts (backspace prefix)
- `backspace.d`: Yank diagnostic messages
- `backspace.h`: Toggle hidden files in file picker
- `backspace.i`: Toggle git-ignore filtering in file picker
- `backspace.l`: Open Helix languages.toml
- `backspace.c`: Open main Helix configuration

## Usage Tips

1. **Git Integration**: The git keybindings help you stay in context while coding. `A-g.b` is particularly useful for understanding code history.

2. **File Picker Toggles**: Use `backspace.h` and `backspace.i` to quickly adjust what files are visible when using Helix's file picker.

3. **Yazelix Integration**: `Alt+r` is a good fit for `reveal_in_yazi.nu` because Yazelix forwards it into the editor when the managed editor is focused, while the same key behaves like `Ctrl+y` outside the editor.

## Customization

Feel free to modify these keybindings to match your workflow. The key principles are:
- Prefer editor-local bindings that do not conflict with Zellij workspace shortcuts
- Use `backspace.` for utility functions that don't interfere with normal editing
- Keep git operations grouped under `A-g.`

For more Helix configuration options, see the [Helix documentation](https://docs.helix-editor.com/configuration.html).

## Complete Keybindings Reference

For a complete list of all Yazelix keybindings across all tools, see [keybindings.md](./keybindings.md).
